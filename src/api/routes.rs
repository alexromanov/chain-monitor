use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use std::str::FromStr;
use crate::chains::{Chain, ChainClient};
use super::AppState;

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn list_chains(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<serde_json::Value>> {
    let chains: Vec<serde_json::Value> = state.clients
        .iter()
        .map(|(chain, _)| {
            serde_json::json!({
                "name": chain.name(),
                "chain": chain,
            })
        })
        .collect();
    
    Json(chains)
}

pub async fn get_latest_block(
    State(state): State<Arc<AppState>>,
    Path(chain_name): Path<String>,
) -> Result<Json<crate::chains::Block>, StatusCode> {
    let chain = Chain::from_str(&chain_name)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let client = state.clients
        .get(&chain)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let block = client.get_latest_block()
        .await
        .map_err(|e| {
            tracing::error!("Failed to get block: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(block))
}

pub async fn get_chain_metrics(
    State(state): State<Arc<AppState>>,
    Path(chain_name): Path<String>,
) -> Result<Json<crate::chains::ChainMetrics>, StatusCode> {
    let chain = Chain::from_str(&chain_name)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let client = state.clients
        .get(&chain)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let metrics = client.get_metrics()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(metrics))
}

pub async fn get_block_by_height(
    State(state): State<Arc<AppState>>,
    Path((chain_name, height)): Path<(String, u64)>,
) -> Result<Json<crate::chains::Block>, StatusCode> {
    let chain = Chain::from_str(&chain_name)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let client = state.clients
        .get(&chain)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let block = client.get_block_by_height(height)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(block))
}