use crate::config::Config;
use crate::solana::SolanaClientType;

// API子模块导出
pub mod orderbook;
pub mod amm;
pub mod assets;
pub mod trades;
pub mod docs;
pub mod bridge;
pub mod rewards;
pub mod analytics;

/// 应用全局状态，在各个API处理函数间共享
pub struct AppState {
    /// Solana客户端
    pub solana_client: SolanaClientType,
    /// 应用配置
    pub config: Config,
}

use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct BuybackStatsResponse {
    total_burned: u64,
    last_buyback_timestamp: u64,
    accumulated_fees: u64,
    next_buyback_estimate: u64,
    current_market_price: f64,
}

#[get("/stats/buyback")]
async fn get_buyback_stats(state: web::Data<AppState>) -> impl Responder {
    let stats = state.buyback_manager.get_stats().await;
    HttpResponse::Ok().json(stats)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(amm::init_routes)
            .configure(bridge::init_routes)
            .configure(rewards::init_routes)
            .configure(analytics::init_routes)
    );
}