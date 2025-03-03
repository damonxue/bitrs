use crate::config::Config;
use crate::solana::SolanaClientType;
use crate::{AppState, buyback::BuybackManager};

// API子模块导出
pub mod orderbook;
pub mod amm;
pub mod assets;
pub mod trades;
pub mod docs;
pub mod bridge;
pub mod rewards;
pub mod analytics;

use actix_web::{get, web, HttpResponse, Responder};
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
            .service(get_buyback_stats)
            .configure(amm::init_routes)
            .configure(bridge::init_routes)
            .configure(rewards::init_routes)
            .configure(analytics::init_routes)
            .configure(orderbook::init_routes)
            .configure(assets::init_routes)
            .configure(trades::init_routes)
    );
}