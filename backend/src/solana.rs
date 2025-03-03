use anchor_client::{Client, Cluster, Program};
use anchor_lang::AccountDeserialize;
use anyhow::{Result, anyhow};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcTransactionConfig, RpcSendTransactionConfig},
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionStatusMeta;
use solana_account_decoder::UiAccountData;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use std::str::FromStr;

/// Solana客户端接口类型
pub type SolanaClientType = Option<Arc<RpcClient>>;

/// 创建连接到Solana节点的RPC客户端
pub async fn create_client(rpc_url: &str) -> Result<SolanaClientType> {
    let client = Arc::new(RpcClient::new_with_commitment(
        rpc_url.to_string(),
        CommitmentConfig::confirmed(),
    ));
    
    // 尝试获取区块高度以验证连接
    match client.get_block_height() {
        Ok(_) => Ok(Some(client)),
        Err(e) => Err(anyhow!("Failed to connect to Solana node: {}", e)),
    }
}

/// 创建Anchor程序客户端，用于与合约交互
pub fn create_program_client<C>(
    rpc_url: &str,
    payer: &Keypair,
    program_id: &str,
) -> Result<Program<C>> {
    let program_id = Pubkey::from_str(program_id)
        .map_err(|e| anyhow!("Invalid program ID: {}", e))?;
    
    let cluster = Cluster::Custom(rpc_url.to_string(), rpc_url.to_string());
    let client = Client::new_with_options(
        cluster,
        payer.clone(),
        CommitmentConfig::confirmed(),
    );
    
    Ok(client.program(program_id))
}

/// 获取账户数据
pub async fn get_account<T: AccountDeserialize>(
    client: &RpcClient,
    address: &Pubkey,
) -> Result<T> {
    let account = client.get_account(address)
        .map_err(|e| anyhow!("Failed to fetch account data: {}", e))?;
    
    let mut data = account.data.as_slice();
    let account_data = T::try_deserialize(&mut data)
        .map_err(|e| anyhow!("Failed to deserialize account data: {}", e))?;
    
    Ok(account_data)
}

/// 查找PDA (Program Derived Address)
pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, program_id)
}

pub struct SolanaService {
    rpc_client: Arc<RpcClient>,
    config: Arc<crate::config::Config>,
    transaction_cache: Arc<RwLock<HashMap<String, TransactionStatus>>>,
}

#[derive(Debug, Clone)]
pub struct TransactionStatus {
    pub signature: String,
    pub status: String,
    pub confirmations: u64,
    pub timestamp: i64,
}

impl SolanaService {
    pub fn new(config: Arc<crate::config::Config>) -> Self {
        let commitment = match config.solana.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(), // default
        };

        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            config.solana.rpc_url.clone(),
            commitment,
        ));

        Self {
            rpc_client,
            config,
            transaction_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn submit_transaction(
        &self,
        transaction: Transaction,
        retry_count: u8,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let commitment_level = match self.config.solana.commitment.as_str() {
            "processed" => CommitmentLevel::Processed,
            "confirmed" => CommitmentLevel::Confirmed,
            "finalized" => CommitmentLevel::Finalized,
            _ => CommitmentLevel::Confirmed,
        };

        let config = RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(commitment_level),
            encoding: None,
            max_retries: Some(retry_count as usize),
            min_context_slot: None,
        };

        let signature = self.rpc_client.send_transaction_with_config(
            &transaction,
            config,
        )?;

        // Cache the transaction status
        let mut cache = self.transaction_cache.write().await;
        cache.insert(signature.to_string(), TransactionStatus {
            signature: signature.to_string(),
            status: "pending".to_string(),
            confirmations: 0,
            timestamp: chrono::Utc::now().timestamp(),
        });

        Ok(signature.to_string())
    }

    pub async fn monitor_transaction(
        &self,
        signature: &str,
    ) -> Result<TransactionStatus, Box<dyn std::error::Error>> {
        let commitment = match self.config.solana.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        };

        let config = RpcTransactionConfig {
            encoding: None,
            commitment: Some(commitment),
            max_supported_transaction_version: None,
        };

        let status = self.rpc_client.get_transaction_with_config(
            &signature.parse()?,
            config,
        )?;

        let transaction_status = TransactionStatus {
            signature: signature.to_string(),
            status: if status.transaction.meta.as_ref().and_then(|m| m.err).is_none() {
                "confirmed".to_string()
            } else {
                "failed".to_string()
            },
            confirmations: status.transaction.meta.as_ref().and_then(|m| m.confirmations).unwrap_or(0),
            timestamp: status.block_time.unwrap_or(0),
        };

        // Update cache
        let mut cache = self.transaction_cache.write().await;
        cache.insert(signature.to_string(), transaction_status.clone());

        Ok(transaction_status)
    }

    pub async fn get_token_balance(
        &self,
        token_account: &Pubkey,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let account = self.rpc_client.get_token_account_balance(token_account)?;
        // Convert the UI amount to u64 based on decimals
        Ok(account.amount.parse::<u64>().unwrap_or(0))
    }

    pub async fn start_transaction_monitor(&self) {
        let transaction_cache = self.transaction_cache.clone();
        let rpc_client = self.rpc_client.clone();
        let required_confirmations = self.config.security.required_confirmations;

        tokio::spawn(async move {
            loop {
                let mut cache = transaction_cache.write().await;
                let signatures: Vec<String> = cache.keys()
                    .filter(|sig| {
                        let status = cache.get(*sig).unwrap();
                        status.status == "pending" && 
                        status.confirmations < required_confirmations
                    })
                    .cloned()
                    .collect();

                for signature in signatures {
                    if let Ok(status) = rpc_client.get_signature_status(&signature.parse().unwrap()) {
                        if let Some(status) = status {
                            let transaction_status = cache.get_mut(&signature).unwrap();
                            transaction_status.status = if status.is_ok() {
                                "confirmed".to_string()
                            } else {
                                "failed".to_string()
                            };
                        }
                    }
                }

                // Clean up old entries
                cache.retain(|_, status| {
                    chrono::Utc::now().timestamp() - status.timestamp < 3600 // Keep for 1 hour
                });

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    }
}

#[derive(Debug)]
pub struct TransactionBuilder {
    transaction: Transaction,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            transaction: Transaction::new_with_payer(&[], None),
        }
    }

    // Add builder methods for common transaction types
    // ...
}