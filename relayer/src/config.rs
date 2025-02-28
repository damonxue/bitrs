use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub solana: SolanaConfig,
    pub ethereum: EthereumConfig,
    pub validators: ValidatorConfig,
}

#[derive(Debug, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub bridge_program_id: String,
    pub ws_url: String,
}

#[derive(Debug, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub bridge_contract: String,
    pub ws_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ValidatorConfig {
    pub required_signatures: u8,
    pub private_key: String,
    pub validator_addresses: Vec<String>,
}

impl Config {
    pub fn load() -> Self {
        let config_str = fs::read_to_string("config.toml")
            .expect("Failed to read config file");
        toml::from_str(&config_str).expect("Failed to parse config")
    }
}