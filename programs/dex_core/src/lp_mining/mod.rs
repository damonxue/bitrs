use anchor_lang::prelude::*;

// LP挖矿系统 - 提供流动性挖矿奖励
pub struct LpMining;

// 流动性池
#[account]
#[derive(Default)]
pub struct LiquidityPool {
    pub market: Pubkey,            // 关联的市场
    pub token_a: Pubkey,           // A代币Mint
    pub token_b: Pubkey,           // B代币Mint
    pub token_a_vault: Pubkey,     // A代币金库
    pub token_b_vault: Pubkey,     // B代币金库
    pub lp_mint: Pubkey,           // LP代币Mint
    pub fee_rate: u16,             // 手续费率（基点 - 1bp = 0.01%）
    pub total_value_locked: u64,   // 总锁定价值 (USD)
    pub total_shares: u64,         // 总份额
    pub created_at: i64,           // 创建时间
    pub last_update_ts: i64,       // 最后更新时间戳
}

// 流动性池状态
#[account]
pub struct LiquidityPoolState {
    pub pool: Pubkey,              // 流动性池
    pub a_reserve: u64,            // A代币储备
    pub b_reserve: u64,            // B代币储备
    pub current_price: u64,        // 当前价格 (基于储备计算)
    pub volume_24h: u64,           // 24小时交易量
    pub fees_a_24h: u64,           // 24小时A代币手续费
    pub fees_b_24h: u64,           // 24小时B代币手续费
    pub apr: u16,                  // 年化收益率 (基点)
    pub is_active: bool,           // 是否活跃
    pub daily_data: [DailyData; 7], // 最近7天数据
}

// 每日数据点
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct DailyData {
    pub date: i64,                 // 日期时间戳
    pub volume: u64,               // 交易量
    pub tvl: u64,                  // 总锁定价值
    pub fees: u64,                 // 产生的手续费
}

// 奖励配置
#[account]
pub struct RewardConfig {
    pub admin: Pubkey,             // 管理员
    pub reward_mint: Pubkey,       // 奖励代币Mint
    pub reward_vault: Pubkey,      // 奖励代币金库
    pub reward_rate: u64,          // 每秒奖励率 (每秒发放的奖励代币数量)
    pub reward_duration: u64,      // 奖励持续时间 (秒)
    pub rewards_start_ts: i64,     // 奖励开始时间
    pub rewards_end_ts: i64,       // 奖励结束时间
    pub last_update_ts: i64,       // 最后更新时间戳
    pub reward_per_share: u128,    // 每份额奖励
    pub total_reward_emissions: u64, // 总奖励发放量
}

// 用户流动性位置
#[account]
#[derive(Default)]
pub struct UserPosition {
    pub owner: Pubkey,             // 所有者
    pub pool: Pubkey,              // 流动性池
    pub shares: u64,               // 份额数量
    pub token_a_deposited: u64,    // 存入A代币数量
    pub token_b_deposited: u64,    // 存入B代币数量
    pub reward_debt: u128,         // 奖励债务
    pub reward_claimed: u64,       // 已提取奖励
    pub last_claim_ts: i64,        // 最后提取时间
    pub creation_ts: i64,          // 创建时间
}

// 质押信息
#[account]
pub struct StakingInfo {
    pub owner: Pubkey,             // 所有者
    pub pool: Pubkey,              // 流动性池
    pub staked_amount: u64,        // 质押数量
    pub reward_debt: u128,         // 奖励债务
    pub reward_claimed: u64,       // 已提取奖励
    pub last_update_ts: i64,       // 最后更新时间
    pub lock_period: u64,          // 锁定期 (秒)
    pub unlock_time: i64,          // 解锁时间
    pub boost_factor: u16,         // 提升因子 (基点表示，10000=1倍)
}

impl LpMining {
    // 初始化流动性池
    pub fn initialize_pool(
        pool: &mut LiquidityPool,
        market: Pubkey,
        token_a: Pubkey,
        token_b: Pubkey,
        token_a_vault: Pubkey,
        token_b_vault: Pubkey,
        lp_mint: Pubkey,
        fee_rate: u16,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        pool.market = market;
        pool.token_a = token_a;
        pool.token_b = token_b;
        pool.token_a_vault = token_a_vault;
        pool.token_b_vault = token_b_vault;
        pool.lp_mint = lp_mint;
        pool.fee_rate = fee_rate;
        pool.total_value_locked = 0;
        pool.total_shares = 0;
        pool.created_at = clock.unix_timestamp;
        pool.last_update_ts = clock.unix_timestamp;
        
        Ok(())
    }
    
    // 初始化池状态
    pub fn initialize_pool_state(
        state: &mut LiquidityPoolState,
        pool: Pubkey,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        state.pool = pool;
        state.a_reserve = 0;
        state.b_reserve = 0;
        state.current_price = 0;
        state.volume_24h = 0;
        state.fees_a_24h = 0;
        state.fees_b_24h = 0;
        state.apr = 0;
        state.is_active = true;
        
        // 初始化历史数据
        let day_seconds = 86400;
        let today_start = clock.unix_timestamp - (clock.unix_timestamp % day_seconds);
        
        for i in 0..state.daily_data.len() {
            state.daily_data[i] = DailyData {
                date: today_start - (i as i64 * day_seconds),
                volume: 0,
                tvl: 0,
                fees: 0,
            };
        }
        
        Ok(())
    }
    
    // 初始化奖励配置
    pub fn initialize_reward_config(
        config: &mut RewardConfig,
        admin: Pubkey,
        reward_mint: Pubkey,
        reward_vault: Pubkey,
        reward_rate: u64,
        reward_duration: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        config.admin = admin;
        config.reward_mint = reward_mint;
        config.reward_vault = reward_vault;
        config.reward_rate = reward_rate;
        config.reward_duration = reward_duration;
        config.rewards_start_ts = clock.unix_timestamp;
        config.rewards_end_ts = clock.unix_timestamp + reward_duration as i64;
        config.last_update_ts = clock.unix_timestamp;
        config.reward_per_share = 0;
        config.total_reward_emissions = reward_rate * reward_duration;
        
        Ok(())
    }
    
    // 创建用户流动性位置
    pub fn create_user_position(
        position: &mut UserPosition,
        owner: Pubkey,
        pool: Pubkey,
        shares: u64,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        position.owner = owner;
        position.pool = pool;
        position.shares = shares;
        position.token_a_deposited = token_a_amount;
        position.token_b_deposited = token_b_amount;
        position.reward_debt = 0;
        position.reward_claimed = 0;
        position.last_claim_ts = clock.unix_timestamp;
        position.creation_ts = clock.unix_timestamp;
        
        Ok(())
    }
    
    // 更新用户流动性位置
    pub fn update_user_position(
        position: &mut UserPosition,
        shares_delta: i64,
        token_a_delta: i64,
        token_b_delta: i64,
        reward_config: &RewardConfig,
    ) -> Result<()> {
        // 先计算累积的奖励
        let pending_reward = Self::calculate_pending_reward(position, reward_config)?;
        
        // 更新位置
        if shares_delta > 0 {
            position.shares = position.shares.saturating_add(shares_delta as u64);
        } else if shares_delta < 0 {
            position.shares = position.shares.saturating_sub((-shares_delta) as u64);
        }
        
        if token_a_delta > 0 {
            position.token_a_deposited = position.token_a_deposited.saturating_add(token_a_delta as u64);
        } else if token_a_delta < 0 {
            position.token_a_deposited = position.token_a_deposited.saturating_sub((-token_a_delta) as u64);
        }
        
        if token_b_delta > 0 {
            position.token_b_deposited = position.token_b_deposited.saturating_add(token_b_delta as u64);
        } else if token_b_delta < 0 {
            position.token_b_deposited = position.token_b_deposited.saturating_sub((-token_b_delta) as u64);
        }
        
        // 更新奖励债务
        position.reward_debt = reward_config.reward_per_share
            .saturating_mul(position.shares as u128)
            .checked_div(1_000_000_000_000)
            .unwrap_or(0);
        
        // 如果有待领取的奖励，更新已领取奖励
        if pending_reward > 0 {
            position.reward_claimed = position.reward_claimed.saturating_add(pending_reward);
            position.last_claim_ts = Clock::get()?.unix_timestamp;
        }
        
        Ok(())
    }
    
    // 创建质押信息
    pub fn create_staking_info(
        staking: &mut StakingInfo,
        owner: Pubkey,
        pool: Pubkey,
        staked_amount: u64,
        lock_period: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        staking.owner = owner;
        staking.pool = pool;
        staking.staked_amount = staked_amount;
        staking.reward_debt = 0;
        staking.reward_claimed = 0;
        staking.last_update_ts = clock.unix_timestamp;
        staking.lock_period = lock_period;
        staking.unlock_time = clock.unix_timestamp + lock_period as i64;
        
        // 根据锁定期计算提升因子
        // 例如：30天 = 100%，60天 = 120%，90天 = 150%，180天 = 200%
        let boost_factor = match lock_period {
            p if p >= 15552000 => 20000, // 180天 - 2倍
            p if p >= 7776000 => 15000,  // 90天 - 1.5倍
            p if p >= 5184000 => 12000,  // 60天 - 1.2倍
            _ => 10000,                  // 默认 - 1倍
        };
        
        staking.boost_factor = boost_factor;
        
        Ok(())
    }
    
    // 更新奖励状态
    pub fn update_reward_state(
        config: &mut RewardConfig,
        pool: &LiquidityPool,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;
        
        // 如果没有总份额，仅更新时间戳
        if pool.total_shares == 0 {
            config.last_update_ts = current_time;
            return Ok(());
        }
        
        // 如果奖励已经结束，不更新
        if current_time > config.rewards_end_ts {
            if config.last_update_ts < config.rewards_end_ts {
                // 计算从上次更新到奖励结束期间的奖励
                let time_delta = config.rewards_end_ts - config.last_update_ts;
                
                if time_delta > 0 {
                    let reward = (config.reward_rate as u128)
                        .saturating_mul(time_delta as u128)
                        .saturating_mul(1_000_000_000_000)
                        .checked_div(pool.total_shares as u128)
                        .unwrap_or(0);
                    
                    config.reward_per_share = config.reward_per_share.saturating_add(reward);
                    config.last_update_ts = config.rewards_end_ts;
                }
            }
            return Ok(());
        }
        
        // 计算新增奖励
        let time_delta = current_time - config.last_update_ts;
        
        if time_delta > 0 {
            let reward = (config.reward_rate as u128)
                .saturating_mul(time_delta as u128)
                .saturating_mul(1_000_000_000_000)
                .checked_div(pool.total_shares as u128)
                .unwrap_or(0);
            
            config.reward_per_share = config.reward_per_share.saturating_add(reward);
            config.last_update_ts = current_time;
        }
        
        Ok(())
    }
    
    // 更新池状态
    pub fn update_pool_state(
        state: &mut LiquidityPoolState,
        a_reserve: u64,
        b_reserve: u64,
        volume_delta: u64,
        fees_a_delta: u64,
        fees_b_delta: u64,
        tvl: u64,
        apr: u16,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        // 更新储备和价格
        state.a_reserve = a_reserve;
        state.b_reserve = b_reserve;
        
        // 计算价格 (B/A)
        state.current_price = if a_reserve > 0 {
            (b_reserve as u128)
                .saturating_mul(1_000_000_000)
                .checked_div(a_reserve as u128)
                .unwrap_or(0) as u64
        } else {
            0
        };
        
        // 更新交易量和手续费
        state.volume_24h = volume_delta;
        state.fees_a_24h = fees_a_delta;
        state.fees_b_24h = fees_b_delta;
        
        // 更新APR
        state.apr = apr;
        
        // 更新日数据
        // 获取今天的开始时间
        let day_seconds = 86400;
        let today_start = clock.unix_timestamp - (clock.unix_timestamp % day_seconds);
        
        // 检查是否需要滚动历史数据
        if state.daily_data[0].date != today_start {
            // 滚动历史数据
            for i in (1..state.daily_data.len()).rev() {
                state.daily_data[i] = state.daily_data[i - 1];
            }
            
            // 重置今天的数据
            state.daily_data[0] = DailyData {
                date: today_start,
                volume: 0,
                tvl,
                fees: 0,
            };
        }
        
        // 更新今天的数据
        state.daily_data[0].volume = state.daily_data[0].volume.saturating_add(volume_delta);
        state.daily_data[0].tvl = tvl;
        state.daily_data[0].fees = state.daily_data[0].fees
            .saturating_add(fees_a_delta)
            .saturating_add(fees_b_delta);
        
        Ok(())
    }
    
    // 重置奖励计划
    pub fn reset_reward_schedule(
        config: &mut RewardConfig,
        reward_rate: u64,
        reward_duration: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        // 需要结束当前奖励周期
        config.rewards_end_ts = clock.unix_timestamp;
        
        // 强制更新奖励状态
        config.last_update_ts = clock.unix_timestamp;
        
        // 设置新的奖励计划
        config.reward_rate = reward_rate;
        config.reward_duration = reward_duration;
        config.rewards_start_ts = clock.unix_timestamp;
        config.rewards_end_ts = clock.unix_timestamp + reward_duration as i64;
        config.total_reward_emissions = reward_rate * reward_duration;
        
        Ok(())
    }
    
    // 计算待领取的奖励
    pub fn calculate_pending_reward(
        position: &UserPosition,
        reward_config: &RewardConfig,
    ) -> Result<u64> {
        if position.shares == 0 {
            return Ok(0);
        }
        
        let accumulated_reward = reward_config.reward_per_share
            .saturating_mul(position.shares as u128)
            .checked_div(1_000_000_000_000)
            .unwrap_or(0);
        
        let pending = accumulated_reward.saturating_sub(position.reward_debt);
        
        Ok(pending as u64)
    }
    
    // 计算流动性份额
    pub fn calculate_liquidity_shares(
        pool: &LiquidityPool,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> u64 {
        if pool.total_shares == 0 {
            // 首次流动性，使用几何平均值
            return (token_a_amount as f64 * token_b_amount as f64).sqrt() as u64;
        }
        
        // 获取池中当前储备（这里简化了，实际上需要从池状态获取）
        // 这里假设我们有一个函数来获取当前储备
        let a_reserve = 1_000_000; // 假设值
        let b_reserve = 2_000_000; // 假设值
        
        // 计算要铸造的份额
        let share_a = (token_a_amount as u128)
            .saturating_mul(pool.total_shares as u128)
            .checked_div(a_reserve as u128)
            .unwrap_or(0) as u64;
        
        let share_b = (token_b_amount as u128)
            .saturating_mul(pool.total_shares as u128)
            .checked_div(b_reserve as u128)
            .unwrap_or(0) as u64;
        
        // 使用较小的份额以防止价格操纵
        std::cmp::min(share_a, share_b)
    }
    
    // 计算添加流动性的代币数量
    pub fn calculate_tokens_for_liquidity(
        a_reserve: u64,
        b_reserve: u64,
        token_a_amount: u64,
    ) -> u64 {
        if a_reserve == 0 || b_reserve == 0 {
            return 0;
        }
        
        // 计算相应的B代币数量
        (token_a_amount as u128)
            .saturating_mul(b_reserve as u128)
            .checked_div(a_reserve as u128)
            .unwrap_or(0) as u64
    }
    
    // 计算移除流动性的代币数量
    pub fn calculate_tokens_from_shares(
        pool: &LiquidityPool,
        a_reserve: u64,
        b_reserve: u64,
        shares: u64,
    ) -> (u64, u64) {
        if pool.total_shares == 0 {
            return (0, 0);
        }
        
        // 计算份额占总份额的比例
        let token_a_amount = (shares as u128)
            .saturating_mul(a_reserve as u128)
            .checked_div(pool.total_shares as u128)
            .unwrap_or(0) as u64;
        
        let token_b_amount = (shares as u128)
            .saturating_mul(b_reserve as u128)
            .checked_div(pool.total_shares as u128)
            .unwrap_or(0) as u64;
        
        (token_a_amount, token_b_amount)
    }
    
    // 计算交易数量 (考虑滑点)
    pub fn calculate_swap_exact_in(
        a_reserve: u64,
        b_reserve: u64,
        amount_in: u64,
        fee_rate: u16,
    ) -> u64 {
        if a_reserve == 0 || b_reserve == 0 || amount_in == 0 {
            return 0;
        }
        
        // 计算手续费
        let fee_amount = (amount_in as u128)
            .saturating_mul(fee_rate as u128)
            .checked_div(10000)
            .unwrap_or(0) as u64;
        
        let amount_in_with_fee = amount_in.saturating_sub(fee_amount);
        
        // 使用恒定乘积公式计算输出
        let numerator = (amount_in_with_fee as u128).saturating_mul(b_reserve as u128);
        let denominator = (a_reserve as u128).saturating_add(amount_in_with_fee as u128);
        
        numerator.checked_div(denominator).unwrap_or(0) as u64
    }
    
    // 计算交易所需的输入数量
    pub fn calculate_swap_exact_out(
        a_reserve: u64,
        b_reserve: u64,
        amount_out: u64,
        fee_rate: u16,
    ) -> u64 {
        if a_reserve == 0 || b_reserve == 0 || amount_out >= b_reserve {
            return 0;
        }
        
        // 使用恒定乘积公式计算所需输入
        let numerator = (a_reserve as u128)
            .saturating_mul(amount_out as u128)
            .saturating_mul(10000);
        
        let denominator = (b_reserve.saturating_sub(amount_out) as u128)
            .saturating_mul((10000 - fee_rate) as u128);
        
        // 向上取整
        let amount_in = (numerator + denominator - 1)
            .checked_div(denominator)
            .unwrap_or(0) as u64;
        
        amount_in
    }
    
    // 计算年化收益率 (APR)
    pub fn calculate_apr(
        daily_fees: u64,
        total_value_locked: u64,
    ) -> u16 {
        if total_value_locked == 0 {
            return 0;
        }
        
        // 计算日收益率
        let daily_rate = (daily_fees as u128)
            .saturating_mul(10000)
            .checked_div(total_value_locked as u128)
            .unwrap_or(0) as u16;
        
        // 年化 (假设365天)
        daily_rate.saturating_mul(365)
    }
    
    // 计算质押奖励
    pub fn calculate_staking_reward(
        staking: &StakingInfo,
        reward_config: &RewardConfig,
    ) -> Result<u64> {
        if staking.staked_amount == 0 {
            return Ok(0);
        }
        
        // 应用提升因子
        let boosted_amount = (staking.staked_amount as u128)
            .saturating_mul(staking.boost_factor as u128)
            .checked_div(10000)
            .unwrap_or(0) as u64;
        
        // 计算累积奖励
        let accumulated_reward = reward_config.reward_per_share
            .saturating_mul(boosted_amount as u128)
            .checked_div(1_000_000_000_000)
            .unwrap_or(0);
        
        // 计算待领取奖励
        let pending = accumulated_reward.saturating_sub(staking.reward_debt);
        
        Ok(pending as u64)
    }
}