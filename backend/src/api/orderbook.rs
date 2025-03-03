use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use log::{info};
use crate::{
    api::AppState,
    errors::ApiError,
    models::{Order, OrderBook, OrderRequest, OrderSide, Market},
};
use uuid::Uuid;

/// 获取可用交易对列表
#[get("/markets")]
pub async fn get_markets(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取交易对列表");
    
    // 在实际项目中，应该查询链上数据获取交易对列表
    // 这里使用模拟数据作为示例
    
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
            base_decimals: 8,
            quote_decimals: 6,
        },
    ];
    
    Ok(HttpResponse::Ok().json(markets))
}

/// 获取指定交易对的订单簿
#[get("/orderbook/{market_id}")]
pub async fn get_orderbook(
    market_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取订单簿: {}", market_id);
    
    // 在实际项目中，应该根据程序ID查询链上数据
    // 这里使用模拟数据作为示例
    
    // 模拟订单簿数据
    let bids = vec![
        Order {
            order_id: "bid-1".to_string(),
            price: 50.25,
            size: 2.5,
            side: OrderSide::Buy,
            user: "user-1".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        },
        Order {
            order_id: "bid-2".to_string(),
            price: 50.15,
            size: 1.8,
            side: OrderSide::Buy,
            user: "user-2".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        },
    ];
    
    let asks = vec![
        Order {
            order_id: "ask-1".to_string(),
            price: 50.35,
            size: 1.2,
            side: OrderSide::Sell,
            user: "user-3".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        },
        Order {
            order_id: "ask-2".to_string(),
            price: 50.45,
            size: 3.0,
            side: OrderSide::Sell,
            user: "user-4".to_string(),
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

/// 下单接口
#[post("/order")]
pub async fn place_order(
    order_req: web::Json<OrderRequest>,
    _app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("下单请求: {:?}", order_req);
    
    // 在实际项目中，这里应该:
    // 1. 验证签名
    // 2. 检查用户余额
    // 3. 将订单提交到链上
    // 4. 返回交易哈希
    
    // 模拟处理，返回交易哈希
    let tx_hash = format!("simulation_tx_{}", Uuid::new_v4());
    
    #[derive(Serialize)]
    struct OrderResponse {
        success: bool,
        tx_hash: String,
        message: String,
    }
    
    let response = OrderResponse {
        success: true,
        tx_hash,
        message: "Order submitted successfully".to_string(),
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// 取消订单接口
#[post("/cancel/{order_id}")]
pub async fn cancel_order(
    order_id: web::Path<String>,
    _app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("取消订单: {}", order_id);
    
    // 在实际项目中，这里应该:
    // 1. 验证用户是订单所有者
    // 2. 向链上提交取消订单指令
    // 3. 返回交易哈希
    
    // 模拟处理，返回交易哈希
    let tx_hash = format!("cancel_tx_{}", Uuid::new_v4());
    
    #[derive(Serialize)]
    struct CancelResponse {
        success: bool,
        tx_hash: String,
        order_id: String,
    }
    
    let response = CancelResponse {
        success: true,
        tx_hash,
        order_id: order_id.to_string(),
    };
    
    Ok(HttpResponse::Ok().json(response))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_markets)
       .service(get_orderbook)
       .service(place_order)
       .service(cancel_order);
}