//! ZRAI daemon entry point.

use std::sync::Arc;

use zrai_core::config::Config;
use zrai_core::proxy::{self, AppState};
use zrai_core::sanitize::NativeSanitizer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // The single allowed disk read: config, once, at startup.
    let config = Config::load("zrai.toml")?;
    let port = config.port();

    let sanitizer = Arc::new(NativeSanitizer::new(config.strict, &config.rules));
    let client = reqwest::Client::builder().build()?;

    let state = AppState {
        client,
        config: Arc::new(config),
        sanitizer,
    };

    let app = axum::Router::new()
        .route("/v1/chat/completions", axum::routing::post(proxy::proxy))
        .route("/v1/messages", axum::routing::post(proxy::proxy))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("ZRAI listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
