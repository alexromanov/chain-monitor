pub mod bitcoin;
pub mod ethereum;
pub mod cardano;
pub mod solana;
pub mod polkadot;
pub mod types;

pub use types::*;

use async_trait::async_trait;
use crate::Result;
use futures::stream::Stream;

#[async_trait]
pub trait ChainClient: Send + Sync {
    async fn get_latest_block(&self) -> Result<Block>;
    async fn get_block_by_height(&self, height: u64) -> Result<Block>;
    fn subscribe_blocks(&self) -> impl Stream<Item = Result<Block>> + Send;
    async fn get_metrics(&self) -> Result<ChainMetrics>;
}

/// Enum wrapper for different chain clients (to enable dynamic dispatch)
#[derive(Clone)]
pub enum AnyChainClient {
    Bitcoin(bitcoin::BitcoinClient),
    Ethereum(ethereum::EthereumClient),
    Cardano(cardano::CardanoClient),
    Solana(solana::SolanaClient),
    Polkadot(polkadot::PolkadotClient),
}

#[async_trait]
impl ChainClient for AnyChainClient {
    async fn get_latest_block(&self) -> Result<Block> {
        match self {
            AnyChainClient::Bitcoin(c) => c.get_latest_block().await,
            AnyChainClient::Ethereum(c) => c.get_latest_block().await,
            AnyChainClient::Cardano(c) => c.get_latest_block().await,
            AnyChainClient::Solana(c) => c.get_latest_block().await,
            AnyChainClient::Polkadot(c) => c.get_latest_block().await,
        }
    }

    async fn get_block_by_height(&self, height: u64) -> Result<Block> {
        match self {
            AnyChainClient::Bitcoin(c) => c.get_block_by_height(height).await,
            AnyChainClient::Ethereum(c) => c.get_block_by_height(height).await,
            AnyChainClient::Cardano(c) => c.get_block_by_height(height).await,
            AnyChainClient::Solana(c) => c.get_block_by_height(height).await,
            AnyChainClient::Polkadot(c) => c.get_block_by_height(height).await,
        }
    }

    fn subscribe_blocks(&self) -> impl Stream<Item = Result<Block>> + Send {
        use futures::stream::BoxStream;
        
        let stream: BoxStream<Result<Block>> = match self {
            AnyChainClient::Bitcoin(c) => Box::pin(c.subscribe_blocks()),
            AnyChainClient::Ethereum(c) => Box::pin(c.subscribe_blocks()),
            AnyChainClient::Cardano(c) => Box::pin(c.subscribe_blocks()),
            AnyChainClient::Solana(c) => Box::pin(c.subscribe_blocks()),
            AnyChainClient::Polkadot(c) => Box::pin(c.subscribe_blocks()),
        };
        
        stream
    }

    async fn get_metrics(&self) -> Result<ChainMetrics> {
        match self {
            AnyChainClient::Bitcoin(c) => c.get_metrics().await,
            AnyChainClient::Ethereum(c) => c.get_metrics().await,
            AnyChainClient::Cardano(c) => c.get_metrics().await,
            AnyChainClient::Solana(c) => c.get_metrics().await,
            AnyChainClient::Polkadot(c) => c.get_metrics().await,
        }
    }
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

    pub fn all() -> Vec<Chain> {
        vec![
            Chain::Bitcoin,
            Chain::Ethereum,
            Chain::Cardano,
            Chain::Solana,
            Chain::Polkadot,
        ]
    }
}

impl std::str::FromStr for Chain {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "bitcoin" => Ok(Chain::Bitcoin),
            "ethereum" => Ok(Chain::Ethereum),
            "cardano" => Ok(Chain::Cardano),
            "solana" => Ok(Chain::Solana),
            "polkadot" => Ok(Chain::Polkadot),
            _ => Err(crate::Error::Chain(format!("Invalid chain: {}", s))),
        }
    }
}