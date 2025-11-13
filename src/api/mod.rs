mod routes;
mod state;

pub use state::AppState;

use axum::{
    Router,
    routing::get,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(routes::health))
        .route("/api/chains", get(routes::list_chains))
        .route("/api/chains/:chain/latest", get(routes::get_latest_block))
        .route("/api/chains/:chain/metrics", get(routes::get_chain_metrics))
        .route("/api/chains/:chain/blocks/:height", get(routes::get_block_by_height))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn start_server(state: Arc<AppState>, addr: &str) -> crate::Result<()> {
    let app = create_router(state);
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::Error::Api(format!("Failed to bind: {}", e)))?;
    
    tracing::info!("Server listening on {}", addr);
    
    axum::serve(listener, app)
        .await
        .map_err(|e| crate::Error::Api(format!("Server error: {}", e)))?;
    
    Ok(())
}