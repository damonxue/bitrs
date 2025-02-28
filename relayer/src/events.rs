use serde::{Serialize, Deserialize};
use ethers::types::{Address as EthAddress, U256};
use solana_sdk::pubkey::Pubkey;
use tokio::sync::mpsc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Insufficient confirmations")]
    InsufficientConfirmations,
    #[error("Invalid event data: {0}")]
    InvalidEventData(String),
    #[error("Channel send error: {0}")]
    ChannelError(String),
    #[error("Event validation failed: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainEvent {
    EthereumEvent(EthereumEvent),
    SolanaEvent(SolanaEvent),
    BscEvent(BscEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumEvent {
    pub event_type: EthEventType,
    pub contract: EthAddress,
    pub transaction_hash: String,
    pub block_number: u64,
    pub data: EventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaEvent {
    pub event_type: SolEventType,
    pub program_id: Pubkey,
    pub signature: String,
    pub slot: u64,
    pub data: EventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BscEvent {
    pub event_type: BscEventType,
    pub contract: EthAddress,
    pub transaction_hash: String,
    pub block_number: u64,
    pub data: EventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EthEventType {
    Deposit,
    Withdrawal,
    PriceUpdate,
    LiquidityChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolEventType {
    BridgeDeposit,
    BridgeWithdrawal,
    OrderbookUpdate,
    PoolUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BscEventType {
    Deposit,
    Withdrawal,
    PriceUpdate,
    LiquidityChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    TokenTransfer {
        token_address: String,
        from: String,
        to: String,
        amount: String,
    },
    PriceUpdate {
        token_address: String,
        price: String,
        timestamp: u64,
    },
    LiquidityUpdate {
        pool_address: String,
        token_a: String,
        token_b: String,
        amount_a: String,
        amount_b: String,
    },
}

pub trait EventValidator {
    fn validate_confirmations(&self, current_block: u64, event_block: u64, required_confirmations: u64) -> Result<(), EventError> {
        if current_block < event_block + required_confirmations {
            return Err(EventError::InsufficientConfirmations);
        }
        Ok(())
    }
    
    fn validate_event_data(&self, data: &EventData) -> Result<(), EventError>;
}

impl EventValidator for EthereumEvent {
    fn validate_event_data(&self, data: &EventData) -> Result<(), EventError> {
        match data {
            EventData::TokenTransfer { amount, .. } => {
                if amount.is_empty() {
                    return Err(EventError::InvalidEventData("Amount cannot be empty".into()));
                }
            }
            EventData::PriceUpdate { price, timestamp, .. } => {
                if price.is_empty() || *timestamp == 0 {
                    return Err(EventError::InvalidEventData("Invalid price update data".into()));
                }
            }
            // ... handle other event types
        }
        Ok(())
    }
}

// Similar implementations for SolanaEvent and BscEvent...

#[derive(Debug)]
pub struct EventProcessor {
    event_sender: mpsc::Sender<ChainEvent>,
    ethereum_confirmations: u64,
    bsc_confirmations: u64,
    solana_confirmations: u64,
}

impl EventProcessor {
    pub fn new(
        event_sender: mpsc::Sender<ChainEvent>,
        ethereum_confirmations: u64,
        bsc_confirmations: u64,
        solana_confirmations: u64,
    ) -> Self {
        Self {
            event_sender,
            ethereum_confirmations,
            bsc_confirmations,
            solana_confirmations,
        }
    }

    async fn validate_and_process<T: EventValidator>(&self, event: &T, data: &EventData) -> Result<(), EventError> {
        // First validate the event data
        event.validate_event_data(data)?;
        
        // Additional processing logic can be added here
        Ok(())
    }

    pub async fn process_ethereum_event(&self, event: EthereumEvent) -> Result<(), EventError> {
        self.validate_and_process(&event, &event.data).await?;
        
        self.event_sender
            .send(ChainEvent::EthereumEvent(event))
            .await
            .map_err(|e| EventError::ChannelError(e.to_string()))
    }

    pub async fn process_solana_event(&self, event: SolanaEvent) -> Result<(), EventError> {
        self.validate_and_process(&event, &event.data).await?;
        
        self.event_sender
            .send(ChainEvent::SolanaEvent(event))
            .await
            .map_err(|e| EventError::ChannelError(e.to_string()))
    }

    pub async fn process_bsc_event(&self, event: BscEvent) -> Result<(), EventError> {
        self.validate_and_process(&event, &event.data).await?;
        
        self.event_sender
            .send(ChainEvent::BscEvent(event))
            .await
            .map_err(|e| EventError::ChannelError(e.to_string()))
    }
}

// Add metrics collection
#[derive(Debug, Default)]
pub struct EventMetrics {
    pub processed_events: std::sync::atomic::AtomicU64,
    pub failed_events: std::sync::atomic::AtomicU64,
    pub last_processed_block: std::sync::atomic::AtomicU64,
}

// Add retry mechanism
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
}