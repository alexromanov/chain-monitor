use chain_monitor::chains::{bitcoin::BitcoinClient, Chain, ChainClient};

#[tokio::test]
async fn test_bitcoint_client_creation() {
    let _client = BitcoinClient::new("http://localhost:8332".to_string());
    assert!(true);
}

#[tokio::test]
async fn test_chain_metrics() {
    let client = BitcoinClient::new("http://localhost:8332".to_string());
    let metrics = client.get_metrics().await.unwrap();

    assert_eq!(metrics.chain, Chain::Bitcoin);
    assert!(metrics.tps > 0.0);
}