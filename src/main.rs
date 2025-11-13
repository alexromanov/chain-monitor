use chain_monitor::api::{AppState, start_server};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();
    
    tracing::info!("Starting Multi-Chain Monitor API...");
    
    let state = Arc::new(AppState::new().await?);
    tracing::info!("Initialized {} chain clients", state.clients.len());
    
    start_server(state, "127.0.0.1:3000").await?;
    
    Ok(())
}