use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcProgramAccountsConfig, RpcTransactionConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    transaction::Transaction,
};
use tokio::sync::mpsc;
use std::str::FromStr;
use std::sync::Arc;
use crate::events::{ChainEvent, SolanaEvent, SolEventType, EventData};
use crate::validators::ValidatorSet;

pub async fn start_listener(
    event_sender: mpsc::Sender<ChainEvent>,
    rpc_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    );

    let program_id = Pubkey::from_str("YOUR_PROGRAM_ID")?;
    let mut slot = client.get_slot()?;

    loop {
        let new_slot = client.get_slot()?;
        if new_slot > slot {
            // Process new blocks
            for current_slot in slot..=new_slot {
                process_slot(&client, current_slot, &program_id, &event_sender).await?;
            }
            slot = new_slot;
        }

        // Process program accounts for any state changes
        process_program_accounts(&client, &program_id, &event_sender).await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn process_slot(
    client: &RpcClient,
    slot: u64,
    program_id: &Pubkey,
    event_sender: &mpsc::Sender<ChainEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let block = client.get_block_with_encoding(
        slot,
        solana_transaction_status::UiTransactionEncoding::Json,
    )?;

    for tx in block.transactions {
        if let Some(meta) = tx.meta {
            if let Some(log_messages) = meta.log_messages {
                if log_messages.iter().any(|msg| msg.contains(&program_id.to_string())) {
                    let event = parse_transaction_logs(
                        &log_messages,
                        slot,
                        &tx.transaction.signatures[0],
                        program_id,
                    )?;
                    
                    if let Some(event) = event {
                        event_sender.send(ChainEvent::SolanaEvent(event)).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn process_program_accounts(
    client: &RpcClient,
    program_id: &Pubkey,
    event_sender: &mpsc::Sender<ChainEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![
            RpcFilterType::Memcmp(Memcmp {
                offset: 0,
                bytes: program_id.to_string(),
                encoding: None,
            }),
        ]),
        account_config: solana_client::rpc_config::RpcAccountInfoConfig {
            encoding: None,
            data_slice: None,
            commitment: Some(CommitmentConfig::confirmed()),
        },
        with_context: Some(true),
    };

    let accounts = client.get_program_accounts_with_config(program_id, config)?;

    for (pubkey, account) in accounts {
        if let Some(event) = parse_account_data(&pubkey, &account.data, program_id)? {
            event_sender.send(ChainEvent::SolanaEvent(event)).await?;
        }
    }

    Ok(())
}

fn parse_transaction_logs(
    logs: &[String],
    slot: u64,
    signature: &str,
    program_id: &Pubkey,
) -> Result<Option<SolanaEvent>, Box<dyn std::error::Error>> {
    for log in logs {
        if let Some(event) = parse_log_message(log, slot, signature, program_id)? {
            return Ok(Some(event));
        }
    }
    Ok(None)
}

fn parse_log_message(
    log: &str,
    slot: u64,
    signature: &str,
    program_id: &Pubkey,
) -> Result<Option<SolanaEvent>, Box<dyn std::error::Error>> {
    // Example log parsing for bridge events
    if log.contains("bridge_deposit") {
        let parts: Vec<&str> = log.split(':').collect();
        if parts.len() >= 2 {
            return Ok(Some(SolanaEvent {
                event_type: SolEventType::BridgeDeposit,
                program_id: *program_id,
                signature: signature.to_string(),
                slot,
                data: EventData::TokenTransfer {
                    token_address: parts[1].trim().to_string(),
                    from: parts[2].trim().to_string(),
                    to: parts[3].trim().to_string(),
                    amount: parts[4].trim().to_string(),
                },
            }));
        }
    }

    // Add more event type parsing as needed

    Ok(None)
}

fn parse_account_data(
    pubkey: &Pubkey,
    data: &[u8],
    program_id: &Pubkey,
) -> Result<Option<SolanaEvent>, Box<dyn std::error::Error>> {
    if data.len() < 8 {
        return Ok(None);
    }

    // Example account data parsing
    // This would be replaced with actual account state deserialization
    match data[0] {
        0 => Ok(Some(SolanaEvent {
            event_type: SolEventType::PoolUpdate,
            program_id: *program_id,
            signature: pubkey.to_string(),
            slot: 0, // Current slot would be fetched from client
            data: EventData::LiquidityUpdate {
                pool_address: pubkey.to_string(),
                token_a: "".to_string(),
                token_b: "".to_string(),
                amount_a: "0".to_string(),
                amount_b: "0".to_string(),
            },
        })),
        _ => Ok(None),
    }
}