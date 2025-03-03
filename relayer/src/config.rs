use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use ethers::types::Address as EthAddress;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// 中继器配置结构体，存储所有链配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ethereum: ChainConfig,
    pub solana: SolanaChainConfig,
    pub bsc: ChainConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub required_confirmations: u64,
    pub trusted_contracts: Vec<String>,
    pub gas_price_multiplier: f64,
    pub max_retries: u32,
    pub retry_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaChainConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub required_confirmations: u64,
    pub trusted_programs: Vec<String>,
    pub commitment: String,
    pub max_retries: u32,
    pub retry_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub log_level: String,
    pub metrics_port: u16,
    pub health_check_port: u16,
    pub event_queue_size: usize,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = std::env::var("RELAYER_CONFIG")
            .unwrap_or_else(|_| "config/relayer.toml".to_string());
        Self::load(config_path)
    }
}

impl ChainConfig {
    pub fn get_trusted_eth_addresses(&self) -> Result<Vec<EthAddress>, Box<dyn std::error::Error>> {
        let mut addresses = Vec::new();
        for address_str in &self.trusted_contracts {
            let address = EthAddress::from_str(address_str)?;
            addresses.push(address);
        }
        Ok(addresses)
    }
}

impl SolanaChainConfig {
    pub fn get_trusted_program_ids(&self) -> Result<Vec<Pubkey>, Box<dyn std::error::Error>> {
        let mut pubkeys = Vec::new();
        for pubkey_str in &self.trusted_programs {
            let pubkey = Pubkey::from_str(pubkey_str)?;
            pubkeys.push(pubkey);
        }
        Ok(pubkeys)
    }
}