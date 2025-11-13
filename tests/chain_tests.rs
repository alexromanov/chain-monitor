use chain_monitor::chains::Chain;

#[tokio::test]
async fn test_all_chain_types() {
    let chains = Chain::all();
    assert_eq!(chains.len(), 5);
    
    assert!(chains.contains(&Chain::Bitcoin));
    assert!(chains.contains(&Chain::Ethereum));
    assert!(chains.contains(&Chain::Cardano));
    assert!(chains.contains(&Chain::Solana));
    assert!(chains.contains(&Chain::Polkadot));
}

#[test]
fn test_chain_name_conversion() {
    use std::str::FromStr;
    
    assert_eq!(Chain::from_str("bitcoin").unwrap(), Chain::Bitcoin);
    assert_eq!(Chain::from_str("ethereum").unwrap(), Chain::Ethereum);
    assert_eq!(Chain::from_str("cardano").unwrap(), Chain::Cardano);
    assert_eq!(Chain::from_str("solana").unwrap(), Chain::Solana);
    assert_eq!(Chain::from_str("polkadot").unwrap(), Chain::Polkadot);
    
    assert!(Chain::from_str("invalid").is_err());
}