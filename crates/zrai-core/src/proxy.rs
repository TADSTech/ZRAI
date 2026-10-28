//! The proxy handler: ingress → sanitize → ZDR → egress → stream back.
//!
//! Everything is processed in memory. The request body is read into a `Bytes` buffer, parsed,
//! sanitized, re-serialized, forwarded, and the upstream response body is streamed straight
//! back to the client. Nothing is written to disk, and per-request state is dropped when the
//! handler returns / the connection closes.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Method, StatusCode, Uri},
    response::Response,
};
use bytes::Bytes;
use serde_json::Value;

use crate::{config::Config, routes, sanitize::Sanitizer, zdr};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub config: Arc<Config>,
    pub sanitizer: Arc<dyn Sanitizer>,
}

/// Build the axum router for the two supported API shapes.
pub fn router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/v1/chat/completions", axum::routing::post(proxy))
        .route("/v1/messages", axum::routing::post(proxy))
        .with_state(state)
}

/// Single handler for both API shapes. The route is resolved from the request path.
pub async fn proxy(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, (StatusCode, String)> {
    if method != Method::POST {
        return Err((
            StatusCode::METHOD_NOT_ALLOWED,
            "only POST is supported".to_string(),
        ));
    }

    let route = routes::route(uri.path()).ok_or((
        StatusCode::NOT_FOUND,
        format!("unknown route: {}", uri.path()),
    ))?;

    // 1. Parse in memory.
    let parsed: Value = serde_json::from_slice(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid JSON: {e}")))?;

    // 2. Sanitize (delegated to the rule layer).
    let sanitized = state.sanitizer.sanitize(&parsed);
    let mut payload = sanitized.value;

    // 3. Enforce ZDR upstream.
    zdr::inject(&mut payload, state.config.zdr.as_ref());

    // 4. Egress.
    let base = state
        .config
        .upstream
        .clone()
        .unwrap_or_else(|| route.default_base.to_string());
    let url = format!("{}{}", base.trim_end_matches('/'), route.path);

    let mut req = state
        .client
        .post(&url)
        .header(header::CONTENT_TYPE, "application/json");
    // Forward only the credentials the upstream needs; never the client's User-Agent/IP.
    if let Some(auth) = headers.get(header::AUTHORIZATION) {
        req = req.header(header::AUTHORIZATION, auth);
    }
    if let Some(key) = headers.get("x-api-key") {
        req = req.header("x-api-key", key);
    }

    let body_bytes = serde_json::to_vec(&payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("encode error: {e}")))?;

    let upstream = req.body(body_bytes).send().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("upstream request failed: {e}"),
        )
    })?;

    // 5. Stream the upstream response back verbatim (SSE or JSON).
    let status = upstream.status();
    let mut builder = Response::builder().status(status);
    if let Some(ct) = upstream.headers().get(header::CONTENT_TYPE) {
        builder = builder.header(header::CONTENT_TYPE, ct);
    }
    let stream = upstream.bytes_stream();
    builder
        .body(Body::from_stream(stream))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("response error: {e}")))
}
