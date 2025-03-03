use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::time::Duration;
use std::str::FromStr;

/// 应用配置结构体，存储所有服务器配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub tokenomics: TokenomicsConfig,
    pub security: SecurityConfig,
    pub analytics: AnalyticsConfig,
    pub solana: SolanaConfig,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout: u64,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TokenomicsConfig {
    pub mining_reward_per_block: u64,
    pub reward_halving_period: u64,
    pub lp_fee_share: f64,
    pub buyback_share: f64,
    pub min_buyback_amount: u64,
    pub max_buyback_interval: u64,
    pub treasury_address: String,
    pub token_mint_address: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct SecurityConfig {
    pub rate_limits: RateLimitConfig,
    pub max_transaction_size: usize,
    pub required_confirmations: u64,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RateLimitConfig {
    pub trading: RateLimit,
    pub analytics: RateLimit,
    pub default: RateLimit,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RateLimit {
    pub requests: u32,
    pub window_seconds: u64,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct AnalyticsConfig {
    pub metrics_update_interval: u64,
    pub max_historical_data_days: u32,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
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

    // Add a function to create a default configuration for development/testing
    pub fn default() -> Self {
        Config {
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                cors_origins: vec!["*".to_string()],
                request_timeout: 30,
            },
            tokenomics: TokenomicsConfig {
                mining_reward_per_block: 100,
                reward_halving_period: 2_100_000,
                lp_fee_share: 0.7,
                buyback_share: 0.3,
                min_buyback_amount: 1000,
                max_buyback_interval: 86400,
                treasury_address: "11111111111111111111111111111111".to_string(),
                token_mint_address: "11111111111111111111111111111111".to_string(),
            },
            security: SecurityConfig {
                rate_limits: RateLimitConfig {
                    trading: RateLimit {
                        requests: 100,
                        window_seconds: 60,
                    },
                    analytics: RateLimit {
                        requests: 300,
                        window_seconds: 60,
                    },
                    default: RateLimit {
                        requests: 200,
                        window_seconds: 60,
                    },
                },
                max_transaction_size: 1024,
                required_confirmations: 1,
            },
            analytics: AnalyticsConfig {
                metrics_update_interval: 300,
                max_historical_data_days: 30,
            },
            solana: SolanaConfig {
                rpc_url: "https://api.devnet.solana.com".to_string(),
                ws_url: "wss://api.devnet.solana.com".to_string(),
                commitment: "confirmed".to_string(),
            },
        }
    }
}

impl RateLimit {
    pub fn to_duration(&self) -> Duration {
        Duration::from_secs(self.window_seconds)
    }
}