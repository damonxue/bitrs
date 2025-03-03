use actix_web::{get, web, HttpResponse, Responder};
use crate::{
    errors::ApiError,
    models::{Trade, OrderSide},
    api::AppState,
};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use log::{info};

/// 获取最近交易记录
#[get("/trades/{market_id}")]
pub async fn get_recent_trades(
    market_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取交易记录: {}", market_id);
    
    // 在实际项目中，这里应该查询链上交易记录
    // 这里使用模拟数据作为示例
    
    let trades = vec![
        Trade {
            market_id: market_id.to_string(),
            price: 50.32,
            size: 1.2,
            side: OrderSide::Buy,
            timestamp: chrono::Utc::now().timestamp() - 60,
            bid_order_id: "bid-123".to_string(),
            ask_order_id: "ask-456".to_string(),
            bid_user: None,
            ask_user: None,
        },
        Trade {
            market_id: market_id.to_string(),
            price: 50.28,
            size: 0.8,
            side: OrderSide::Sell,
            timestamp: chrono::Utc::now().timestamp() - 180,
            bid_order_id: "bid-789".to_string(),
            ask_order_id: "ask-012".to_string(),
            bid_user: None,
            ask_user: None,
        },
        Trade {
            market_id: market_id.to_string(),
            price: 50.35,
            size: 2.0,
            side: OrderSide::Buy,
            timestamp: chrono::Utc::now().timestamp() - 300,
            bid_order_id: "bid-345".to_string(),
            ask_order_id: "ask-678".to_string(),
            bid_user: None,
            ask_user: None,
        },
    ];
    
    Ok(HttpResponse::Ok().json(trades))
}

/// 获取用户交易记录
#[get("/trades/user/{wallet}")]
pub async fn get_user_trades(
    wallet: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取用户交易记录: {}", wallet);
    
    // 验证钱包地址格式
    let pubkey = match Pubkey::from_str(&wallet) {
        Ok(key) => key,
        Err(_) => return Err(ApiError::BadRequest("无效的钱包地址".to_string())),
    };
    
    // 在实际项目中，这里应该查询链上该用户的交易记录
    // 这里使用模拟数据作为示例
    
    let trades = vec![
        Trade {
            market_id: "SOL-USDC".to_string(),
            price: 50.28,
            size: 1.5,
            side: OrderSide::Buy,
            timestamp: chrono::Utc::now().timestamp() - 1800,
            bid_order_id: "bid-123".to_string(),
            ask_order_id: "ask-456".to_string(),
            bid_user: Some(wallet.to_string()),
            ask_user: None,
        },
        Trade {
            market_id: "BTC-USDC".to_string(),
            price: 29850.50,
            size: 0.05,
            side: OrderSide::Buy,
            timestamp: chrono::Utc::now().timestamp() - 3600,
            bid_order_id: "bid-789".to_string(),
            ask_order_id: "ask-012".to_string(),
            bid_user: Some(wallet.to_string()),
            ask_user: None,
        },
        Trade {
            market_id: "SOL-USDC".to_string(),
            price: 50.15,
            size: 2.2,
            side: OrderSide::Sell,
            timestamp: chrono::Utc::now().timestamp() - 7200,
            bid_order_id: "bid-345".to_string(),
            ask_order_id: "ask-678".to_string(),
            bid_user: None,
            ask_user: Some(wallet.to_string()),
        },
    ];
    
    Ok(HttpResponse::Ok().json(trades))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_recent_trades)
       .service(get_user_trades);
}