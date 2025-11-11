pub mod bitcoin;
pub mod types;

pub use types::*;

use async_trait::async_trait;
use crate::Result;
use futures::Stream;
use std::pin::Pin;

pub type BlockStream = Pin<Box<dyn Stream<Item = Result<Block>> + Send>>;

#[async_trait]
pub trait ChainClient: Send + Sync {
    async fn get_latest_block(&self) -> Result<Block>;
    async fn get_block_by_height(&self, height: u64) -> Result<Block>;
    fn subscribe_blocks(&self) -> BlockStream;
    async fn get_metrics(&self) -> Result<ChainMetrics>;
}

/// Chain identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Chain {
    Bitcoin,
    Ethereum,
    Cardano,
    Solana,
    Polkadot,
}

impl Chain {
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Bitcoin => "bitcoin",
            Chain::Ethereum => "ethereum",
            Chain::Cardano => "cardano",
            Chain::Solana => "solana",
            Chain::Polkadot => "polkadot",
        }
    }
}