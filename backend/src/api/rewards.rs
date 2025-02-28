use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::tokenomics::TokenEconomics;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

#[derive(Serialize)]
struct RewardInfo {
    pending_rewards: f64,
    claimed_rewards: f64,
    apr: f64,
    next_distribution: u64,
}

#[derive(Serialize)]
struct PoolRewards {
    pool_id: String,
    daily_rewards: f64,
    apr: f64,
    total_staked: f64,
}

#[get("/rewards/{wallet}")]
async fn get_rewards(
    wallet: web::Path<String>,
    token_economics: web::Data<Arc<TokenEconomics>>,
) -> impl Responder {
    let pubkey = match Pubkey::from_str(&wallet) {
        Ok(key) => key,
        Err(_) => return HttpResponse::BadRequest().json("Invalid wallet address"),
    };

    let reward_info = RewardInfo {
        pending_rewards: 0.0,  // To be implemented with actual reward calculation
        claimed_rewards: 0.0,  // To be implemented with historical data
        apr: 0.0,             // To be calculated based on current rates
        next_distribution: 0,  // Next reward distribution block
    };

    HttpResponse::Ok().json(reward_info)
}

#[get("/pools/rewards")]
async fn get_pool_rewards(
    token_economics: web::Data<Arc<TokenEconomics>>,
) -> impl Responder {
    let pools = vec![
        PoolRewards {
            pool_id: "SOL-USDC".to_string(),
            daily_rewards: 1000.0,
            apr: 25.5,
            total_staked: 1000000.0,
        },
        // Add more pools as needed
    ];

    HttpResponse::Ok().json(pools)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_rewards)
       .service(get_pool_rewards);
}