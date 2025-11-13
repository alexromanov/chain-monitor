use crate::chains::{
    bitcoin::BitcoinClient,
    ethereum::EthereumClient,
    cardano::{CardanoClient, CardanoNetwork},
    solana::SolanaClient,
    polkadot::PolkadotClient,
    Chain,
    AnyChainClient,
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    pub clients: HashMap<Chain, AnyChainClient>,
}

impl AppState {
    pub async fn new() -> Self {
        let mut clients: HashMap<Chain, AnyChainClient> = HashMap::new();
        
        if let Ok(rpc_url) = std::env::var("BITCOIN_RPC_URL") {
            let client = BitcoinClient::new(rpc_url);
            clients.insert(Chain::Bitcoin, AnyChainClient::Bitcoin(client));
            tracing::info!("Initialized Bitcoin client");
        }

        if let Ok(ws_url) = std::env::var("ETHEREUM_WS_URL") {
            match EthereumClient::new(ws_url).await {
                Ok(client) => {
                    clients.insert(Chain::Ethereum, AnyChainClient::Ethereum(client));
                    tracing::info!("Initialized Ethereum client");
                }
                Err(e) => {
                    tracing::error!("Failed to initialize Ethereum client: {}", e);
                }
            }
        }

        if let Ok(project_id) = std::env::var("CARDANO_PROJECT_ID") {
            let network = std::env::var("CARDANO_NETWORK")
                .unwrap_or_else(|_| "preview".to_string());

            let network = match network.as_str() {
                "mainnet" => CardanoNetwork::Mainnet,
                "preprod" => CardanoNetwork::Preprod,
                _ => CardanoNetwork::Preview,
            };

            let client = CardanoClient::new(network, project_id);
            clients.insert(Chain::Cardano, AnyChainClient::Cardano(client));
            tracing::info!("Initialized Cardano client");
        }

        if let Ok(rpc_url) = std::env::var("SOLANA_RPC_URL") {
            let client = SolanaClient::new(rpc_url);
            clients.insert(Chain::Solana, AnyChainClient::Solana(client));
            tracing::info!("Initialized Solana client");
        }

        if let Ok(rpc_url) = std::env::var("POLKADOT_RPC_URL") {
            let client = PolkadotClient::new(rpc_url);
            clients.insert(Chain::Polkadot, AnyChainClient::Polkadot(client));
            tracing::info!("Initialized Polkadot client");
        }

        if clients.is_empty() {
            tracing::warn!("No chain clients configured. Please set environment variables for at least one chain.");
        }
        
        Self { clients }
    }
}