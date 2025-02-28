use crate::{
    api::AppState,
    errors::ApiError,
    models::{Token, Balance},
};
use actix_web::{web, HttpResponse, Responder};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use log::{info, error};

/// 获取指定钱包地址的资产余额
pub async fn get_balance(
    wallet: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取钱包余额: {}", wallet);
    
    let solana_client = match &app_state.solana_client {
        Some(client) => client,
        None => return Err(ApiError::SolanaError("未连接到Solana节点".to_string())),
    };
    
    // 验证钱包地址格式
    let wallet_pubkey = Pubkey::from_str(&wallet)
        .map_err(|_| ApiError::BadRequest("无效的钱包地址格式".to_string()))?;
    
    // 在实际项目中，应当查询链上SPL代币账户数据
    // 这里使用模拟数据作为示例
    
    let balances = vec![
        Balance {
            token: "So11111111111111111111111111111111111111112".to_string(), // SOL
            amount: 10.5,
        },
        Balance {
            token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            amount: 500.75,
        },
    ];
    
    Ok(HttpResponse::Ok().json(balances))
}

/// 获取交易所支持的代币列表
pub async fn get_supported_tokens(
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    info!("获取支持代币列表");
    
    // 在实际项目中，可以从数据库或配置中获取支持的代币列表
    // 这里使用内置列表作为示例
    
    let tokens = vec![
        Token {
            mint: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            decimals: 9,
            logo_uri: Some("https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png".to_string()),
        },
        Token {
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            logo_uri: Some("https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png".to_string()),
        },
        Token {
            mint: "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E".to_string(),
            symbol: "BTC".to_string(),
            name: "Wrapped Bitcoin (Sollet)".to_string(),
            decimals: 6,
            logo_uri: Some("https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E/logo.png".to_string()),
        },
    ];
    
    Ok(HttpResponse::Ok().json(tokens))
}