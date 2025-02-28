use ethers::{
    providers::{Provider, StreamExt, Ws},
    types::{Address, BlockNumber, Filter, Log, U256},
};
use tokio::sync::mpsc;
use std::sync::Arc;
use crate::events::{ChainEvent, BscEvent, BscEventType, EventData};

const BSC_BLOCK_TIME: u64 = 3; // BSC has 3 second block times
const SAFE_CONFIRMATIONS: u64 = 15; // Number of confirmations needed for BSC

pub async fn start_listener(
    event_sender: mpsc::Sender<ChainEvent>,
    rpc_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(Provider::<Ws>::connect(rpc_url).await?);
    
    // Get current block for initialization
    let block_number = provider.get_block_number().await?;
    let start_block = block_number.as_u64().saturating_sub(SAFE_CONFIRMATIONS);

    // Set up event filters specific to BSC
    let bridge_filter = create_bridge_filter(start_block);
    let pool_filter = create_pool_filter(start_block);
    let price_filter = create_price_filter(start_block);

    // Subscribe to events
    let bridge_stream = provider.subscribe_logs(&bridge_filter).await?;
    let pool_stream = provider.subscribe_logs(&pool_filter).await?;
    let price_stream = provider.subscribe_logs(&price_filter).await?;

    // Handle bridge events
    let bridge_handler = handle_events(
        bridge_stream,
        event_sender.clone(),
        BscEventType::Deposit,
        provider.clone(),
    );

    // Handle pool events
    let pool_handler = handle_events(
        pool_stream,
        event_sender.clone(),
        BscEventType::LiquidityChange,
        provider.clone(),
    );

    // Handle price events
    let price_handler = handle_events(
        price_stream,
        event_sender.clone(),
        BscEventType::PriceUpdate,
        provider.clone(),
    );

    // Monitor block confirmations
    let confirmation_monitor = monitor_confirmations(provider.clone());

    // Run all handlers concurrently
    tokio::try_join!(
        bridge_handler,
        pool_handler,
        price_handler,
        confirmation_monitor
    )?;

    Ok(())
}

fn create_bridge_filter(from_block: u64) -> Filter {
    Filter::new()
        .from_block(BlockNumber::Number(from_block.into()))
        .address(vec![
            // Add your bridge contract addresses here
        ])
        .event("BridgeEvent(address,uint256,bytes32)")
}

fn create_pool_filter(from_block: u64) -> Filter {
    Filter::new()
        .from_block(BlockNumber::Number(from_block.into()))
        .address(vec![
            // Add your pool contract addresses here
        ])
        .event("PoolUpdate(address,uint256,uint256)")
}

fn create_price_filter(from_block: u64) -> Filter {
    Filter::new()
        .from_block(BlockNumber::Number(from_block.into()))
        .address(vec![
            // Add your price oracle addresses here
        ])
        .event("PriceUpdate(address,uint256)")
}

async fn handle_events(
    mut stream: ethers::providers::LogStream,
    event_sender: mpsc::Sender<ChainEvent>,
    event_type: BscEventType,
    provider: Arc<Provider<Ws>>,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(log) = stream.next().await {
        if let Ok(log) = log {
            // Wait for safe confirmations
            let current_block = provider.get_block_number().await?;
            let log_block = log.block_number.unwrap();
            
            if current_block.as_u64() >= log_block.as_u64() + SAFE_CONFIRMATIONS {
                let event = parse_event(log, event_type.clone(), provider.clone()).await?;
                event_sender.send(ChainEvent::BscEvent(event)).await?;
            }
        }
    }
    Ok(())
}

async fn parse_event(
    log: Log,
    event_type: BscEventType,
    provider: Arc<Provider<Ws>>,
) -> Result<BscEvent, Box<dyn std::error::Error>> {
    let data = match event_type {
        BscEventType::Deposit => {
            let token_address = format!("{:?}", log.address);
            let amount = U256::from_big_endian(&log.data[..32]);
            let bridge_id = log.data[32..].to_vec();

            EventData::TokenTransfer {
                token_address,
                from: format!("{:?}", log.topics[1]),
                to: "bridge".to_string(),
                amount: amount.to_string(),
            }
        }
        BscEventType::Withdrawal => {
            let token_address = format!("{:?}", log.address);
            let amount = U256::from_big_endian(&log.data[..32]);
            
            EventData::TokenTransfer {
                token_address,
                from: "bridge".to_string(),
                to: format!("{:?}", log.topics[1]),
                amount: amount.to_string(),
            }
        }
        BscEventType::PriceUpdate => {
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
        BscEventType::LiquidityChange => {
            let pool_address = format!("{:?}", log.address);
            let (amount_a, amount_b) = parse_liquidity_amounts(&log.data);

            EventData::LiquidityUpdate {
                pool_address,
                token_a: format!("{:?}", log.topics[1]),
                token_b: format!("{:?}", log.topics[2]),
                amount_a: amount_a.to_string(),
                amount_b: amount_b.to_string(),
            }
        }
    };

    Ok(BscEvent {
        event_type,
        contract: log.address,
        transaction_hash: format!("{:?}", log.transaction_hash.unwrap()),
        block_number: log.block_number.unwrap().as_u64(),
        data,
    })
}

async fn monitor_confirmations(
    provider: Arc<Provider<Ws>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_monitored = provider.get_block_number().await?.as_u64();
    
    loop {
        let current = provider.get_block_number().await?.as_u64();
        if current > last_monitored + SAFE_CONFIRMATIONS {
            // Process any reorgs or missed events
            for block_number in last_monitored..=current - SAFE_CONFIRMATIONS {
                check_block_finality(provider.clone(), block_number).await?;
            }
            last_monitored = current - SAFE_CONFIRMATIONS;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(BSC_BLOCK_TIME)).await;
    }
}

async fn check_block_finality(
    provider: Arc<Provider<Ws>>,
    block_number: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let block = provider.get_block(block_number.into()).await?;
    
    if let Some(block) = block {
        // Check for any reorgs and handle them
        // This would involve comparing with previously processed blocks
        if block.hash.is_some() {
            // Block is final, could trigger reprocessing if hash changed
        }
    }
    
    Ok(())
}

fn parse_liquidity_amounts(data: &[u8]) -> (U256, U256) {
    let amount_a = U256::from_big_endian(&data[..32]);
    let amount_b = U256::from_big_endian(&data[32..]);
    (amount_a, amount_b)
}