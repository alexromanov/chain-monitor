use super::{Block, Chain, ChainClient, ChainMetrics};
use crate::Result;
use async_trait::async_trait;
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct BlockfrostBlock {
    height: Option<u64>,
    hash: String,
    time: i64,
    #[serde(rename = "tx_count")]
    transaction_count: usize,
    size: u64,
    slot: u64,
    epoch: u32,
}

#[derive(Debug, Deserialize)]
struct BlockfrostNetwork {
    stake: NetworkStake,
    supply: NetworkSupply,
}

#[derive(Debug, Deserialize)]
struct NetworkStake {
    live: String,
    active: String,
}

#[derive(Debug, Deserialize)]
struct NetworkSupply {
    max: String,
    circulating: String,
}

pub struct CardanoClient {
    api_url: String,
    project_id: String,
    client: reqwest::Client,
}

impl CardanoClient {
    pub fn new(network: CardanoNetwork, project_id: String) -> Self {
        let api_url = match network {
            CardanoNetwork::Mainnet => "https://cardano-mainnet.blockfrost.io/api/v0",
            CardanoNetwork::Preprdo => "https://cardano-preprod.blockfrost.io/api/v0",
            CardanoNetwork::Preview => "https://cardano-preview.blockfrost.io/api/v0",
        };

        Self {
            api_url: api_url.to_string(),
            project_id,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let url = format("{}{}", self.api_url, endpoint);

        let response = self.client.get(&url).header("project_id", &self.project_id).send().await.map_err(|e| crate::Error::Chain(format!("Blockfrost API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(crate::Error::Chain(format!("Blockfrost API error: {} - {}", response.status(), response.text().await.unwrap_or_default)))
        }

        response.json().await.map_err(|e| crate::Error::Chain(format!("Failed to parse response: {}", e )));
    }

    async fn get_block_by_identifier(&self, identifier: &str) -> Result<Block> {
        let block: BlockfrostBlock = self.fetch("/blocks/{}", identifier).await?;

        Ok(Block {
            chain: Chain::Cardano,
            height: block.height.unwrap_or(0),
            hash: block.hash,
            timestamp: block.time,
            transaction_count: block.transaction_count,
            size: block.size,
        })
    }
}

#[async_trait]
impl ChainClient for CardanoClient {
    async fn get_latest_block(&self) -> Result<Block> {
        self.get_block_by_identifier("latest").await
    }
    
    async fn get_block_by_height(&self, height: u64) -> Result<Block> {
        self.get_block_by_identifier(&height.to_string()).await
    }
    
    fn subscribe_blocks(&self) -> impl Stream<Item = Result<Block>> + Send {
        let client = self.clone();
        let mut last_hash = String::new();
        
        stream::unfold(last_hash, move |mut prev_hash| {
            let client = client.clone();
            async move {
                tokio::time::sleep(Duration::from_secs(20)).await;
                
                match client.get_latest_block().await {
                    Ok(block) => {
                        if block.hash != prev_hash {
                            prev_hash = block.hash.clone();
                            Some((Ok(block), prev_hash))
                        } else {
                            Some((Ok(block), prev_hash))
                        }
                    }
                    Err(e) => Some((Err(e), prev_hash)),
                }
            }
        })
    }
    
    async fn get_metrics(&self) -> Result<ChainMetrics> {
        let latest_block = self.get_latest_block().await?;
        
        let block_time = 20.0;
        
        let tps = latest_block.transaction_count as f64 / block_time;
        
        let _network_info: BlockfrostNetwork = self.fetch("/network").await
            .unwrap_or_else(|_| {
                serde_json::from_str(r#"{"stake":{"live":"0","active":"0"},"supply":{"max":"45000000000000000","circulating":"0"}}"#).unwrap()
            });
        
        Ok(ChainMetrics {
            chain: Chain::Cardano,
            tps,
            block_time,
            pending_transactions: 0,
            peer_count: 0,
        })
    }
}

impl Clone for CardanoClient {
    fn clone(&self) -> Self {
        Self {
            api_url: self.api_url.clone(),
            project_id: self.project_id.clone(),
            client: self.client.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CardanoNetwork {
    Mainnet,
    Preprod,
    Preview,
}