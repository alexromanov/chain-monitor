use crate::chains::{bitcoin::BitcoinClient, Chain, ChainClient};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub clients: HashMap<Chain, Arc<dyn ChainClient>>,
}

impl AppState {
    pub fn new() -> Self {
        let mut clients: HashMap<Chain, Arc<dyn ChainClient>> = HashMap::new();
        
        // Initialize Bitcoin client
        let bitcoin_client = BitcoinClient::new(
            std::env::var("BITCOIN_RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8332".to_string())
        );
        
        clients.insert(Chain::Bitcoin, Arc::new(bitcoin_client));
        
        Self { clients }
    }
}