use super::{Block, Chain, ChainClient, ChainMetrics};
use crate::Result;
use async_trait::async_trait;
use ethers::{
    prelude::*,
    providers::{Provider, Ws},
};
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;

pub struct EthereumClient {
    provider: Arc<Provider<Ws>>,
    chain: Chain,
}

impl EthereumClient {
    pub async fn new(ws_url: String) -> Result<Self> {
        let provider = Provider::<Ws>::connect(&ws_url)
            .await
            .map_err(|e| crate::Error::Chain(format!("Failed to connect to Ethereum: {}", e)))?;
        
        Ok(Self {
            provider: Arc::new(provider),
            chain: Chain::Ethereum,
        })
    }
    
    async fn get_block_details(&self, block_number: U64) -> Result<Block> {
        let block = self.provider
            .get_block_with_txs(block_number)
            .await
            .map_err(|e| crate::Error::Chain(format!("Failed to get block: {}", e)))?
            .ok_or_else(|| crate::Error::Chain("Block not found".to_string()))?;
        
        Ok(Block {
            chain: self.chain,
            height: block.number.unwrap_or_default().as_u64(),
            hash: format!("{:?}", block.hash.unwrap_or_default()),
            timestamp: block.timestamp.as_u64() as i64,
            transaction_count: block.transactions.len(),
            size: block.size.unwrap_or_default().as_u64(),
        })
    }
}

#[async_trait]
impl ChainClient for EthereumClient {
    async fn get_latest_block(&self) -> Result<Block> {
        let block_number = self.provider
            .get_block_number()
            .await
            .map_err(|e| crate::Error::Chain(format!("Failed to get block number: {}", e)))?;
        
        self.get_block_details(block_number).await
    }
    
    async fn get_block_by_height(&self, height: u64) -> Result<Block> {
        self.get_block_details(U64::from(height)).await
    }
    
    fn subscribe_blocks(&self) -> impl Stream<Item = Result<Block>> + Send {
        let provider = self.provider.clone();
        let chain = self.chain;
        
        async_stream::stream! {
            let mut stream = match provider.subscribe_blocks().await {
                Ok(s) => s,
                Err(e) => {
                    yield Err(crate::Error::Chain(format!("Subscription failed: {}", e)));
                    return;
                }
            };
            
            while let Some(block) = stream.next().await {
                yield Ok(Block {
                    chain,
                    height: block.number.unwrap_or_default().as_u64(),
                    hash: format!("{:?}", block.hash.unwrap_or_default()),
                    timestamp: block.timestamp.as_u64() as i64,
                    transaction_count: block.transactions.len(),
                    size: block.size.unwrap_or_default().as_u64(),
                });
            }
        }
    }
    
    async fn get_metrics(&self) -> Result<ChainMetrics> {
        let block_number = self.provider.get_block_number().await
            .map_err(|e| crate::Error::Chain(format!("Failed to get metrics: {}", e)))?;
        
        // Get last 2 blocks to calculate block time
        let latest_block = self.provider.get_block(block_number).await
            .map_err(|e| crate::Error::Chain(e.to_string()))?
            .ok_or_else(|| crate::Error::Chain("Block not found".to_string()))?;
        
        let prev_block = self.provider.get_block(block_number - 1).await
            .map_err(|e| crate::Error::Chain(e.to_string()))?
            .ok_or_else(|| crate::Error::Chain("Previous block not found".to_string()))?;
        
        let block_time = (latest_block.timestamp - prev_block.timestamp).as_u64() as f64;
        
        // Get pending transaction count
        let pending_tx_count = match self.provider.txpool_content().await {
            Ok(content) => content.pending.len() + content.queued.len(),
            Err(_) => 0, // Fallback if txpool is not available
        };
        
        // Calculate TPS based on recent blocks
        let tx_count = latest_block.transactions.len();
        let tps = if block_time > 0.0 {
            tx_count as f64 / block_time
        } else {
            0.0
        };
        
        Ok(ChainMetrics {
            chain: self.chain,
            tps,
            block_time,
            pending_transactions: pending_tx_count,
            peer_count: 0, // Would need admin API access
        })
    }
}

impl Clone for EthereumClient {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            chain: self.chain,
        }
    }
}