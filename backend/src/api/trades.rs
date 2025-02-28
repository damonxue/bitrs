use crate::{
    api::AppState,
    errors::ApiError,
    models::{Trade, OrderSide},
};
use actix_web::{web, HttpResponse, Responder};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use log::{info, error};

/// 获取指定市场的最近交易记录
pub async fn get_recent_trades(
    market_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取最近交易: {}", market_id);
    
    // 在实际项目中，这些数据应当从链上或数据库中获取
    // 这里使用模拟数据作为示例
    
    let trades = vec![
        Trade {
            market_id: market_id.to_string(),
            price: 50.25,
            size: 1.2,
            side: OrderSide::Buy,
            timestamp: chrono::Utc::now().timestamp() - 60,
            bid_order_id: "bid123".to_string(),
            ask_order_id: "ask456".to_string(),
            bid_user: None, // 匿名化用户信息
            ask_user: None,
        },
        Trade {
            market_id: market_id.to_string(),
            price: 50.30,
            size: 0.5,
            side: OrderSide::Sell,
            timestamp: chrono::Utc::now().timestamp() - 180,
            bid_order_id: "bid789".to_string(),
            ask_order_id: "ask012".to_string(),
            bid_user: None,
            ask_user: None,
        },
        Trade {
            market_id: market_id.to_string(),
            price: 50.15,
            size: 2.0,
            side: OrderSide::Buy,
            timestamp: chrono::Utc::now().timestamp() - 300,
            bid_order_id: "bid345".to_string(),
            ask_order_id: "ask678".to_string(),
            bid_user: None,
            ask_user: None,
        },
    ];
    
    Ok(HttpResponse::Ok().json(trades))
}

/// 获取指定钱包地址的交易历史
pub async fn get_user_trades(
    wallet: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取用户交易历史: {}", wallet);
    
    // 验证钱包地址格式
    let wallet_pubkey = Pubkey::from_str(&wallet)
        .map_err(|_| ApiError::BadRequest("无效的钱包地址格式".to_string()))?;
    
    // 在实际项目中，应当从链上或数据库中查询用户参与的交易
    // 这里使用模拟数据作为示例
    
    let trades = vec![
        Trade {
            market_id: "SOL-USDC".to_string(),
            price: 51.25,
            size: 0.8,
            side: OrderSide::Buy,
            timestamp: chrono::Utc::now().timestamp() - 1800,
            bid_order_id: "user_bid1".to_string(),
            ask_order_id: "other_ask1".to_string(),
            bid_user: Some(wallet.to_string()),
            ask_user: None,
        },
        Trade {
            market_id: "BTC-USDC".to_string(),
            price: 50000.00,
            size: 0.01,
            side: OrderSide::Sell,
            timestamp: chrono::Utc::now().timestamp() - 3600,
            bid_order_id: "other_bid1".to_string(),
            ask_order_id: "user_ask1".to_string(),
            bid_user: None,
            ask_user: Some(wallet.to_string()),
        },
        Trade {
            market_id: "SOL-USDC".to_string(),
            price: 49.75,
            size: 1.5,
            side: OrderSide::Sell,
            timestamp: chrono::Utc::now().timestamp() - 7200,
            bid_order_id: "other_bid2".to_string(),
            ask_order_id: "user_ask2".to_string(),
            bid_user: None,
            ask_user: Some(wallet.to_string()),
        },
    ];
    
    Ok(HttpResponse::Ok().json(trades))
}