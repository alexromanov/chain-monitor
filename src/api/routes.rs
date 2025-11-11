use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::chains::Chain;
use super::AppState;

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn list_chains(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<String>> {
    let chains: Vec<String> = state.clients
        .keys()
        .map(|c| c.name().to_string())
        .collect();
    
    Json(chains)
}

pub async fn get_latest_block(
    State(state): State<Arc<AppState>>,
    Path(chain_name): Path<String>,
) -> Result<Json<crate::chains::Block>, StatusCode> {
    let chain = match chain_name.as_str() {
        "bitcoin" => Chain::Bitcoin,
        _ => return Err(StatusCode::NOT_FOUND),
    };
    
    let client = state.clients
        .get(&chain)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let block = client.get_latest_block()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(block))
}