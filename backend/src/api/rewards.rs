use actix_web::{web, get, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::{
    errors::ApiError,
    api::AppState,
    tokenomics::TokenEconomics,
};
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

#[derive(Serialize)]
struct StakingInfo {
    tokens_staked: u64,
    rewards_earned: u64,
    apr: f64,
    unlock_time: u64,
}

#[derive(Serialize)]
struct RewardsResponse {
    daily_rewards: u64,
    weekly_rewards: u64,
    current_apr: f64,
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

#[get("/staking/{wallet}")]
async fn get_staking_info(
    wallet: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    // Convert wallet address to pubkey
    let pubkey = match Pubkey::from_str(&wallet) {
        Ok(key) => key,
        Err(_) => return Err(ApiError::BadRequest("Invalid wallet address".to_string())),
    };
    
    // In a real implementation, this would fetch staking data from the blockchain
    // For now, we return mock data
    let staking_info = StakingInfo {
        tokens_staked: 10000,
        rewards_earned: 250,
        apr: 12.5,
        unlock_time: chrono::Utc::now().timestamp() as u64 + 86400 * 7, // 7 days from now
    };
    
    Ok(HttpResponse::Ok().json(staking_info))
}

#[get("/rewards/stats")]
async fn get_rewards_stats(
    token_economics: web::Data<Arc<TokenEconomics>>,
) -> Result<impl Responder, ApiError> {
    // In a real implementation, this would calculate actual rewards from on-chain data
    // For now, we return mock data based on tokenomics
    let rewards_info = RewardsResponse {
        daily_rewards: 1000,
        weekly_rewards: 7000,
        current_apr: 15.5,
    };
    
    Ok(HttpResponse::Ok().json(rewards_info))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_rewards)
       .service(get_pool_rewards)
       .service(get_staking_info)
       .service(get_rewards_stats);
}