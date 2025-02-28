use ethers::types::{Address as EthAddress, U256, Transaction as EthTransaction};
use solana_sdk::{
    transaction::Transaction as SolTransaction,
    signature::Signature,
    pubkey::Pubkey,
};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::events::{ChainEvent, EthereumEvent, SolanaEvent, BscEvent};

pub struct ValidatorSet {
    validators: HashMap<Pubkey, Validator>,
    required_signatures: u8,
}

struct Validator {
    address: Pubkey,
    eth_address: String,
    signatures: Arc<Mutex<HashMap<u64, Vec<u8>>>>, // sequence -> signature
}

impl ValidatorSet {
    pub fn new(required_signatures: u8) -> Self {
        Self {
            validators: HashMap::new(),
            required_signatures,
        }
    }

    pub fn add_validator(&mut self, solana_address: Pubkey, eth_address: String) {
        self.validators.insert(
            solana_address,
            Validator {
                address: solana_address,
                eth_address,
                signatures: Arc::new(Mutex::new(HashMap::new())),
            },
        );
    }

    pub fn collect_signature(
        &self,
        validator: &Pubkey,
        sequence: u64,
        signature: Vec<u8>,
    ) -> bool {
        if let Some(v) = self.validators.get(validator) {
            let mut sigs = v.signatures.lock().unwrap();
            sigs.insert(sequence, signature);
            return true;
        }
        false
    }

    pub fn has_enough_signatures(&self, sequence: u64) -> bool {
        let count = self.validators
            .values()
            .filter(|v| v.signatures.lock().unwrap().contains_key(&sequence))
            .count();
        count >= self.required_signatures as usize
    }

    pub fn get_signatures(&self, sequence: u64) -> Vec<(String, Vec<u8>)> {
        self.validators
            .values()
            .filter_map(|v| {
                v.signatures
                    .lock()
                    .unwrap()
                    .get(&sequence)
                    .map(|sig| (v.eth_address.clone(), sig.clone()))
            })
            .collect()
    }

    pub fn clear_signatures(&self, sequence: u64) {
        for validator in self.validators.values() {
            validator.signatures.lock().unwrap().remove(&sequence);
        }
    }
}

pub fn load_validators() -> Arc<ValidatorSet> {
    let config = crate::config::Config::load();
    let mut validator_set = ValidatorSet::new(config.validators.required_signatures);

    for address in config.validators.validator_addresses {
        // Convert address strings to Pubkey and add to validator set
        if let Ok(pubkey) = address.parse::<Pubkey>() {
            validator_set.add_validator(pubkey, address);
        }
    }

    Arc::new(validator_set)
}

pub struct MessageValidator {
    ethereum_validator: EthereumValidator,
    solana_validator: SolanaValidator,
    bsc_validator: BscValidator,
    processed_messages: Arc<RwLock<HashMap<String, MessageStatus>>>,
}

#[derive(Debug, Clone)]
pub struct MessageStatus {
    pub source_chain: String,
    pub target_chain: String,
    pub status: ValidationStatus,
    pub timestamp: i64,
    pub confirmations: u64,
}

#[derive(Debug, Clone)]
pub enum ValidationStatus {
    Pending,
    Confirmed,
    Failed(String),
}

pub struct EthereumValidator {
    required_confirmations: u64,
    trusted_contracts: Vec<EthAddress>,
}

pub struct SolanaValidator {
    required_confirmations: u64,
    trusted_programs: Vec<Pubkey>,
}

pub struct BscValidator {
    required_confirmations: u64,
    trusted_contracts: Vec<EthAddress>,
}

impl MessageValidator {
    pub fn new(
        eth_confirmations: u64,
        sol_confirmations: u64,
        bsc_confirmations: u64,
        eth_contracts: Vec<EthAddress>,
        sol_programs: Vec<Pubkey>,
        bsc_contracts: Vec<EthAddress>,
    ) -> Self {
        Self {
            ethereum_validator: EthereumValidator::new(eth_confirmations, eth_contracts),
            solana_validator: SolanaValidator::new(sol_confirmations, sol_programs),
            bsc_validator: BscValidator::new(bsc_confirmations, bsc_contracts),
            processed_messages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn validate_event(&self, event: &ChainEvent) -> Result<bool, String> {
        match event {
            ChainEvent::EthereumEvent(eth_event) => {
                self.ethereum_validator.validate_event(eth_event).await
            }
            ChainEvent::SolanaEvent(sol_event) => {
                self.solana_validator.validate_event(sol_event).await
            }
            ChainEvent::BscEvent(bsc_event) => {
                self.bsc_validator.validate_event(bsc_event).await
            }
        }
    }

    pub async fn track_message(&self, message_id: String, status: MessageStatus) {
        let mut messages = self.processed_messages.write().await;
        messages.insert(message_id, status);
    }

    pub async fn verify_message_uniqueness(&self, message_id: &str) -> bool {
        let messages = self.processed_messages.read().await;
        !messages.contains_key(message_id)
    }
}

impl EthereumValidator {
    pub fn new(required_confirmations: u64, trusted_contracts: Vec<EthAddress>) -> Self {
        Self {
            required_confirmations,
            trusted_contracts,
        }
    }

    pub async fn validate_event(&self, event: &EthereumEvent) -> Result<bool, String> {
        // Verify contract is trusted
        if !self.trusted_contracts.contains(&event.contract) {
            return Err("Untrusted contract address".to_string());
        }

        // Additional validation logic specific to Ethereum
        // - Verify transaction receipt
        // - Check event logs
        // - Validate parameters
        Ok(true)
    }

    pub async fn validate_transaction(&self, tx: &EthTransaction) -> Result<bool, String> {
        // Implement Ethereum-specific transaction validation
        Ok(true)
    }
}

impl SolanaValidator {
    pub fn new(required_confirmations: u64, trusted_programs: Vec<Pubkey>) -> Self {
        Self {
            required_confirmations,
            trusted_programs,
        }
    }

    pub async fn validate_event(&self, event: &SolanaEvent) -> Result<bool, String> {
        // Verify program is trusted
        if !self.trusted_programs.contains(&event.program_id) {
            return Err("Untrusted program ID".to_string());
        }

        // Additional validation logic specific to Solana
        // - Verify transaction signature
        // - Check program logs
        // - Validate account state changes
        Ok(true)
    }

    pub async fn validate_transaction(&self, tx: &SolTransaction) -> Result<bool, String> {
        // Implement Solana-specific transaction validation
        Ok(true)
    }
}

impl BscValidator {
    pub fn new(required_confirmations: u64, trusted_contracts: Vec<EthAddress>) -> Self {
        Self {
            required_confirmations,
            trusted_contracts,
        }
    }

    pub async fn validate_event(&self, event: &BscEvent) -> Result<bool, String> {
        // Verify contract is trusted
        if !self.trusted_contracts.contains(&event.contract) {
            return Err("Untrusted contract address".to_string());
        }

        // Additional validation logic specific to BSC
        // - Verify transaction receipt
        // - Check event logs
        // - Validate parameters
        Ok(true)
    }

    pub async fn validate_transaction(&self, tx: &EthTransaction) -> Result<bool, String> {
        // Implement BSC-specific transaction validation
        Ok(true)
    }
}