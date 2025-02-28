use ethers::{
    providers::{Provider, StreamExt, Ws},
    types::{Address, BlockNumber, Filter, Log, U256},
};
use tokio::sync::mpsc;
use std::sync::Arc;
use crate::events::{ChainEvent, EthereumEvent, EthEventType, EventData};

pub async fn start_listener(
    event_sender: mpsc::Sender<ChainEvent>,
    rpc_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(Provider::<Ws>::connect(rpc_url).await?);
    let block_number = provider.get_block_number().await?;

    // Set up event filters
    let deposit_filter = create_deposit_filter(block_number.as_u64());
    let withdrawal_filter = create_withdrawal_filter(block_number.as_u64());
    let price_filter = create_price_update_filter(block_number.as_u64());

    // Subscribe to events
    let deposit_stream = provider.subscribe_logs(&deposit_filter).await?;
    let withdrawal_stream = provider.subscribe_logs(&withdrawal_filter).await?;
    let price_stream = provider.subscribe_logs(&price_filter).await?;

    // Handle deposit events
    let deposit_handler = handle_events(
        deposit_stream,
        event_sender.clone(),
        EthEventType::Deposit,
        provider.clone(),
    );

    // Handle withdrawal events
    let withdrawal_handler = handle_events(
        withdrawal_stream,
        event_sender.clone(),
        EthEventType::Withdrawal,
        provider.clone(),
    );

    // Handle price update events
    let price_handler = handle_events(
        price_stream,
        event_sender.clone(),
        EthEventType::PriceUpdate,
        provider.clone(),
    );

    // Run all handlers concurrently
    tokio::try_join!(deposit_handler, withdrawal_handler, price_handler)?;

    Ok(())
}

fn create_deposit_filter(from_block: u64) -> Filter {
    Filter::new()
        .from_block(BlockNumber::Number(from_block.into()))
        .event("Deposit(address,uint256)")
}

fn create_withdrawal_filter(from_block: u64) -> Filter {
    Filter::new()
        .from_block(BlockNumber::Number(from_block.into()))
        .event("Withdrawal(address,uint256)")
}

fn create_price_update_filter(from_block: u64) -> Filter {
    Filter::new()
        .from_block(BlockNumber::Number(from_block.into()))
        .event("PriceUpdate(address,uint256)")
}

async fn handle_events(
    mut stream: ethers::providers::LogStream,
    event_sender: mpsc::Sender<ChainEvent>,
    event_type: EthEventType,
    provider: Arc<Provider<Ws>>,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(log) = stream.next().await {
        if let Ok(log) = log {
            let event = parse_event(log, event_type.clone(), provider.clone()).await?;
            event_sender.send(ChainEvent::EthereumEvent(event)).await?;
        }
    }
    Ok(())
}

async fn parse_event(
    log: Log,
    event_type: EthEventType,
    provider: Arc<Provider<Ws>>,
) -> Result<EthereumEvent, Box<dyn std::error::Error>> {
    let data = match event_type {
        EthEventType::Deposit | EthEventType::Withdrawal => {
            let token_address = format!("{:?}", log.address);
            let amount = U256::from_big_endian(&log.data);
            let from = format!("{:?}", log.topics[1]);

            EventData::TokenTransfer {
                token_address,
                from,
                to: token_address.clone(), // For deposits, 'to' is the contract
                amount: amount.to_string(),
            }
        }
        EthEventType::PriceUpdate => {
            let token_address = format!("{:?}", log.address);
            let price = U256::from_big_endian(&log.data);
            
            EventData::PriceUpdate {
                token_address,
                price: price.to_string(),
                timestamp: provider.get_block(log.block_number.unwrap())
                    .await?
                    .unwrap()
                    .timestamp
                    .as_u64(),
            }
        }
        EthEventType::LiquidityChange => {
            let pool_address = format!("{:?}", log.address);
            let amounts = parse_liquidity_amounts(&log.data);

            EventData::LiquidityUpdate {
                pool_address,
                token_a: format!("{:?}", log.topics[1]),
                token_b: format!("{:?}", log.topics[2]),
                amount_a: amounts.0.to_string(),
                amount_b: amounts.1.to_string(),
            }
        }
    };

    Ok(EthereumEvent {
        event_type,
        contract: log.address,
        transaction_hash: format!("{:?}", log.transaction_hash.unwrap()),
        block_number: log.block_number.unwrap().as_u64(),
        data,
    })
}

fn parse_liquidity_amounts(data: &[u8]) -> (U256, U256) {
    let amount_a = U256::from_big_endian(&data[..32]);
    let amount_b = U256::from_big_endian(&data[32..]);
    (amount_a, amount_b)
}