use crate::{
    api::AppState,
    errors::ApiError,
    models::{Pool, SwapQuoteRequest, SwapQuoteResponse},
};
use actix_web::{web, HttpResponse, Responder};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use log::{info, error};
use actix_web::{get, post, web, HttpResponse, Responder};
use crate::tokenomics::TokenEconomics;
use std::sync::Arc;

pub struct AmmState {
    token_economics: Arc<TokenEconomics>,
}

#[get("/pools")]
async fn get_pools(state: web::Data<AmmState>) -> impl Responder {
    HttpResponse::Ok().json("List of liquidity pools")
}

#[post("/swap")]
async fn swap(
    state: web::Data<AmmState>,
    pool_id: web::Path<String>,
    swap_data: web::Json<SwapRequest>,
) -> impl Responder {
    // Calculate fees and distribute them according to tokenomics rules
    let trade_volume = swap_data.amount_in;
    let lp_address = swap_data.pool_provider;

    let fee_transactions = state.token_economics
        .distribute_trading_fees(trade_volume, lp_address)
        .await;

    // Execute the swap after handling fees
    HttpResponse::Ok().json("Swap executed")
}

#[post("/add-liquidity")]
async fn add_liquidity(
    state: web::Data<AmmState>,
    liquidity_data: web::Json<AddLiquidityRequest>,
) -> impl Responder {
    // Calculate and distribute mining rewards for the LP
    let mining_reward = state.token_economics
        .distribute_mining_rewards(
            liquidity_data.provider,
            liquidity_data.amount,
            liquidity_data.duration,
        )
        .await;

    HttpResponse::Ok().json("Liquidity added")
}

#[get("/apr/{pool_id}")]
async fn get_pool_apr(
    state: web::Data<AmmState>,
    pool_id: web::Path<String>,
) -> impl Responder {
    // Calculate current APR for the pool
    let pool_tvl = 1000000; // Get actual TVL from pool
    let apr = state.token_economics.get_lp_apr(pool_tvl).await;

    HttpResponse::Ok().json(apr)
}

// Request structs
#[derive(serde::Deserialize)]
struct SwapRequest {
    amount_in: u64,
    pool_provider: Pubkey,
    // other swap parameters...
}

#[derive(serde::Deserialize)]
struct AddLiquidityRequest {
    provider: Pubkey,
    amount: u64,
    duration: u64,
    // other liquidity parameters...
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_pools);
    cfg.service(swap);
    cfg.service(add_liquidity);
    cfg.service(get_pool_apr);
}

/// 获取所有可用AMM流动性池信息
pub async fn get_pools(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取流动性池列表");
    
    let solana_client = match &app_state.solana_client {
        Some(client) => client,
        None => return Err(ApiError::SolanaError("未连接到Solana节点".to_string())),
    };
    
    // 在实际项目中，应该查询链上数据获取流动性池列表
    // 这里使用模拟数据作为示例
    
    let pools = vec![
        Pool {
            pool_id: "SOL-USDC".to_string(),
            token_a: "So11111111111111111111111111111111111111112".to_string(),
            token_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            reserve_a: 1000.0,
            reserve_b: 50000.0,
            fee_rate: 0.003,
            lp_token_supply: 7071.0,
        },
        Pool {
            pool_id: "BTC-USDC".to_string(),
            token_a: "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E".to_string(),
            token_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            reserve_a: 5.0,
            reserve_b: 250000.0,
            fee_rate: 0.003,
            lp_token_supply: 1118.0,
        },
    ];
    
    Ok(HttpResponse::Ok().json(pools))
}

/// 获取指定流动性池的详细信息
pub async fn get_pool(
    pool_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取流动性池详情: {}", pool_id);
    
    let solana_client = match &app_state.solana_client {
        Some(client) => client,
        None => return Err(ApiError::SolanaError("未连接到Solana节点".to_string())),
    };
    
    // 在实际项目中，应该根据池ID查询链上数据
    // 这里使用模拟数据作为示例
    
    if pool_id.as_str() == "SOL-USDC" {
        let pool = Pool {
            pool_id: "SOL-USDC".to_string(),
            token_a: "So11111111111111111111111111111111111111112".to_string(),
            token_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            reserve_a: 1000.0,
            reserve_b: 50000.0,
            fee_rate: 0.003,
            lp_token_supply: 7071.0,
        };
        
        Ok(HttpResponse::Ok().json(pool))
    } else if pool_id.as_str() == "BTC-USDC" {
        let pool = Pool {
            pool_id: "BTC-USDC".to_string(),
            token_a: "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E".to_string(),
            token_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            reserve_a: 5.0,
            reserve_b: 250000.0,
            fee_rate: 0.003,
            lp_token_supply: 1118.0,
        };
        
        Ok(HttpResponse::Ok().json(pool))
    } else {
        Err(ApiError::NotFound("流动性池不存在".to_string()))
    }
}

/// 获取AMM交易报价
pub async fn get_swap_quote(
    quote_req: web::Json<SwapQuoteRequest>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取交易报价: {:?}", quote_req);
    
    // 在实际项目中，应该根据池数据计算实际的交换报价
    // 使用恒定乘积公式: x * y = k
    // 这里使用简化的计算方式
    
    let pool_id = &quote_req.pool_id;
    let amount_in = quote_req.amount_in;
    
    // 模拟不同池子的报价计算
    let (reserve_in, reserve_out, token_out, fee_rate) = if pool_id == "SOL-USDC" {
        if quote_req.token_in == "So11111111111111111111111111111111111111112" {
            // SOL -> USDC
            (1000.0, 50000.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 0.003)
        } else {
            // USDC -> SOL
            (50000.0, 1000.0, "So11111111111111111111111111111111111111112", 0.003)
        }
    } else if pool_id == "BTC-USDC" {
        if quote_req.token_in == "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E" {
            // BTC -> USDC
            (5.0, 250000.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", 0.003)
        } else {
            // USDC -> BTC
            (250000.0, 5.0, "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E", 0.003)
        }
    } else {
        return Err(ApiError::NotFound("流动性池不存在".to_string()));
    };
    
    // 计算手续费
    let fee = amount_in * fee_rate;
    
    // 应用恒定乘积公式计算输出金额
    let amount_in_with_fee = amount_in * (1.0 - fee_rate);
    let amount_out = reserve_out * amount_in_with_fee / (reserve_in + amount_in_with_fee);
    
    // 计算价格影响
    let spot_price = reserve_out / reserve_in;
    let execution_price = amount_out / amount_in;
    let price_impact = (1.0 - execution_price / spot_price) * 100.0;
    
    // 构造响应
    let response = SwapQuoteResponse {
        token_out: token_out.to_string(),
        amount_out,
        price_impact,
        fee,
    };
    
    Ok(HttpResponse::Ok().json(response))
}