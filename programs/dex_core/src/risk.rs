use crate::core::PlaceOrder;
use crate::events::{EventHandler, RiskWarningType};
use crate::orderbook::{Order, Side};
use crate::ErrorCode;
use anchor_lang::prelude::*;
use std::collections::HashMap;

// 风险引擎 - 负责风控和安全检查
pub struct RiskEngine;

// 风险级别定义
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

// 风险参数配置
#[account]
#[derive(Default)]
pub struct RiskParameters {
    pub market: Pubkey,                       // 市场公钥
    pub authority: Pubkey,                    // 风控参数管理员
    pub price_limit_percent: u8,              // 价格限制百分比
    pub max_open_orders_per_user: u16,        // 每用户最大挂单数
    pub max_position_size: u64,               // 最大持仓规模
    pub market_open_hour: u8,                 // 市场开放时间（小时）
    pub market_close_hour: u8,                // 市场关闭时间（小时）
    pub is_maintenance_mode: bool,            // 是否处于维护模式
    pub circuit_breaker_threshold: i16,       // 熔断阈值（价格变动百分比，基点）
    pub circuit_breaker_cooldown_minutes: u8, // 熔断冷却时间（分钟）
    pub last_circuit_breaker_time: i64,       // 上次熔断时间
    pub max_concentration_ratio: u8,          // 最大集中度比率（单一用户占比）
    pub min_order_size: u64,                  // 最小订单规模
    pub max_order_size: u64,                  // 最大订单规模
    pub is_active: bool,                      // 是否启用风控系统
    // 保留字段，用于将来的扩展
    pub reserved: [u8; 64],
}

// 用户风险记录
#[account]
#[derive(Default)]
pub struct UserRiskProfile {
    pub owner: Pubkey,                // 用户公钥
    pub risk_score: u8,               // 风险评分(0-100)
    pub last_warning_ts: i64,         // 上次风险警告时间
    pub warning_count: u16,           // 风险警告计数
    pub total_volume_30d: u64,        // 30天交易量
    pub max_position_value: u64,      // 最大持仓价值
    pub last_activity_ts: i64,        // 上次活动时间
    pub is_restricted: bool,          // 是否受限
    pub markets_traded: [Pubkey; 10], // 交易过的市场
    pub markets_count: u8,            // 交易市场数量
    // 保留字段，用于将来的扩展
    pub reserved: [u8; 64],
}

// 市场风险指标
#[account]
#[derive(Default)]
pub struct MarketRiskMetrics {
    pub market: Pubkey,                  // 市场公钥
    pub last_update_slot: u64,           // 最后更新的slot
    pub price_change_24h_bps: i32,       // 24小时价格变化（基点）
    pub volume_24h: u64,                 // 24小时交易量
    pub liquidity_index: u32,            // 流动性指数
    pub volatility_index: u32,           // 波动率指数
    pub circuit_breaker_triggered: bool, // 是否触发熔断
    pub price_band_lower: u64,           // 价格下限
    pub price_band_upper: u64,           // 价格上限
    pub max_buy_size: u64,               // 最大买单规模
    pub max_sell_size: u64,              // 最大卖单规模
    pub avg_execution_time_ms: u32,      // 平均执行时间(毫秒)
    pub active_users_count: u32,         // 活跃用户数
    // 保留字段，用于将来的扩展
    pub reserved: [u8; 64],
}

impl RiskEngine {
    // 检查用户下单前是否有足够的资金
    pub fn check_funds(ctx: &PlaceOrder, order: &Order) -> Result<()> {
        let market = &ctx.accounts.market;

        match order.side {
            Side::Bid => {
                // 对于买单，检查用户是否有足够的报价代币
                let required_funds = Self::calculate_required_quote_for_bid(
                    order.price,
                    order.quantity,
                    market.lot_size,
                    market.tick_size,
                    market.taker_fee,
                );

                // 如果用户账户余额小于所需资金，则返回错误
                require!(
                    ctx.accounts.user_token_account.amount >= required_funds,
                    ErrorCode::InsufficientFunds
                );
            }
            Side::Ask => {
                // 对于卖单，检查用户是否有足够的基础代币
                let required_base = order.quantity * market.lot_size;

                // 如果用户账户余额小于所需资金，则返回错误
                require!(
                    ctx.accounts.user_token_account.amount >= required_base,
                    ErrorCode::InsufficientFunds
                );
            }
        }

        Ok(())
    }

    // 计算买单所需的报价代币数量
    pub fn calculate_required_quote_for_bid(
        price: u64,
        quantity: u64,
        lot_size: u64,
        tick_size: u64,
        taker_fee: i64,
    ) -> u64 {
        // 计算基本金额：价格 * 数量 * 最小交易单位
        let base_amount = price
            .checked_mul(quantity)
            .unwrap()
            .checked_mul(lot_size)
            .unwrap()
            .checked_div(tick_size)
            .unwrap();

        // 如果有手续费，加上手续费
        let fee_amount = if taker_fee > 0 {
            (base_amount as u128)
                .checked_mul(taker_fee as u128)
                .unwrap()
                .checked_div(10000)
                .unwrap() as u64
        } else {
            0
        };

        base_amount.checked_add(fee_amount).unwrap()
    }

    // 检查市场是否接近容量限制
    pub fn check_market_capacity(order_book_free_capacity: u32) -> Result<()> {
        // 如果剩余容量少于一定百分比，发出警告
        if order_book_free_capacity < 100 {
            // 例如：剩余订单节点少于100个
            msg!(
                "警告：市场接近容量上限，剩余容量：{}",
                order_book_free_capacity
            );
        }

        Ok(())
    }

    // 检查交易价格是否在合理范围内
    pub fn check_price_within_limits(
        price: u64,
        side: Side,
        last_price: Option<u64>,
        price_limit_percent: u8,
    ) -> Result<()> {
        // 如果有上一次成交价，检查价格波动是否在限制范围内
        if let Some(last) = last_price {
            let limit = (last as u128 * price_limit_percent as u128 / 100) as u64;

            match side {
                Side::Bid => {
                    // 买单价格不能超过上一次价格的一定百分比
                    if price > last.checked_add(limit).unwrap_or(u64::MAX) {
                        return Err(ErrorCode::InvalidOrderPrice.into());
                    }
                }
                Side::Ask => {
                    // 卖单价格不能低于上一次价格的一定百分比
                    if price < last.checked_sub(limit).unwrap_or(0) {
                        return Err(ErrorCode::InvalidOrderPrice.into());
                    }
                }
            }
        }

        Ok(())
    }

    // 验证用户是否在允许的操作限制内
    pub fn validate_user_limits(
        user_pubkey: Pubkey,
        order_count: u16,
        max_orders_per_user: u16,
    ) -> Result<()> {
        // 检查用户是否超过最大订单数量限制
        if order_count >= max_orders_per_user {
            return Err(ErrorCode::OrderBookFull.into());
        }

        // 可以添加更多的用户限制检查，例如交易频率、交易量等

        Ok(())
    }

    // 检查市场操作是否在允许的时段
    pub fn check_market_hours(
        current_ts: i64,
        market_open_ts: i64,
        market_close_ts: i64,
        is_maintenance_mode: bool,
    ) -> Result<()> {
        // 如果市场处于维护模式，禁止所有交易
        if is_maintenance_mode {
            return Err(ErrorCode::MarketNotActive.into());
        }

        // 如果设置了开放和关闭时间，检查当前是否在交易时段
        if market_open_ts > 0 && market_close_ts > 0 {
            if current_ts < market_open_ts || current_ts > market_close_ts {
                return Err(ErrorCode::MarketNotActive.into());
            }
        }

        Ok(())
    }

    // 计算订单的风险分数
    pub fn calculate_risk_score(
        user_pubkey: Pubkey,
        order_size: u64,
        order_price: u64,
        market_volatility: u64,
        user_history_score: u8,
    ) -> u8 {
        // 风险分数计算逻辑，综合考虑订单大小、市场波动性和用户历史
        // 返回0-100的分数，数字越大风险越高

        let size_factor = if order_size > 1000 { 30 } else { 10 };
        let volatility_factor = if market_volatility > 500 { 40 } else { 20 };
        let history_factor = user_history_score; // 用户历史记录分数（0-30）

        let total_score = size_factor + volatility_factor + history_factor;
        std::cmp::min(total_score, 100)
    }

    // 推送风险事件到监控系统
    pub fn emit_risk_event(market: Pubkey, user: Pubkey, event_type: &str, risk_score: u8) {
        msg!(
            "风险事件: 市场={}, 用户={}, 类型={}, 风险分数={}",
            market,
            user,
            event_type,
            risk_score
        );

        // 实际实现中可能需要将事件写入日志或通过其他方式通知监控系统
    }

    // 初始化风险参数
    pub fn initialize_risk_parameters(
        ctx: Context<InitializeRiskParams>,
        price_limit_percent: u8,
        max_open_orders_per_user: u16,
        max_position_size: u64,
        market_open_hour: u8,
        market_close_hour: u8,
    ) -> Result<()> {
        let risk_params = &mut ctx.accounts.risk_parameters;

        risk_params.market = ctx.accounts.market.key();
        risk_params.authority = ctx.accounts.authority.key();
        risk_params.price_limit_percent = price_limit_percent;
        risk_params.max_open_orders_per_user = max_open_orders_per_user;
        risk_params.max_position_size = max_position_size;
        risk_params.market_open_hour = market_open_hour;
        risk_params.market_close_hour = market_close_hour;
        risk_params.is_maintenance_mode = false;
        risk_params.circuit_breaker_threshold = 500; // 默认5%
        risk_params.circuit_breaker_cooldown_minutes = 10;
        risk_params.last_circuit_breaker_time = 0;
        risk_params.max_concentration_ratio = 20; // 默认20%
        risk_params.min_order_size = 1;
        risk_params.max_order_size = u64::MAX;
        risk_params.is_active = true;

        Ok(())
    }

    // 更新风险参数
    pub fn update_risk_parameters(
        ctx: Context<UpdateRiskParams>,
        param_type: u8,
        value: u64,
    ) -> Result<()> {
        let risk_params = &mut ctx.accounts.risk_parameters;

        // 验证调用者是否为授权账户
        require!(
            ctx.accounts.authority.key() == risk_params.authority,
            ErrorCode::UnauthorizedOperation
        );

        // 根据参数类型更新相应的字段
        match param_type {
            0 => risk_params.price_limit_percent = value as u8,
            1 => risk_params.max_open_orders_per_user = value as u16,
            2 => risk_params.max_position_size = value,
            3 => risk_params.market_open_hour = value as u8,
            4 => risk_params.market_close_hour = value as u8,
            5 => risk_params.is_maintenance_mode = value != 0,
            6 => risk_params.circuit_breaker_threshold = value as i16,
            7 => risk_params.circuit_breaker_cooldown_minutes = value as u8,
            8 => risk_params.max_concentration_ratio = value as u8,
            9 => risk_params.min_order_size = value,
            10 => risk_params.max_order_size = value,
            11 => risk_params.is_active = value != 0,
            _ => return Err(ErrorCode::InvalidParameters.into()),
        }

        Ok(())
    }

    // 检查订单是否符合风险参数
    pub fn validate_order_risk(
        ctx: &Context<PlaceOrder>,
        order: &Order,
        risk_params: &RiskParameters,
    ) -> Result<()> {
        // 如果风控系统未启用，直接通过
        if !risk_params.is_active {
            return Ok(());
        }

        // 检查维护模式
        if risk_params.is_maintenance_mode {
            return Err(ErrorCode::MarketNotActive.into());
        }

        // 检查交易时段
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        let current_hour = ((current_timestamp / 3600) % 24) as u8;

        if risk_params.market_open_hour < risk_params.market_close_hour {
            // 正常时段 (例如: 9:00 - 17:00)
            if current_hour < risk_params.market_open_hour
                || current_hour >= risk_params.market_close_hour
            {
                return Err(ErrorCode::MarketNotActive.into());
            }
        } else {
            // 跨午夜时段 (例如: 22:00 - 6:00)
            if current_hour < risk_params.market_open_hour
                && current_hour >= risk_params.market_close_hour
            {
                return Err(ErrorCode::MarketNotActive.into());
            }
        }

        // 检查订单规模
        if order.quantity < risk_params.min_order_size {
            return Err(ErrorCode::InvalidOrderQuantity.into());
        }

        if order.quantity > risk_params.max_order_size {
            return Err(ErrorCode::InvalidOrderQuantity.into());
        }

        // 检查熔断
        if risk_params.last_circuit_breaker_time > 0 {
            let cooldown_seconds = risk_params.circuit_breaker_cooldown_minutes as i64 * 60;
            if current_timestamp - risk_params.last_circuit_breaker_time < cooldown_seconds {
                return Err(ErrorCode::MarketNotActive.into());
            }
        }

        // 其他风险检查可以在这里添加

        Ok(())
    }

    // 检测价格异常波动
    pub fn detect_price_anomalies(
        market: Pubkey,
        current_price: u64,
        avg_price_24h: u64,
        risk_params: &RiskParameters,
    ) -> Result<bool> {
        if avg_price_24h == 0 {
            return Ok(false);
        }

        // 计算价格变化百分比（基点）
        let price_change_bps = if current_price > avg_price_24h {
            ((current_price as i128 - avg_price_24h as i128) * 10000 / avg_price_24h as i128) as i16
        } else {
            -((avg_price_24h as i128 - current_price as i128) * 10000 / avg_price_24h as i128)
                as i16
        };

        // 如果价格变化超过熔断阈值，触发熔断
        if price_change_bps.abs() > risk_params.circuit_breaker_threshold {
            // 记录熔断时间
            let mut risk_params_mut = risk_params.clone();
            risk_params_mut.last_circuit_breaker_time = Clock::get()?.unix_timestamp;

            // 发出风险警告事件
            let warning_type = if price_change_bps > 0 {
                RiskWarningType::PriceSurge
            } else {
                RiskWarningType::PriceCollapse
            };

            EventHandler::emit_risk_warning(
                market,
                Pubkey::default(), // 系统级警告，无特定用户
                warning_type,
                95, // 高严重性
                format!("价格异常波动: {}基点", price_change_bps),
            );

            return Ok(true); // 返回熔断已触发
        }

        Ok(false)
    }

    // 分析用户交易模式
    pub fn analyze_user_trading_pattern(
        user_profile: &UserRiskProfile,
        recent_orders: &[Order],
        market_metrics: &MarketRiskMetrics,
    ) -> RiskLevel {
        // 基本风险级别基于用户风险评分
        let base_risk_level = if user_profile.risk_score < 30 {
            RiskLevel::Low
        } else if user_profile.risk_score < 60 {
            RiskLevel::Medium
        } else if user_profile.risk_score < 85 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        };

        // 分析订单频率
        let order_frequency = recent_orders.len();
        let frequency_risk = if order_frequency > 100 {
            RiskLevel::High
        } else if order_frequency > 50 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // 分析订单规模异常
        let mut unusual_size_count = 0;
        for order in recent_orders {
            if order.quantity > market_metrics.max_buy_size / 2
                || order.quantity > market_metrics.max_sell_size / 2
            {
                unusual_size_count += 1;
            }
        }

        let size_risk = if unusual_size_count > 5 {
            RiskLevel::High
        } else if unusual_size_count > 2 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // 综合风险级别（取最高风险）
        let combined_risk = match (base_risk_level, frequency_risk, size_risk) {
            (RiskLevel::Critical, _, _) => RiskLevel::Critical,
            (_, RiskLevel::Critical, _) => RiskLevel::Critical,
            (_, _, RiskLevel::Critical) => RiskLevel::Critical,
            (RiskLevel::High, _, _) => RiskLevel::High,
            (_, RiskLevel::High, _) => RiskLevel::High,
            (_, _, RiskLevel::High) => RiskLevel::High,
            (RiskLevel::Medium, _, _) => RiskLevel::Medium,
            (_, RiskLevel::Medium, _) => RiskLevel::Medium,
            (_, _, RiskLevel::Medium) => RiskLevel::Medium,
            _ => RiskLevel::Low,
        };

        combined_risk
    }

    // 更新市场风险指标
    pub fn update_market_risk_metrics(
        metrics: &mut MarketRiskMetrics,
        current_price: u64,
        last_price_24h: u64,
        volume_24h: u64,
        liquidity_index: u32,
        execution_time_ms: u32,
        active_users: u32,
    ) {
        metrics.last_update_slot = Clock::get().unwrap().slot;

        // 计算24小时价格变化（基点）
        if last_price_24h > 0 {
            metrics.price_change_24h_bps = if current_price > last_price_24h {
                ((current_price as i128 - last_price_24h as i128) * 10000 / last_price_24h as i128)
                    as i32
            } else {
                -((last_price_24h as i128 - current_price as i128) * 10000 / last_price_24h as i128)
                    as i32
            };
        }

        metrics.volume_24h = volume_24h;
        metrics.liquidity_index = liquidity_index;

        // 计算波动率指数 (简化版，基于价格变化的绝对值)
        metrics.volatility_index = (metrics.price_change_24h_bps.abs() as u32) / 10;

        // 确定价格波动区间
        metrics.price_band_lower = current_price.saturating_sub(current_price / 10); // 下限：当前价格的90%
        metrics.price_band_upper = current_price.saturating_add(current_price / 10); // 上限：当前价格的110%

        metrics.avg_execution_time_ms = execution_time_ms;
        metrics.active_users_count = active_users;
    }

    // 检测市场操纵行为
    pub fn detect_market_manipulation(
        market: Pubkey,
        user: Pubkey,
        recent_trades: &[(u64, u64, Side)], // (价格,数量,方向)
        market_metrics: &MarketRiskMetrics,
    ) -> Result<bool> {
        if recent_trades.len() < 5 {
            return Ok(false); // 交易次数不足以判断
        }

        // 检测价格操纵 - 分析快速买卖形成的价格压力
        let mut buy_volume = 0u64;
        let mut sell_volume = 0u64;
        let mut price_movements = 0;
        let mut last_price = recent_trades[0].0;

        for &(price, quantity, side) in recent_trades {
            match side {
                Side::Bid => buy_volume = buy_volume.saturating_add(quantity),
                Side::Ask => sell_volume = sell_volume.saturating_add(quantity),
            }

            // 计算价格变动
            if (price as i128 - last_price as i128).abs() * 10000 / last_price as i128 > 100 {
                // 价格变动超过1%
                price_movements += 1;
            }
            last_price = price;
        }

        // 检测洗盘行为 - 大量自买自卖
        let wash_trading_suspected = buy_volume > market_metrics.volume_24h / 20
            && sell_volume > market_metrics.volume_24h / 20
            && (buy_volume as i128 - sell_volume as i128).abs() < (buy_volume as i128 / 10);

        // 检测价格操纵 - 频繁的大幅价格波动
        let price_manipulation_suspected = price_movements > 3;

        if wash_trading_suspected || price_manipulation_suspected {
            // 发出风险警告
            EventHandler::emit_risk_warning(
                market,
                user,
                RiskWarningType::MarketManipulation,
                90, // 高严重性
                format!(
                    "可能的市场操纵行为: 洗盘={}, 价格操纵={}",
                    wash_trading_suspected, price_manipulation_suspected
                ),
            );

            return Ok(true);
        }

        Ok(false)
    }

    // 获取适用的风险等级
    pub fn get_applicable_risk_level(
        user_profile: Option<&UserRiskProfile>,
        market_metrics: &MarketRiskMetrics,
        order_size: u64,
        order_price: u64,
    ) -> RiskLevel {
        // 基于用户历史计算基础风险等级
        let base_risk_level = if let Some(profile) = user_profile {
            if profile.risk_score > 80 {
                RiskLevel::High
            } else if profile.risk_score > 50 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            }
        } else {
            RiskLevel::Medium // 没有用户档案时的默认风险
        };

        // 基于市场条件的风险
        let market_risk_level = if market_metrics.volatility_index > 50 {
            RiskLevel::High
        } else if market_metrics.volatility_index > 20 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // 基于订单特征的风险
        let order_risk_level = if order_size > market_metrics.max_buy_size / 2
            || (order_price > market_metrics.price_band_upper
                || order_price < market_metrics.price_band_lower)
        {
            RiskLevel::High
        } else if order_size > market_metrics.max_buy_size / 5 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // 取三者中最高的风险等级
        let max_risk = [base_risk_level, market_risk_level, order_risk_level]
            .iter()
            .fold(RiskLevel::Low, |acc, &level| {
                if level as u8 > acc as u8 {
                    level
                } else {
                    acc
                }
            });

        max_risk
    }

    // 计算交易所整体风险指数
    pub fn calculate_exchange_risk_index(market_metrics: &[MarketRiskMetrics]) -> u8 {
        if market_metrics.is_empty() {
            return 50; // 默认中等风险
        }

        let mut total_volatility = 0u32;
        let mut circuit_breaker_count = 0u32;
        let mut total_volume = 0u64;

        for metric in market_metrics {
            total_volatility += metric.volatility_index;
            if metric.circuit_breaker_triggered {
                circuit_breaker_count += 1;
            }
            total_volume += metric.volume_24h;
        }

        let avg_volatility = total_volatility / market_metrics.len() as u32;
        let circuit_breaker_ratio = circuit_breaker_count * 100 / market_metrics.len() as u32;

        // 计算风险指数 (0-100)
        let mut risk_index = (avg_volatility / 2) as u8; // 波动率贡献 (0-50)
        risk_index += (circuit_breaker_ratio / 2) as u8; // 熔断贡献 (0-50)

        // 额外风险因素
        if total_volume == 0 {
            risk_index += 10; // 交易量过低的额外风险
        }

        risk_index = risk_index.min(100); // 确保不超过100

        risk_index
    }
}

// 初始化风险参数所需的账户
#[derive(Accounts)]
pub struct InitializeRiskParams<'info> {
    #[account(mut)]
    pub market: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<RiskParameters>()
    )]
    pub risk_parameters: Account<'info, RiskParameters>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// 更新风险参数所需的账户
#[derive(Accounts)]
pub struct UpdateRiskParams<'info> {
    #[account(mut)]
    pub market: AccountInfo<'info>,

    #[account(mut)]
    pub risk_parameters: Account<'info, RiskParameters>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

// 在用户交易历史中查找洗盘交易模式
pub fn find_wash_trading_patterns(user_trades: &[UserTrade], threshold_percent: u8) -> bool {
    if user_trades.len() < 10 {
        return false; // 交易记录不足以判断
    }

    // 创建价格-数量映射，检测在同一价格点的买卖行为
    let mut price_actions = HashMap::new();

    for trade in user_trades {
        let key = trade.price;
        let entry = price_actions.entry(key).or_insert((0u64, 0u64)); // (买入量, 卖出量)

        if trade.side == 0 {
            // 买入
            entry.0 += trade.quantity;
        } else {
            // 卖出
            entry.1 += trade.quantity;
        }
    }

    // 检查是否有明显的自买自卖模式
    for (_, (buy_volume, sell_volume)) in price_actions {
        let min_volume = std::cmp::min(buy_volume, sell_volume);
        let max_volume = std::cmp::max(buy_volume, sell_volume);

        if max_volume == 0 {
            continue;
        }

        // 如果买卖差异小于阈值，且数量足够大，视为可疑
        let diff_percent = ((max_volume - min_volume) * 100 / max_volume) as u8;

        if diff_percent < threshold_percent && min_volume > 1000 {
            return true; // 发现可疑模式
        }
    }

    false
}
