use super::{Block, Chain, ChainClient, ChainMetrics};
use crate::Result;
use async_trait::async_trait;
use futures::stream::{self, Stream};
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::RpcBlockConfig,
};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::{UiTransactionEncoding, TransactionDetails};
use std::sync::Arc;
use std::time::Duration;

pub struct SolanaClient {
    client: Arc<RpcClient>,
}

impl SolanaClient {
    pub fn new(rpc_url: String) -> Self {
        let client = RpcClient::new_with_commitment(
            rpc_url, 
            CommitmentConfig::confirmed());

        Self {
            client: Arc::new(client),
        }
    }

    async fn fetch_block_details(&self, slot: u64) -> Result<Block> {
        let config = RpcBlockConfig {
            encoding: Some(UiTransactionEncoding::Base64),
            transaction_details: Some(TransactionDetails::Signatures),
            rewards: Some(false),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        };

        let block = self.client.get_block_with_config(slot, config)
            .await
            .map_err(|e| crate::Error::Chain(format!("Failed to get Solana block: {}", e)))?;

        let block_time = block.block_time.unwrap_or(0);
        let transaction_count = block.transactions.map(|txs| txs.len()).unwrap_or(0);
        let block_hash = block.blockhash;

        Ok(Block {
            chain: Chain::Solana,
            height: slot,
            hash: blockhash,
            timestamp: block_time,
            transaction_count,
            size: 0,
        })
    }
}

#[async_trait]
impl ChainClient for SolanaClient {
    async fn get_latest_block(&self) -> Result<Block> {
        let slot = self.client.get_slot().await.map_err(|e| crate::Error::Chain(format!("Failed to get latest slot: {}", e)))?;
        self.fetch_block_details(slot).await
    }

    async fn get_block_by_height(&self, height: u64) -> Result<Block> {
        self.fetch_block_details(height).await
    }

    fn subscribe_blocks(&self) -> impl Stream<Item = Result<Block>> + Send {
        let client = self.client.clone();
        let mut last_slot = 0u64;

        stream::unfold(last_slot, move | mut prev_slot| {
            let client = client.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(400)).await;

                match client.get_slot().await {
                    Ok(slot) => {
                        if slot > prev_slot {
                            prev_slot = slot;

                            let config = RpcBlockConfig {
                                encoding: Some(UiTransactionEncoding::Base64),
                                transaction_details: Some(TransactionDetails::Signatures),
                                rewards: Some(false),
                                commitment: Some(CommitmentConfig::confirmed()),
                                max_supported_transaction_version: Some(0),
                            };
                            
                            match client.get_block_with_config(slot, config).await {
                                Ok(block) => {
                                    let block_result = Block {
                                        chain: Chain::Solana,
                                        height: slot,
                                        hash: block.blockhash,
                                        timestamp: block.block_time.unwrap_or(0),
                                        transaction_count: block.transactions.map(|txs| txs.len()).unwrap_or(0),
                                        size: 0,
                                    };
                                    Some((Ok(block_result), prev_slot))
                                }
                                Err(e) => {
                                    Some((Err(crate::Error::Chain(e.to_string())), prev_slot))
                                }
                            }
                        } else {
                            Some((
                                Err(crate::Error::Chain("No new block".to_string())),
                                prev_slot,
                            ))
                        }
                    }
                    Err(e) => {
                        Some((Err(crate::Error::Chain(e.to_string())), prev_slot))
                    }
                }
            }
        })
        .filter_map(|result| async move {
            match result {
                Ok(block) => Some(Ok(block)),
                Err(e) if e.to_string().contains("No new block") => None,
                Err(e) => Some(Err(e)),
            }
        })
    }

    async fn get_metrics(&self) -> Result<ChainMetrics> {
        let samples = self.client.get_recent_performance_samples()
            .await
            .map_err(|e| crate::Error::Chain(format!("Failed to get performance samples: {}", e)))?;

        let avg_tps = if !samples.is_empty() {
            let total_tps: u64 = samples.iter().map(|s| s.num_transactions / s.sample_period_secs).sum();
            (total_tps as f64) / (samples.len() as f64)
        } else {
            0.0
        };

        let current_slot = self.client.get_slot().await
            .map_err(|e| crate::Error::Chain(e.to_string()))?;

        let prev_slot = current_slot.saturating_sub(1);

        let current_time = self.client.get_block_time(current_slot).await
            .map_err(|e| crate::Error::Chain(e.to_string()))?;

        let prev_time = self.client.get_block_time(prev_slot).await
            .map_err(|e| crate::Error::Chain(e.to_string()))?;

        let block_time = (current_time - prev_time) as f64;

        Ok(ChainMetrics {
            chain: Chain::Solana,
            tps: avg_tps,
            block_time,
            pending_transactions: 0,
            peer_count: 0,
        })
    }
}

impl Clone for SolanaClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}