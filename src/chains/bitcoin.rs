use super::{Block, Chain, ChainMetrics, ChainClient};
use crate::Result;
use async_trait::async_trait;
use futures::stream::{self, Stream};
use serde_json::json;

pub struct BitcoinClient {
    rpc_url: String,
    client: reqwest::Client,
}

impl BitcoinClient {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            client: reqwest::Client::new(),
        }
    }

    async fn call_rpc(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": method,
            "params": params
        });
        
        let response = self.client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| crate::Error::Chain(format!("RPC call failed: {}", e)))?;
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| crate::Error::Chain(format!("Failed to parse response: {}", e)))?;
        
        json.get("result")
            .cloned()
            .ok_or_else(|| crate::Error::Chain("No result in response".to_string()))
    }
}

#[async_trait]
impl ChainClient for BitcoinClient {
    async fn get_latest_block(&self) -> Result<Block> {
        let height: u64 = self.call_rpc("getblockcount", json!([]))
            .await?
            .as_u64()
            .ok_or_else(|| crate::Error::Chain("Invalid block height".to_string()))?;

        self.get_block_by_height(height).await
    }

    async fn get_block_by_height(&self, height: u64) -> Result<Block> {
        let block_hash = self.call_rpc("getblockhash", json!([height]))
            .await?
            .as_str()
            .ok_or_else(|| crate::Error::Chain("Invalid block hash".to_string()))?
            .to_string();
        
        let block_data = self.call_rpc("getblock", json!([block_hash, 1]))
            .await?;

        Ok(Block {
            chain: Chain::Bitcoin,
            height,
            hash: block_hash,
            timestamp: block_data["time"].as_i64().unwrap_or(0),
            transaction_count: block_data["tx"].as_array().map(|v| v.len()).unwrap_or(0),
            size: block_data["size"].as_u64().unwrap_or(0),
        })
    }

    fn subscribe_blocks(&self) -> impl Stream<Item = Result<Block>> + Send {
        let client = self.clone();
        
        stream::unfold((), move |_| {
            let client = client.clone();
            async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                match client.get_latest_block().await {
                    Ok(block) => Some((Ok(block), ())),
                    Err(e) => Some((Err(e), ())),
                }
            }
        })
    }

    async fn get_metrics(&self) -> Result<ChainMetrics> {
        Ok(ChainMetrics {
            chain: Chain::Bitcoin,
            tps: 7.0,
            block_time: 600.0,
            pending_transactions: 0,
            peer_count: 0,
        })
    }
}

impl Clone for BitcoinClient {
    fn clone(&self) -> Self {
        Self {
            rpc_url: self.rpc_url.clone(),
            client: self.client.clone(),
        }
    }
}