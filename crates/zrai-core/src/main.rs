//! ZRAI daemon entry point.

use std::sync::Arc;

use zrai_core::config::Config;
use zrai_core::proxy::{self, AppState};
use zrai_core::sanitize;

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

    let sanitizer = sanitize::build(&config);
    let client = reqwest::Client::builder().build()?;

    let state = AppState {
        client,
        config: Arc::new(config),
        sanitizer,
    };

    let app = proxy::router(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("ZRAI listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
