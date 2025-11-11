use chain_monitor::{AppState, start_server};
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    tracing::info!("Starting Chain Monitor API...");

    let state = Arc::new(AppState::new());

    start_server(state, "127.0.0.1:3000").await?;

    Ok(())
}