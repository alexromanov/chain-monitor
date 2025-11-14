use chain_monitor::tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_url = std::env::var("API_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    tui::run(api_url).await?;
    
    Ok(())
}