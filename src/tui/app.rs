use crate::chains::{Block, Chain, ChainMetrics};
use crate::Result;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ChainStatus {
    pub chain: Chain,
    pub latest_block: Option<Block>,
    pub metrics: Option<ChainMetrics>,
    pub status: ConnectionStatus,
    pub last_update: Instant,
    pub block_history: VecDeque<Block>,
    pub tps_history: VecDeque<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error(String),
}

impl ChainStatus {
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            latest_block: None,
            metrics: None,
            status: ConnectionStatus::Disconnected,
            last_update: Instant::now(),
            block_history: VecDeque::with_capacity(100),
            tps_history: VecDeque::with_capacity(50),
        }
    }
    
    pub fn update_block(&mut self, block: Block) {
        self.latest_block = Some(block.clone());
        self.block_history.push_back(block);
        if self.block_history.len() > 100 {
            self.block_history.pop_front();
        }
        self.last_update = Instant::now();
        self.status = ConnectionStatus::Connected;
    }
    
    pub fn update_metrics(&mut self, metrics: ChainMetrics) {
        self.tps_history.push_back(metrics.tps as u64);
        if self.tps_history.len() > 50 {
            self.tps_history.pop_front();
        }
        self.metrics = Some(metrics);
        self.last_update = Instant::now();
    }
    
    pub fn health_indicator(&self) -> &str {
        match &self.status {
            ConnectionStatus::Connected => {
                if self.last_update.elapsed() < Duration::from_secs(30) {
                    "🟢"
                } else {
                    "🟡"
                }
            }
            ConnectionStatus::Disconnected => "🔴",
            ConnectionStatus::Error(_) => "❌",
        }
    }
}

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

pub struct App {
    pub api_url: String,
    pub chains: HashMap<Chain, ChainStatus>,
    pub selected_chain_index: usize,
    pub logs: VecDeque<LogEntry>,
    pub client: reqwest::Client,
    pub should_quit: bool,
    pub current_tab: Tab,
    pub last_update: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Dashboard,
    ChainDetails,
    Logs,
}

impl App {
    pub async fn new(api_url: String) -> Result<Self> {
        let client = reqwest::Client::new();
        
        let mut chains = HashMap::new();
        for chain in Chain::all() {
            chains.insert(chain, ChainStatus::new(chain));
        }
        
        let mut app = Self {
            api_url,
            chains,
            selected_chain_index: 0,
            logs: VecDeque::with_capacity(1000),
            client,
            should_quit: false,
            current_tab: Tab::Dashboard,
            last_update: Instant::now(),
        };
        
        app.add_log(LogLevel::Info, "Chain Monitor started");
        
        app.fetch_all_data().await?;
        
        Ok(app)
    }
    
    pub async fn update(&mut self) -> Result<()> {
        if self.last_update.elapsed() >= Duration::from_secs(5) {
            self.fetch_all_data().await?;
            self.last_update = Instant::now();
        }
        
        Ok(())
    }
    
    async fn fetch_all_data(&mut self) -> Result<()> {
        for chain in Chain::all() {
            if let Ok(block) = self.fetch_latest_block(chain).await {
                if let Some(status) = self.chains.get_mut(&chain) {
                    status.update_block(block);
                }
            }
            
            if let Ok(metrics) = self.fetch_metrics(chain).await {
                if let Some(status) = self.chains.get_mut(&chain) {
                    status.update_metrics(metrics);
                }
            }
        }
        
        Ok(())
    }
    
    async fn fetch_latest_block(&self, chain: Chain) -> Result<Block> {
        let url = format!("{}/api/chains/{}/latest", self.api_url, chain.name());
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::Error::Api(format!("Failed to fetch block: {}", e)))?;
        
        let block = response
            .json::<Block>()
            .await
            .map_err(|e| crate::Error::Api(format!("Failed to parse block: {}", e)))?;
        
        Ok(block)
    }
    
    async fn fetch_metrics(&self, chain: Chain) -> Result<ChainMetrics> {
        let url = format!("{}/api/chains/{}/metrics", self.api_url, chain.name());
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::Error::Api(format!("Failed to fetch metrics: {}", e)))?;
        
        let metrics = response
            .json::<ChainMetrics>()
            .await
            .map_err(|e| crate::Error::Api(format!("Failed to parse metrics: {}", e)))?;
        
        Ok(metrics)
    }
    
    pub fn add_log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.logs.push_back(LogEntry {
            timestamp: Instant::now(),
            level,
            message: message.into(),
        });
        
        if self.logs.len() > 1000 {
            self.logs.pop_front();
        }
    }
    
    pub fn get_selected_chain(&self) -> Option<&ChainStatus> {
        let chains: Vec<_> = Chain::all();
        chains.get(self.selected_chain_index)
            .and_then(|chain| self.chains.get(chain))
    }
    
    pub fn next_chain(&mut self) {
        self.selected_chain_index = (self.selected_chain_index + 1) % Chain::all().len();
    }
    
    pub fn previous_chain(&mut self) {
        if self.selected_chain_index == 0 {
            self.selected_chain_index = Chain::all().len() - 1;
        } else {
            self.selected_chain_index -= 1;
        }
    }
    
    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Dashboard => Tab::ChainDetails,
            Tab::ChainDetails => Tab::Logs,
            Tab::Logs => Tab::Dashboard,
        };
    }
    
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}