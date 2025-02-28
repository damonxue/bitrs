use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// 订单簿相关数据模型
#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub price: f64,
    pub size: f64,
    pub side: OrderSide,
    pub user: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBook {
    pub market_id: String,
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Market {
    pub market_id: String,
    pub base_token: String,
    pub quote_token: String,
    pub base_decimals: u8,
    pub quote_decimals: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRequest {
    pub market_id: String,
    pub price: f64,
    pub size: f64,
    pub side: OrderSide,
    pub user: String,
    pub signature: Option<String>,
}

// AMM相关数据模型
#[derive(Debug, Serialize, Deserialize)]
pub struct Pool {
    pub pool_id: String,
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: f64,
    pub reserve_b: f64,
    pub fee_rate: f64,
    pub lp_token_supply: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapQuoteRequest {
    pub pool_id: String,
    pub token_in: String,
    pub amount_in: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapQuoteResponse {
    pub token_out: String,
    pub amount_out: f64,
    pub price_impact: f64,
    pub fee: f64,
}

// 资产相关数据模型
#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub logo_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub token: String,
    pub amount: f64,
}

// 交易相关数据模型
#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub market_id: String,
    pub price: f64,
    pub size: f64,
    pub side: OrderSide,
    pub timestamp: i64,
    pub bid_order_id: String,
    pub ask_order_id: String,
    pub bid_user: Option<String>,
    pub ask_user: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenValidation {
    pub token_mint: Pubkey,
    pub last_validation: u64,
    pub is_verified: bool,
    pub total_volume: u64,
    pub max_daily_volume: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RewardValidation {
    pub user: Pubkey,
    pub last_claim: u64,
    pub total_claimed: u64,
    pub staking_duration: u64,
    pub staking_amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuybackValidation {
    pub last_buyback: u64,
    pub total_burned: u64,
    pub min_interval: u64,
    pub max_buyback_amount: u64,
}

impl TokenValidation {
    pub fn validate_volume(&self, amount: u64) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let is_same_day = (current_time - self.last_validation) < 86400;
        let new_volume = if is_same_day {
            self.total_volume.saturating_add(amount)
        } else {
            amount
        };

        new_volume <= self.max_daily_volume
    }
}

impl RewardValidation {
    pub fn can_claim_rewards(&self, min_staking_duration: u64) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Ensure minimum staking duration is met
        if current_time - self.last_claim < min_staking_duration {
            return false;
        }

        // Verify staking amount is still locked
        self.staking_amount > 0
    }
}

impl BuybackValidation {
    pub fn can_execute_buyback(&self, amount: u64) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Ensure minimum interval between buybacks
        if current_time - self.last_buyback < self.min_interval {
            return false;
        }

        // Verify buyback amount is within limits
        amount <= self.max_buyback_amount
    }
}