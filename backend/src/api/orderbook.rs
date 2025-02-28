use crate::{
    api::AppState,
    errors::ApiError,
    models::{OrderBook, Order, OrderRequest, Market, OrderSide},
};
use actix_web::{web, HttpResponse, Responder};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use log::{info, error};

/// 获取指定市场的订单簿数据
pub async fn get_orderbook(
    market_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取订单簿: {}", market_id);
    
    let solana_client = match &app_state.solana_client {
        Some(client) => client,
        None => return Err(ApiError::SolanaError("未连接到Solana节点".to_string())),
    };
    
    // 解析市场ID为Pubkey
    let market_id_pubkey = Pubkey::from_str(&market_id)
        .map_err(|_| ApiError::BadRequest("无效的市场ID格式".to_string()))?;
    
    // 查找订单簿账户地址
    let program_id = Pubkey::from_str(&app_state.config.orderbook_program_id)
        .map_err(|_| ApiError::BadRequest("无效的程序ID格式".to_string()))?;
    
    // 理论上这里应当查询链上数据，将链上的Order结构反序列化为API模型
    // 简化版实现，实际项目中需要接入真实数据
    
    // 模拟一些订单数据作为示例
    let bids = vec![
        Order {
            order_id: "bid1".to_string(),
            price: 50.0,
            size: 2.0,
            side: OrderSide::Buy,
            user: "user1".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        },
        Order {
            order_id: "bid2".to_string(),
            price: 49.5,
            size: 3.0,
            side: OrderSide::Buy,
            user: "user2".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        },
    ];
    
    let asks = vec![
        Order {
            order_id: "ask1".to_string(),
            price: 50.5,
            size: 1.5,
            side: OrderSide::Sell,
            user: "user3".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        },
        Order {
            order_id: "ask2".to_string(),
            price: 51.0,
            size: 4.0,
            side: OrderSide::Sell,
            user: "user4".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        },
    ];
    
    let orderbook = OrderBook {
        market_id: market_id.to_string(),
        bids,
        asks,
    };
    
    Ok(HttpResponse::Ok().json(orderbook))
}

/// 获取所有可用交易市场信息
pub async fn get_markets(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取市场列表");
    
    // 模拟一些市场数据作为示例
    let markets = vec![
        Market {
            market_id: "SOL-USDC".to_string(),
            base_token: "So11111111111111111111111111111111111111112".to_string(),
            quote_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            base_decimals: 9,
            quote_decimals: 6,
        },
        Market {
            market_id: "BTC-USDC".to_string(),
            base_token: "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E".to_string(),
            quote_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            base_decimals: 6,
            quote_decimals: 6,
        },
    ];
    
    Ok(HttpResponse::Ok().json(markets))
}

/// 提交新订单
pub async fn place_order(
    order_req: web::Json<OrderRequest>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("提交订单: {:?}", order_req);
    
    let solana_client = match &app_state.solana_client {
        Some(client) => client,
        None => return Err(ApiError::SolanaError("未连接到Solana节点".to_string())),
    };
    
    // 在实际项目中，这里需要:
    // 1. 验证订单签名
    // 2. 构造Solana交易
    // 3. 发送交易到链上
    // 4. 返回交易ID

    // 简化版模拟响应
    let tx_hash = format!("simulation_tx_{}", uuid::Uuid::new_v4());
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "tx_hash": tx_hash,
    })))
}

/// 取消现有订单
pub async fn cancel_order(
    order_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("取消订单: {}", order_id);
    
    let solana_client = match &app_state.solana_client {
        Some(client) => client,
        None => return Err(ApiError::SolanaError("未连接到Solana节点".to_string())),
    };
    
    // 在实际项目中，这里需要:
    // 1. 构造取消订单的Solana交易
    // 2. 发送交易到链上
    // 3. 返回交易ID
    
    // 简化版模拟响应
    let tx_hash = format!("cancel_tx_{}", uuid::Uuid::new_v4());
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "tx_hash": tx_hash,
    })))
}