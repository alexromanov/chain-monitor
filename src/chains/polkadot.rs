use super::{Block, Chain, ChainClient, ChainMetrics};
use crate::Result;
use async_trait::async_trait;
use futures::stream::{self, Stream};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PolkadotBlock {
    number: String,
    hash: String,
    #[serde(rename = "parentHash")]
    parent_hash: String,
    extrinsics: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PolkadotHeader {
    number: String,
    #[serde(rename = "parentHash")]
    parent_hash: String,
}

pub struct PolkadotClient {
    rpc_url: String,
    client: reqwest::Client,
}

impl PolkadotClient {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            client: reqwest::Client::new(),
        }
    }
    
    async fn call_rpc(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        
        let response = self.client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| crate::Error::Chain(format!("Polkadot RPC failed: {}", e)))?;
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| crate::Error::Chain(format!("Failed to parse response: {}", e)))?;
        
        json.get("result")
            .cloned()
            .ok_or_else(|| crate::Error::Chain("No result in RPC response".to_string()))
    }
    
    fn hex_to_u64(hex: &str) -> u64 {
        let hex = hex.trim_start_matches("0x");
        u64::from_str_radix(hex, 16).unwrap_or(0)
    }
}

#[async_trait]
impl ChainClient for PolkadotClient {
    async fn get_latest_block(&self) -> Result<Block> {
        let hash = self.call_rpc("chain_getFinalizedHead", serde_json::json!([]))
            .await?
            .as_str()
            .ok_or_else(|| crate::Error::Chain("Invalid block hash".to_string()))?
            .to_string();
        
        let block_data = self.call_rpc("chain_getBlock", serde_json::json!([hash]))
            .await?;
        
        let header = &block_data["block"]["header"];
        let block_number_hex = header["number"]
            .as_str()
            .ok_or_else(|| crate::Error::Chain("No block number".to_string()))?;
        
        let block_number = Self::hex_to_u64(block_number_hex);
        
        let extrinsics = block_data["block"]["extrinsics"]
            .as_array()
            .map(|arr| arr.len())
            .unwrap_or(0);
        
        let timestamp = chrono::Utc::now().timestamp();
        
        Ok(Block {
            chain: Chain::Polkadot,
            height: block_number,
            hash,
            timestamp,
            transaction_count: extrinsics,
            size: 0,
        })
    }
    
    async fn get_block_by_height(&self, height: u64) -> Result<Block> {
        let height_hex = format!("0x{:x}", height);
        
        let hash = self.call_rpc("chain_getBlockHash", serde_json::json!([height_hex]))
            .await?
            .as_str()
            .ok_or_else(|| crate::Error::Chain("Invalid block hash".to_string()))?
            .to_string();
        
        let block_data = self.call_rpc("chain_getBlock", serde_json::json!([hash]))
            .await?;
        
        let extrinsics = block_data["block"]["extrinsics"]
            .as_array()
            .map(|arr| arr.len())
            .unwrap_or(0);
        
        let timestamp = chrono::Utc::now().timestamp();
        
        Ok(Block {
            chain: Chain::Polkadot,
            height,
            hash,
            timestamp,
            transaction_count: extrinsics,
            size: 0,
        })
    }
    
    fn subscribe_blocks(&self) -> impl Stream<Item = Result<Block>> + Send {
        let client = self.clone();
        
        stream::unfold((), move |_| {
            let client = client.clone();
            async move {
                tokio::time::sleep(Duration::from_secs(6)).await;
                
                match client.get_latest_block().await {
                    Ok(block) => Some((Ok(block), ())),
                    Err(e) => Some((Err(e), ())),
                }
            }
        })
    }
    
    async fn get_metrics(&self) -> Result<ChainMetrics> {
        let latest = self.get_latest_block().await?;
        let prev = self.get_block_by_height(latest.height - 1).await?;
        
        let block_time = (latest.timestamp - prev.timestamp).abs() as f64;
        let block_time = if block_time > 0.0 { block_time } else { 6.0 }; // Default to 6s
        
        let tps = latest.transaction_count as f64 / block_time;
        
        Ok(ChainMetrics {
            chain: Chain::Polkadot,
            tps,
            block_time,
            pending_transactions: 0,
            peer_count: 0,
        })
    }
}

impl Clone for PolkadotClient {
    fn clone(&self) -> Self {
        Self {
            rpc_url: self.rpc_url.clone(),
            client: self.client.clone(),
        }
    }
}