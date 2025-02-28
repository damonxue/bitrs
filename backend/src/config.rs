use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::time::Duration;

/// 应用配置结构体，存储所有服务器配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub tokenomics: TokenomicsConfig,
    pub security: SecurityConfig,
    pub analytics: AnalyticsConfig,
    pub solana: SolanaConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TokenomicsConfig {
    pub mining_reward_per_block: u64,
    pub reward_halving_period: u64,
    pub lp_fee_share: f64,
    pub buyback_share: f64,
    pub min_buyback_amount: u64,
    pub max_buyback_interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub rate_limits: RateLimitConfig,
    pub max_transaction_size: usize,
    pub required_confirmations: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub trading: RateLimit,
    pub analytics: RateLimit,
    pub default: RateLimit,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimit {
    pub requests: u32,
    pub window_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnalyticsConfig {
    pub metrics_update_interval: u64,
    pub max_historical_data_days: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub commitment: String,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = std::env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config/default.toml".to_string());
        
        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        
        Ok(config)
    }
}

impl RateLimit {
    pub fn to_duration(&self) -> Duration {
        Duration::from_secs(self.window_seconds)
    }
}