//! End-to-end test of the proxy request path:
//! client → ZRAI (sanitize + ZDR) → mock upstream.
//!
//! Uses the deterministic `NativeSanitizer` so the test does not depend on the generated
//! TypeScript bundle. The embedded JS engine is covered separately in `js_sanitizer` tests.

use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::HeaderMap,
    routing::post,
    Json, Router,
};
use bytes::Bytes;
use serde_json::{json, Value};

use zrai_core::config::Config;
use zrai_core::proxy::{self, AppState};
use zrai_core::sanitize::NativeSanitizer;

#[derive(Clone)]
struct Captured {
    headers: HeaderMap,
    body: Value,
}

async fn capture_handler(
    State(store): State<Arc<Mutex<Option<Captured>>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Json<Value> {
    let body = serde_json::from_slice(&body).unwrap_or(Value::Null);
    *store.lock().unwrap() = Some(Captured { headers, body });
    Json(json!({ "ok": true }))
}

/// Start a mock upstream that records the last request it received.
async fn spawn_mock() -> (String, Arc<Mutex<Option<Captured>>>) {
    let store: Arc<Mutex<Option<Captured>>> = Arc::new(Mutex::new(None));
    let app = Router::new()
        .route("/v1/chat/completions", post(capture_handler))
        .route("/v1/messages", post(capture_handler))
        .with_state(store.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}"), store)
}

/// Start ZRAI pointed at `upstream` and return its base URL.
async fn spawn_zrai(upstream: String) -> String {
    let config = Config {
        upstream: Some(upstream),
        zdr: Some(json!({ "provider": { "zdr": true } })),
        ..Default::default()
    };
    let state = AppState {
        client: reqwest::Client::new(),
        config: Arc::new(config),
        sanitizer: Arc::new(NativeSanitizer::new(false, &[])),
    };
    let app = proxy::router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    format!("http://{addr}")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn forwards_sanitized_payload_with_zdr_and_auth() {
    let (mock_base, store) = spawn_mock().await;
    let zrai_base = spawn_zrai(mock_base).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{zrai_base}/v1/chat/completions"))
        .header("authorization", "Bearer client-key-123")
        .json(&json!({
            "model": "gpt-test",
            "messages": [
                { "role": "user", "content": "see /home/alice/secret.py and mail jane@corp.com" }
            ]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let cap = store
        .lock()
        .unwrap()
        .clone()
        .expect("upstream should have received a request");

    // Context was sanitized before egress.
    assert_eq!(
        cap.body["messages"][0]["content"],
        json!("see <path> and mail <email>")
    );
    // ZDR was injected.
    assert_eq!(cap.body["provider"]["zdr"], json!(true));
    // Model passed through untouched.
    assert_eq!(cap.body["model"], json!("gpt-test"));
    // Client credentials were forwarded to the upstream.
    assert_eq!(
        cap.headers.get("authorization").unwrap(),
        "Bearer client-key-123"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accepts_anthropic_route() {
    let (mock_base, store) = spawn_mock().await;
    let zrai_base = spawn_zrai(mock_base).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{zrai_base}/v1/messages"))
        .json(&json!({
            "model": "claude-test",
            "messages": [ { "role": "user", "content": "open C:\\Users\\bob\\k.pem" } ]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let cap = store.lock().unwrap().clone().expect("captured");
    assert_eq!(cap.body["messages"][0]["content"], json!("open <path>"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rejects_unknown_route() {
    let (mock_base, _store) = spawn_mock().await;
    let zrai_base = spawn_zrai(mock_base).await;

    let resp = reqwest::Client::new()
        .post(format!("{zrai_base}/v1/embeddings"))
        .json(&json!({ "input": "x" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}
