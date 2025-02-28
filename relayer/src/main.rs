use tokio;
use std::sync::Arc;
use tokio::sync::mpsc;
use ethers::types::Address as EthAddress;
use solana_sdk::pubkey::Pubkey;

mod events;
mod validators;
mod chains;

use events::{ChainEvent, EventProcessor};
use validators::MessageValidator;
use chains::{ethereum, solana, mod};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration
    let config = load_config()?;

    // Set up channels for event processing
    let (event_sender, mut event_receiver) = mpsc::channel::<ChainEvent>(100);

    // Initialize validators
    let message_validator = Arc::new(MessageValidator::new(
        config.ethereum.required_confirmations,
        config.solana.required_confirmations,
        config.bsc.required_confirmations,
        config.ethereum.trusted_contracts,
        config.solana.trusted_programs,
        config.bsc.trusted_contracts,
    ));

    // Initialize event processor
    let event_processor = EventProcessor::new(
        event_sender.clone(),
        config.ethereum.required_confirmations,
        config.bsc.required_confirmations,
        config.solana.required_confirmations,
    );

    // Start chain listeners
    let eth_listener = ethereum::start_listener(event_sender.clone(), config.ethereum.rpc_url);
    let sol_listener = solana::start_listener(event_sender.clone(), config.solana.rpc_url);
    let bsc_listener = chains::bsc::start_listener(event_sender.clone(), config.bsc.rpc_url);

    // Main event processing loop
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            let validator = message_validator.clone();
            
            tokio::spawn(async move {
                if let Ok(is_valid) = validator.validate_event(&event).await {
                    if is_valid {
                        // Process validated event
                        process_validated_event(event).await;
                    }
                }
            });
        }
    });

    // Keep the main thread alive
    tokio::signal::ctrl_c().await?;
    println!("Shutting down relayer...");
    Ok(())
}

async fn process_validated_event(event: ChainEvent) {
    match event {
        ChainEvent::EthereumEvent(eth_event) => {
            // Handle Ethereum event
            println!("Processing Ethereum event: {:?}", eth_event);
        }
        ChainEvent::SolanaEvent(sol_event) => {
            // Handle Solana event
            println!("Processing Solana event: {:?}", sol_event);
        }
        ChainEvent::BscEvent(bsc_event) => {
            // Handle BSC event
            println!("Processing BSC event: {:?}", bsc_event);
        }
    }
}

#[derive(Debug)]
struct Config {
    ethereum: ChainConfig<EthAddress>,
    solana: ChainConfig<Pubkey>,
    bsc: ChainConfig<EthAddress>,
}

#[derive(Debug)]
struct ChainConfig<T> {
    rpc_url: String,
    required_confirmations: u64,
    trusted_contracts: Vec<T>,
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Load configuration from environment or config file
    Ok(Config {
        ethereum: ChainConfig {
            rpc_url: std::env::var("ETH_RPC_URL")?,
            required_confirmations: 12,
            trusted_contracts: vec![],
        },
        solana: ChainConfig {
            rpc_url: std::env::var("SOL_RPC_URL")?,
            required_confirmations: 32,
            trusted_contracts: vec![],
        },
        bsc: ChainConfig {
            rpc_url: std::env::var("BSC_RPC_URL")?,
            required_confirmations: 15,
            trusted_contracts: vec![],
        },
    })
}