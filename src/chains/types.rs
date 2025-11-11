use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub chain: super::Chain,
    pub height: u64,
    pub hash: String,
    pub timestamp: i64,
    pub transaction_count: usize,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: String,
    pub fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetrics {
    pub chain: super::Chain,
    pub tps: f64,
    pub block_time: f64,
    pub pending_transactions: usize,
    pub peer_count: u32,
}