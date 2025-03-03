use tokio;
use std::sync::Arc;
use tokio::sync::mpsc;
use log::{info, error, warn};
use prometheus_exporter::start_http_server;

mod events;
mod validators;
mod chains;
mod config;

use events::{ChainEvent, EventProcessor};
use validators::MessageValidator;
use chains::{ethereum, solana, bsc};
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    info!("BitRS Relayer starting...");

    // 加载配置
    let config = Config::load_default()?;
    info!("Configuration loaded successfully");

    // 初始化Prometheus度量
    if let Ok(metrics_server) = start_http_server(config.general.metrics_port) {
        info!("Metrics server started on port {}", config.general.metrics_port);
    } else {
        warn!("Failed to start metrics server");
    }

    // 设置事件处理通道
    let (event_sender, mut event_receiver) = mpsc::channel::<ChainEvent>(config.general.event_queue_size);

    // 初始化验证器
    let message_validator = Arc::new(MessageValidator::new(
        config.ethereum.required_confirmations,
        config.solana.required_confirmations,
        config.bsc.required_confirmations,
        config.ethereum.get_trusted_eth_addresses()?,
        config.solana.get_trusted_program_ids()?,
        config.bsc.get_trusted_eth_addresses()?,
    ));
    info!("Message validator initialized");

    // 初始化事件处理器
    let event_processor = Arc::new(EventProcessor::new(
        event_sender.clone(),
        config.ethereum.required_confirmations,
        config.bsc.required_confirmations,
        config.solana.required_confirmations,
    ));
    info!("Event processor initialized");

    // 启动链监听器
    let eth_config = config.ethereum.clone();
    let sol_config = config.solana.clone();
    let bsc_config = config.bsc.clone();

    info!("Starting Ethereum listener...");
    let eth_handle = ethereum::start_listener(
        event_sender.clone(), 
        eth_config.rpc_url, 
        eth_config.ws_url, 
        eth_config.trusted_contracts
    );
    
    info!("Starting Solana listener...");
    let sol_handle = solana::start_listener(
        event_sender.clone(), 
        sol_config.rpc_url, 
        sol_config.ws_url, 
        sol_config.trusted_programs,
        sol_config.commitment
    );
    
    info!("Starting BSC listener...");
    let bsc_handle = bsc::start_listener(
        event_sender.clone(), 
        bsc_config.rpc_url,
        bsc_config.ws_url,
        bsc_config.trusted_contracts
    );

    // 健康检查服务
    tokio::spawn(async move {
        let health_server = axum::Server::bind(&format!("0.0.0.0:{}", config.general.health_check_port).parse().unwrap())
            .serve(axum::routing::get("/health").to(|| async { "OK" }).into_make_service());
        
        info!("Health check server started on port {}", config.general.health_check_port);
        if let Err(e) = health_server.await {
            error!("Health check server error: {}", e);
        }
    });

    // 主事件处理循环
    let processor = event_processor.clone();
    let validator = message_validator.clone();
    
    tokio::spawn(async move {
        info!("Event processing loop started");
        while let Some(event) = event_receiver.recv().await {
            let event_processor = processor.clone();
            let event_validator = validator.clone();
            
            tokio::spawn(async move {
                match event_validator.validate_event(&event).await {
                    Ok(true) => {
                        info!("Event validated successfully: {:?}", event);
                        if let Err(e) = process_validated_event(&event).await {
                            error!("Error processing event: {}", e);
                        }
                    },
                    Ok(false) => {
                        warn!("Event validation failed: {:?}", event);
                    },
                    Err(e) => {
                        error!("Error during validation: {}", e);
                    }
                }
            });
        }
    });

    // 保持主线程存活
    info!("Relayer is running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down relayer...");
    Ok(())
}

async fn process_validated_event(event: &ChainEvent) -> Result<(), Box<dyn std::error::Error>> {
    match event {
        ChainEvent::EthereumEvent(eth_event) => {
            // 处理以太坊事件
            info!("Processing Ethereum event: {:?}", eth_event);
            // 实现跨链操作
        }
        ChainEvent::SolanaEvent(sol_event) => {
            // 处理Solana事件
            info!("Processing Solana event: {:?}", sol_event);
            // 实现跨链操作
        }
        ChainEvent::BscEvent(bsc_event) => {
            // 处理BSC事件
            info!("Processing BSC event: {:?}", bsc_event);
            // 实现跨链操作
        }
    }
    Ok(())
}