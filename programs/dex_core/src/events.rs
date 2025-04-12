use crate::orderbook::Side;
use anchor_lang::prelude::*;

// 事件处理器 - 负责发出各种事件
pub struct EventHandler;

impl EventHandler {
    // 发出市场创建事件
    pub fn emit_market_created(
        market: Pubkey,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        lot_size: u64,
        tick_size: u64,
    ) {
        emit!(MarketCreatedEvent {
            market,
            base_mint,
            quote_mint,
            lot_size,
            tick_size,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出下单事件
    pub fn emit_order_placed(
        market: Pubkey,
        order_id: u128,
        owner: Pubkey,
        side: Side,
        price: u64,
        quantity: u64,
    ) {
        emit!(OrderPlacedEvent {
            market,
            order_id,
            owner,
            side,
            price,
            quantity,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出取消订单事件
    pub fn emit_order_canceled(
        market: Pubkey,
        order_id: u128,
        owner: Pubkey,
        side: Side,
        price: u64,
        quantity: u64,
    ) {
        emit!(OrderCanceledEvent {
            market,
            order_id,
            owner,
            side,
            price,
            quantity,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出交易事件
    pub fn emit_trade(
        market: Pubkey,
        side: Side,
        maker_order_id: u128,
        taker_order_id: u128,
        price: u64,
        quantity: u64,
        maker: Pubkey,
        taker: Pubkey,
    ) {
        emit!(TradeEvent {
            market,
            side,
            maker_order_id,
            taker_order_id,
            price,
            quantity,
            maker,
            taker,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出资金结算事件
    pub fn emit_funds_settled(market: Pubkey, owner: Pubkey, base_amount: u64, quote_amount: u64) {
        emit!(FundsSettledEvent {
            market,
            owner,
            base_amount,
            quote_amount,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出LP池流动性变化事件
    pub fn emit_liquidity_changed(
        market: Pubkey,
        owner: Pubkey,
        is_addition: bool, // true: 添加流动性, false: 移除流动性
        base_amount: u64,
        quote_amount: u64,
        lp_tokens: u64,
    ) {
        emit!(LiquidityChangedEvent {
            market,
            owner,
            is_addition,
            base_amount,
            quote_amount,
            lp_tokens,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出奖励领取事件
    pub fn emit_rewards_claimed(market: Pubkey, owner: Pubkey, reward_amount: u64) {
        emit!(RewardsClaimedEvent {
            market,
            owner,
            reward_amount,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出跨链订单创建事件
    pub fn emit_cross_chain_order_created(
        source_market: Pubkey,
        target_chain_id: u64,
        order_id: u128,
        owner: Pubkey,
        base_amount: u64,
        quote_amount: u64,
    ) {
        emit!(CrossChainOrderCreatedEvent {
            source_market,
            target_chain_id,
            order_id,
            owner,
            base_amount,
            quote_amount,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出跨链交易确认事件
    pub fn emit_cross_chain_tx_confirmed(
        target_market: Pubkey,
        source_chain_id: u64,
        tx_hash: [u8; 32],
        order_id: u128,
        owner: Pubkey,
    ) {
        emit!(CrossChainTxConfirmedEvent {
            target_market,
            source_chain_id,
            tx_hash,
            order_id,
            owner,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出高级订单事件
    pub fn emit_advanced_order_created(
        market: Pubkey,
        order_id: u128,
        owner: Pubkey,
        strategy_type: u8,
        params_hash: [u8; 32],
    ) {
        emit!(AdvancedOrderCreatedEvent {
            market,
            order_id,
            owner,
            strategy_type,
            params_hash,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出市场状态更新事件
    pub fn emit_market_status_changed(market: Pubkey, is_active: bool, reason: String) {
        emit!(MarketStatusChangedEvent {
            market,
            is_active,
            reason,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出风险警告事件
    pub fn emit_risk_warning(
        market: Pubkey,
        owner: Pubkey,
        warning_type: RiskWarningType,
        severity: u8,
        details: String,
    ) {
        emit!(RiskWarningEvent {
            market,
            owner,
            warning_type,
            severity,
            details,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出市场状态监控事件
    pub fn emit_market_metrics(
        market: Pubkey,
        volume_24h: u64,
        trades_count: u32,
        highest_bid: u64,
        lowest_ask: u64,
        liquidity_index: u32,
    ) {
        emit!(MarketMetricsEvent {
            market,
            volume_24h,
            trades_count,
            highest_bid,
            lowest_ask,
            liquidity_index,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出账户活动事件
    pub fn emit_account_activity(
        owner: Pubkey,
        activity_type: AccountActivityType,
        market: Option<Pubkey>,
        amount: u64,
        related_tx: Option<[u8; 32]>,
    ) {
        emit!(AccountActivityEvent {
            owner,
            activity_type,
            market,
            amount,
            related_tx,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出系统状态事件
    pub fn emit_system_status(
        status_type: SystemStatusType,
        is_healthy: bool,
        metrics: SystemMetrics,
        message: Option<String>,
    ) {
        emit!(SystemStatusEvent {
            status_type,
            is_healthy,
            metrics,
            message,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }

    // 发出存储优化事件
    pub fn emit_storage_optimization(
        market: Pubkey,
        optimization_type: StorageOptimizationType,
        old_size: u32,
        new_size: u32,
        success: bool,
    ) {
        emit!(StorageOptimizationEvent {
            market,
            optimization_type,
            old_size,
            new_size,
            success,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
}

// 定义各种事件
#[event]
pub struct MarketCreatedEvent {
    pub market: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lot_size: u64,
    pub tick_size: u64,
    pub timestamp: i64,
}

#[event]
pub struct OrderPlacedEvent {
    pub market: Pubkey,
    pub order_id: u128,
    pub owner: Pubkey,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: i64,
}

#[event]
pub struct OrderCanceledEvent {
    pub market: Pubkey,
    pub order_id: u128,
    pub owner: Pubkey,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: i64,
}

#[event]
pub struct TradeEvent {
    pub market: Pubkey,
    pub side: Side,
    pub maker_order_id: u128,
    pub taker_order_id: u128,
    pub price: u64,
    pub quantity: u64,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct FundsSettledEvent {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityChangedEvent {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub is_addition: bool,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub lp_tokens: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardsClaimedEvent {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub reward_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct CrossChainOrderCreatedEvent {
    pub source_market: Pubkey,
    pub target_chain_id: u64,
    pub order_id: u128,
    pub owner: Pubkey,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct CrossChainTxConfirmedEvent {
    pub target_market: Pubkey,
    pub source_chain_id: u64,
    pub tx_hash: [u8; 32],
    pub order_id: u128,
    pub owner: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AdvancedOrderCreatedEvent {
    pub market: Pubkey,
    pub order_id: u128,
    pub owner: Pubkey,
    pub strategy_type: u8,
    pub params_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct MarketStatusChangedEvent {
    pub market: Pubkey,
    pub is_active: bool,
    pub reason: String,
    pub timestamp: i64,
}

// 定义风险警告类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum RiskWarningType {
    PriceSurge = 0,
    PriceCollapse = 1,
    UnusualVolume = 2,
    LiquidityDrain = 3,
    MarketManipulation = 4,
    FundingSafety = 5,
    AccountAnomaly = 6,
    SystemRisk = 7,
}

// 定义账户活动类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum AccountActivityType {
    Deposit = 0,
    Withdrawal = 1,
    FeePayment = 2,
    RewardClaim = 3,
    MarginCall = 4,
    LiquidationEvent = 5,
    AccountUpdate = 6,
}

// 定义系统状态类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum SystemStatusType {
    OrderbookStatus = 0,
    LiquidityPoolStatus = 1,
    CrossChainBridgeStatus = 2,
    OracleStatus = 3,
    GlobalMarketStatus = 4,
    ProtocolUpgrade = 5,
}

// 定义存储优化类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum StorageOptimizationType {
    Compression = 0,
    Defragmentation = 1,
    HistoryTruncation = 2,
    CachePruning = 3,
    IndexRebuild = 4,
}

// 系统指标结构
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct SystemMetrics {
    pub order_processing_latency_ms: u32,
    pub transaction_count_per_second: u32,
    pub memory_usage_percentage: u8,
    pub cpu_usage_percentage: u8,
    pub active_users_count: u32,
}

// 新增的事件定义
#[event]
pub struct RiskWarningEvent {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub warning_type: RiskWarningType,
    pub severity: u8, // 0-100, 数字越大表示严重性越高
    pub details: String,
    pub timestamp: i64,
}

#[event]
pub struct MarketMetricsEvent {
    pub market: Pubkey,
    pub volume_24h: u64,
    pub trades_count: u32,
    pub highest_bid: u64,
    pub lowest_ask: u64,
    pub liquidity_index: u32, // 衡量市场深度的指标
    pub timestamp: i64,
}

#[event]
pub struct AccountActivityEvent {
    pub owner: Pubkey,
    pub activity_type: AccountActivityType,
    pub market: Option<Pubkey>,
    pub amount: u64,
    pub related_tx: Option<[u8; 32]>,
    pub timestamp: i64,
}

#[event]
pub struct SystemStatusEvent {
    pub status_type: SystemStatusType,
    pub is_healthy: bool,
    pub metrics: SystemMetrics,
    pub message: Option<String>,
    pub timestamp: i64,
}

#[event]
pub struct StorageOptimizationEvent {
    pub market: Pubkey,
    pub optimization_type: StorageOptimizationType,
    pub old_size: u32,
    pub new_size: u32,
    pub success: bool,
    pub timestamp: i64,
}
