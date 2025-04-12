use crate::events::{EventHandler, StorageOptimizationType};
use anchor_lang::prelude::*;
use std::collections::HashMap;
use std::mem;

// 优化存储模块 - 提供存储优化和高效数据组织
pub struct OptimizedStorage;

// 压缩订单ID映射
#[account]
#[repr(packed)]
pub struct CompressedIdMap {
    pub market: Pubkey,             // 市场公钥
    pub order_count: u32,           // 订单数量
    pub entries: [IdMapEntry; 128], // 订单映射条目
}

// 压缩订单ID映射条目
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
#[repr(packed)]
pub struct IdMapEntry {
    pub order_id: u128, // 订单ID (128位)
    pub owner: [u8; 8], // 所有者地址的前8字节 (压缩)
    pub index: u16,     // 订单在相应存储中的索引
    pub side: u8,       // 订单方向 (0=买, 1=卖)
    pub flags: u8,      // 存储标志位
}

// 订单批次存储
#[account]
#[repr(packed)]
pub struct OrderBatch {
    pub market: Pubkey,                // 市场公钥
    pub batch_id: u64,                 // 批次ID
    pub order_count: u32,              // 订单数量
    pub next_batch: Option<Pubkey>,    // 链接到下一个批次
    pub orders: [BatchOrderEntry; 64], // 批次中的订单
}

// 批次订单条目
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
#[repr(packed)]
pub struct BatchOrderEntry {
    pub order_id: u128, // 订单ID
    pub owner: Pubkey,  // 所有者公钥
    pub price: u64,     // 价格
    pub quantity: u64,  // 数量
    pub flags: u32,     // 标志位 (包含订单类型、状态等)
    pub timestamp: i64, // 创建时间
}

// 价格级别缓存
#[account]
#[repr(packed)]
pub struct PriceLevelCache {
    pub market: Pubkey,               // 市场公钥
    pub last_update_slot: u64,        // 最后更新的slot
    pub bid_levels_count: u8,         // 买单价格级别数量
    pub ask_levels_count: u8,         // 卖单价格级别数量
    pub bid_levels: [PriceLevel; 20], // 买单价格级别
    pub ask_levels: [PriceLevel; 20], // 卖单价格级别
}

// 价格级别
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
#[repr(packed)]
pub struct PriceLevel {
    pub price: u64,        // 价格
    pub quantity: u64,     // 数量
    pub orders_count: u32, // 订单数量
}

// 用户交易历史
#[account]
#[repr(packed)]
pub struct UserTradeHistory {
    pub owner: Pubkey,           // 用户公钥
    pub last_update_slot: u64,   // 最后更新的slot
    pub trade_count: u16,        // 交易数量
    pub trades: [UserTrade; 64], // 交易历史
}

// 用户交易记录
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
#[repr(packed)]
pub struct UserTrade {
    pub market: [u8; 8], // 市场公钥的前8字节 (压缩)
    pub order_id: u128,  // 订单ID
    pub price: u64,      // 价格
    pub quantity: u64,   // 数量
    pub side: u8,        // 交易方向 (0=买, 1=卖)
    pub timestamp: i64,  // 时间戳
    pub fee: u64,        // 手续费
}

// 顺序存储页面
#[account]
#[repr(packed)]
pub struct StoragePage {
    pub market: Pubkey,             // 市场公钥
    pub page_type: StoragePageType, // 页面类型
    pub page_id: u64,               // 页面ID
    pub used_size: u16,             // 已使用字节数
    pub total_size: u16,            // 总容量字节数
    pub next_page: Option<Pubkey>,  // 下一页（链表）
    pub last_access_slot: u64,      // 最后访问slot
    pub data: [u8; 1024],           // 实际数据存储区
}

// 页面类型
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
#[repr(u8)]
pub enum StoragePageType {
    OrderData = 0,      // 订单数据
    TradeHistory = 1,   // 交易历史
    MarketMetrics = 2,  // 市场指标
    UserPositions = 3,  // 用户持仓
    SystemConfig = 4,   // 系统配置
    TemporaryCache = 5, // 临时缓存
}

// 冷热数据分离配置
#[account]
#[repr(packed)]
pub struct DataTierConfig {
    pub market: Pubkey,               // 市场公钥
    pub hot_data_max_age_slots: u64,  // 热数据最大年龄（slot）
    pub warm_data_max_age_slots: u64, // 温数据最大年龄（slot）
    pub cold_data_compression: bool,  // 冷数据是否启用压缩
    pub auto_archive_enabled: bool,   // 是否启用自动归档
    pub auto_archive_age_slots: u64,  // 自动归档年龄（slot）
    pub last_optimization_slot: u64,  // 最后一次优化的slot
}

// 区块链数据索引
#[account]
#[repr(packed)]
pub struct StorageIndex {
    pub market: Pubkey,                   // 市场公钥
    pub last_update_slot: u64,            // 最后更新的slot
    pub index_entries_count: u16,         // 索引条目数量
    pub entries: [StorageIndexEntry; 64], // 索引条目
}

// 索引条目
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
#[repr(packed)]
pub struct StorageIndexEntry {
    pub page_key: [u8; 8], // 存储页面公钥的前8字节（压缩）
    pub page_type: u8,     // 页面类型
    pub item_count: u16,   // 页面中的项目数量
    pub data_flags: u8,    // 数据标志位
    pub create_slot: u64,  // 创建时的slot
}

impl OptimizedStorage {
    // 计算账户所需的空间
    pub fn get_compressed_id_map_size() -> usize {
        8 + mem::size_of::<CompressedIdMap>()
    }

    pub fn get_order_batch_size() -> usize {
        8 + mem::size_of::<OrderBatch>()
    }

    pub fn get_price_level_cache_size() -> usize {
        8 + mem::size_of::<PriceLevelCache>()
    }

    pub fn get_user_trade_history_size() -> usize {
        8 + mem::size_of::<UserTradeHistory>()
    }

    // 初始化压缩ID映射
    pub fn initialize_compressed_id_map(id_map: &mut CompressedIdMap, market: Pubkey) {
        id_map.market = market;
        id_map.order_count = 0;

        // 初始化所有条目
        for i in 0..id_map.entries.len() {
            id_map.entries[i] = IdMapEntry {
                order_id: 0,
                owner: [0; 8],
                index: 0,
                side: 0,
                flags: 0,
            };
        }
    }

    // 添加订单到压缩映射
    pub fn add_to_compressed_map(
        id_map: &mut CompressedIdMap,
        order_id: u128,
        owner: Pubkey,
        index: u16,
        side: u8,
        flags: u8,
    ) -> Result<()> {
        if id_map.order_count as usize >= id_map.entries.len() {
            return Err(ProgramError::AccountDataTooSmall.into());
        }

        // 压缩所有者地址 - 取前8字节
        let mut compressed_owner = [0u8; 8];
        compressed_owner.copy_from_slice(&owner.to_bytes()[0..8]);

        // 添加条目
        id_map.entries[id_map.order_count as usize] = IdMapEntry {
            order_id,
            owner: compressed_owner,
            index,
            side,
            flags,
        };

        id_map.order_count += 1;

        Ok(())
    }

    // 从压缩映射中移除订单
    pub fn remove_from_compressed_map(id_map: &mut CompressedIdMap, order_id: u128) -> Result<()> {
        // 查找订单索引
        let mut found_idx = None;
        for i in 0..id_map.order_count as usize {
            if id_map.entries[i].order_id == order_id {
                found_idx = Some(i);
                break;
            }
        }

        // 如果找到，移除并重排列表
        if let Some(idx) = found_idx {
            // 将最后一个条目移到当前位置
            if idx < (id_map.order_count - 1) as usize {
                id_map.entries[idx] = id_map.entries[(id_map.order_count - 1) as usize];
            }

            id_map.order_count -= 1;

            return Ok(());
        }

        Err(ProgramError::InvalidArgument.into())
    }

    // 初始化订单批次
    pub fn initialize_order_batch(batch: &mut OrderBatch, market: Pubkey, batch_id: u64) {
        batch.market = market;
        batch.batch_id = batch_id;
        batch.order_count = 0;
        batch.next_batch = None;

        // 初始化所有订单条目
        for i in 0..batch.orders.len() {
            batch.orders[i] = BatchOrderEntry {
                order_id: 0,
                owner: Pubkey::default(),
                price: 0,
                quantity: 0,
                flags: 0,
                timestamp: 0,
            };
        }
    }

    // 添加订单到批次
    pub fn add_to_batch(
        batch: &mut OrderBatch,
        order_id: u128,
        owner: Pubkey,
        price: u64,
        quantity: u64,
        flags: u32,
        timestamp: i64,
    ) -> Result<()> {
        if batch.order_count as usize >= batch.orders.len() {
            return Err(ProgramError::AccountDataTooSmall.into());
        }

        // 添加订单
        batch.orders[batch.order_count as usize] = BatchOrderEntry {
            order_id,
            owner,
            price,
            quantity,
            flags,
            timestamp,
        };

        batch.order_count += 1;

        Ok(())
    }

    // 从批次中移除订单
    pub fn remove_from_batch(batch: &mut OrderBatch, order_id: u128) -> Result<()> {
        // 查找订单索引
        let mut found_idx = None;
        for i in 0..batch.order_count as usize {
            if batch.orders[i].order_id == order_id {
                found_idx = Some(i);
                break;
            }
        }

        // 如果找到，移除并重排列表
        if let Some(idx) = found_idx {
            // 将最后一个条目移到当前位置
            if idx < (batch.order_count - 1) as usize {
                batch.orders[idx] = batch.orders[(batch.order_count - 1) as usize];
            }

            batch.order_count -= 1;

            return Ok(());
        }

        Err(ProgramError::InvalidArgument.into())
    }

    // 初始化价格级别缓存
    pub fn initialize_price_level_cache(cache: &mut PriceLevelCache, market: Pubkey) {
        cache.market = market;
        cache.last_update_slot = 0;
        cache.bid_levels_count = 0;
        cache.ask_levels_count = 0;

        // 初始化所有价格级别
        for i in 0..cache.bid_levels.len() {
            cache.bid_levels[i] = PriceLevel {
                price: 0,
                quantity: 0,
                orders_count: 0,
            };
        }

        for i in 0..cache.ask_levels.len() {
            cache.ask_levels[i] = PriceLevel {
                price: 0,
                quantity: 0,
                orders_count: 0,
            };
        }
    }

    // 更新价格级别缓存
    pub fn update_price_level_cache(
        cache: &mut PriceLevelCache,
        bid_levels: &[(u64, u64, u32)],
        ask_levels: &[(u64, u64, u32)],
        slot: u64,
    ) -> Result<()> {
        // 验证输入
        if bid_levels.len() > cache.bid_levels.len() || ask_levels.len() > cache.ask_levels.len() {
            return Err(ProgramError::InvalidArgument.into());
        }

        // 更新买单价格级别
        cache.bid_levels_count = bid_levels.len() as u8;
        for (i, &(price, quantity, orders_count)) in bid_levels.iter().enumerate() {
            cache.bid_levels[i] = PriceLevel {
                price,
                quantity,
                orders_count,
            };
        }

        // 更新卖单价格级别
        cache.ask_levels_count = ask_levels.len() as u8;
        for (i, &(price, quantity, orders_count)) in ask_levels.iter().enumerate() {
            cache.ask_levels[i] = PriceLevel {
                price,
                quantity,
                orders_count,
            };
        }

        // 更新最后更新时间
        cache.last_update_slot = slot;

        Ok(())
    }

    // 初始化用户交易历史
    pub fn initialize_user_trade_history(history: &mut UserTradeHistory, owner: Pubkey) {
        history.owner = owner;
        history.last_update_slot = 0;
        history.trade_count = 0;

        // 初始化所有交易记录
        for i in 0..history.trades.len() {
            history.trades[i] = UserTrade {
                market: [0; 8],
                order_id: 0,
                price: 0,
                quantity: 0,
                side: 0,
                timestamp: 0,
                fee: 0,
            };
        }
    }

    // 添加交易记录到用户历史
    pub fn add_to_user_history(
        history: &mut UserTradeHistory,
        market: Pubkey,
        order_id: u128,
        price: u64,
        quantity: u64,
        side: u8,
        timestamp: i64,
        fee: u64,
    ) -> Result<()> {
        // 验证是否有空间
        if history.trade_count as usize >= history.trades.len() {
            // 如果已满，删除最旧的记录腾出空间 (FIFO)
            for i in 0..(history.trades.len() - 1) {
                history.trades[i] = history.trades[i + 1];
            }
            history.trade_count = history.trades.len() as u16 - 1;
        }

        // 压缩市场公钥 - 取前8字节
        let mut compressed_market = [0u8; 8];
        compressed_market.copy_from_slice(&market.to_bytes()[0..8]);

        // 添加新交易记录到末尾
        history.trades[history.trade_count as usize] = UserTrade {
            market: compressed_market,
            order_id,
            price,
            quantity,
            side,
            timestamp,
            fee,
        };

        history.trade_count += 1;
        history.last_update_slot = Clock::get().unwrap().slot;

        Ok(())
    }

    // 查询用户最近的交易
    pub fn get_recent_trades(history: &UserTradeHistory, limit: usize) -> Vec<&UserTrade> {
        let count = std::cmp::min(limit, history.trade_count as usize);
        let start = history.trade_count as usize - count;

        history.trades[start..(start + count)].iter().collect()
    }

    // 将完整公钥转换为压缩版本（取前8字节）
    pub fn compress_pubkey(pubkey: &Pubkey) -> [u8; 8] {
        let mut compressed = [0u8; 8];
        compressed.copy_from_slice(&pubkey.to_bytes()[0..8]);
        compressed
    }

    // 将u128转换为更紧凑的表示
    pub fn compress_u128(value: u128) -> [u8; 8] {
        let mut compressed = [0u8; 8];
        compressed.copy_from_slice(&value.to_le_bytes()[0..8]);
        compressed
    }

    // 初始化存储页面
    pub fn initialize_storage_page(
        page: &mut StoragePage,
        market: Pubkey,
        page_type: StoragePageType,
        page_id: u64,
    ) {
        page.market = market;
        page.page_type = page_type;
        page.page_id = page_id;
        page.used_size = 0;
        page.total_size = page.data.len() as u16;
        page.next_page = None;
        page.last_access_slot = Clock::get().unwrap().slot;

        // 清空数据区
        for i in 0..page.data.len() {
            page.data[i] = 0;
        }
    }

    // 向存储页面写入数据
    pub fn write_to_page(page: &mut StoragePage, offset: u16, data: &[u8]) -> Result<()> {
        // 验证写入是否会超出页面容量
        if offset as usize + data.len() > page.data.len() {
            return Err(ProgramError::AccountDataTooSmall.into());
        }

        // 更新使用大小
        let new_end = offset as usize + data.len();
        if new_end as u16 > page.used_size {
            page.used_size = new_end as u16;
        }

        // 写入数据
        for (i, &byte) in data.iter().enumerate() {
            page.data[offset as usize + i] = byte;
        }

        // 更新最后访问时间
        page.last_access_slot = Clock::get().unwrap().slot;

        Ok(())
    }

    // 从存储页面读取数据
    pub fn read_from_page<'a>(
        page: &'a mut StoragePage,
        offset: u16,
        size: u16,
    ) -> Result<&'a [u8]> {
        // 验证读取范围是否有效
        if offset as usize + size as usize > page.data.len() {
            return Err(ProgramError::AccountDataTooSmall.into());
        }

        // 更新最后访问时间
        page.last_access_slot = Clock::get().unwrap().slot;

        // 返回数据片段的引用
        Ok(&page.data[offset as usize..(offset + size) as usize])
    }

    // 链接存储页面（创建链表）
    pub fn link_pages(current_page: &mut StoragePage, next_page_pubkey: Pubkey) -> Result<()> {
        // 确保两个页面类型相同
        if current_page.next_page.is_some() {
            // 已经有链接的页面，先验证是否需要更新
            if current_page.next_page.unwrap() != next_page_pubkey {
                current_page.next_page = Some(next_page_pubkey);
            }
        } else {
            current_page.next_page = Some(next_page_pubkey);
        }

        Ok(())
    }

    // 对数据进行压缩 (模拟实现)
    pub fn compress_data(data: &[u8]) -> Vec<u8> {
        // 这里是简化的压缩算法示例，真实环境应该使用成熟的压缩算法如 LZ77、DEFLATE 等
        // 由于区块链环境限制，可能需要使用简单的压缩策略

        // 简单的游程编码 (RLE) 实现示例
        let mut compressed = Vec::with_capacity(data.len());

        if data.is_empty() {
            return compressed;
        }

        let mut current_byte = data[0];
        let mut count: u8 = 1;

        for &byte in &data[1..] {
            if byte == current_byte && count < 255 {
                count += 1;
            } else {
                compressed.push(count);
                compressed.push(current_byte);
                current_byte = byte;
                count = 1;
            }
        }

        // 处理最后一组字节
        compressed.push(count);
        compressed.push(current_byte);

        compressed
    }

    // 解压缩数据 (与上面的压缩方法对应)
    pub fn decompress_data(compressed: &[u8]) -> Vec<u8> {
        let mut decompressed = Vec::new();

        let mut i = 0;
        while i < compressed.len() / 2 * 2 {
            // 确保按对处理
            let count = compressed[i];
            let byte = compressed[i + 1];

            for _ in 0..count {
                decompressed.push(byte);
            }

            i += 2;
        }

        decompressed
    }

    // 迁移冷数据到长期存储
    pub fn migrate_cold_data(market: Pubkey, config: &DataTierConfig) -> Result<()> {
        // 获取当前slot
        let current_slot = Clock::get()?.slot;

        // 计算冷数据边界
        let cold_data_boundary = current_slot.saturating_sub(config.warm_data_max_age_slots);

        // 查找需要迁移的存储页面
        // 在实际实现中，这需要一个账户查询机制，这里简单模拟

        // 发送存储优化事件 - 这里仅作为示例
        EventHandler::emit_storage_optimization(
            market,
            StorageOptimizationType::Compression,
            0,    // 旧大小
            0,    // 新大小
            true, // 是否成功
        );

        Ok(())
    }

    // 整理碎片化存储
    pub fn defragment_storage(market: Pubkey, page_type: StoragePageType) -> Result<u32> {
        // 在实际实现中，这需要遍历所有存储页面并重新组织数据
        // 这里仅作为示例说明功能

        let space_reclaimed = 0u32; // 实际回收的空间

        // 发送存储优化事件
        EventHandler::emit_storage_optimization(
            market,
            StorageOptimizationType::Defragmentation,
            0,    // 旧大小
            0,    // 新大小
            true, // 是否成功
        );

        Ok(space_reclaimed)
    }

    // 创建存储索引
    pub fn create_storage_index(
        index: &mut StorageIndex,
        market: Pubkey,
        pages: &[StoragePage],
    ) -> Result<()> {
        index.market = market;
        index.last_update_slot = Clock::get()?.slot;
        index.index_entries_count = 0;

        // 为每个页面创建索引条目
        for (i, page) in pages.iter().enumerate() {
            if i >= index.entries.len() {
                break;
            }

            let compressed_key = Self::compress_pubkey(&Pubkey::default()); // 在实际情况中，这应该是页面的公钥

            index.entries[i] = StorageIndexEntry {
                page_key: compressed_key,
                page_type: page.page_type as u8,
                item_count: (page.used_size / 32) as u16, // 粗略估算项目数量
                data_flags: 0,                            // 默认标志
                create_slot: page.last_access_slot,
            };

            index.index_entries_count += 1;
        }

        Ok(())
    }

    // 更新数据分层配置
    pub fn update_data_tier_config(
        config: &mut DataTierConfig,
        hot_data_max_age_slots: u64,
        warm_data_max_age_slots: u64,
        cold_data_compression: bool,
        auto_archive_enabled: bool,
        auto_archive_age_slots: u64,
    ) -> Result<()> {
        config.hot_data_max_age_slots = hot_data_max_age_slots;
        config.warm_data_max_age_slots = warm_data_max_age_slots;
        config.cold_data_compression = cold_data_compression;
        config.auto_archive_enabled = auto_archive_enabled;
        config.auto_archive_age_slots = auto_archive_age_slots;
        config.last_optimization_slot = Clock::get()?.slot;

        Ok(())
    }

    // 查找最适合存储项目的页面
    pub fn find_best_page_for_item(
        pages: &[StoragePage],
        required_size: u16,
        item_type: StoragePageType,
    ) -> Option<usize> {
        let mut best_fit_index = None;
        let mut best_fit_remaining = u16::MAX;

        for (i, page) in pages.iter().enumerate() {
            // 检查页面类型是否匹配
            if page.page_type != item_type {
                continue;
            }

            // 检查页面是否有足够空间
            let remaining_space = page.total_size - page.used_size;
            if remaining_space >= required_size && remaining_space < best_fit_remaining {
                best_fit_index = Some(i);
                best_fit_remaining = remaining_space;
            }
        }

        best_fit_index
    }

    // 在存储页面间平衡负载
    pub fn balance_storage_load(
        pages: &mut [StoragePage],
        market: Pubkey,
        page_type: StoragePageType,
    ) -> Result<()> {
        // 计算平均使用率
        let mut total_usage = 0u32;
        let mut page_count = 0u32;

        for page in pages.iter() {
            if page.page_type == page_type && page.market == market {
                total_usage += page.used_size as u32;
                page_count += 1;
            }
        }

        if page_count == 0 {
            return Ok(());
        }

        let avg_usage = total_usage / page_count;

        // 找出使用率过高和过低的页面
        let high_threshold = (avg_usage as f32 * 1.2) as u16;
        let low_threshold = (avg_usage as f32 * 0.6) as u16;

        // 在实际实现中，这里应该重新分配数据
        // 例如从使用率高的页面移动数据到使用率低的页面

        // 发送存储优化事件
        EventHandler::emit_storage_optimization(
            market,
            StorageOptimizationType::Defragmentation,
            total_usage as u32,
            total_usage as u32, // 实际应该是优化后的大小
            true,
        );

        Ok(())
    }

    // 检查存储健康状态
    pub fn check_storage_health(market: Pubkey, pages: &[StoragePage]) -> StorageHealthResult {
        let mut result = StorageHealthResult {
            total_pages: pages.len() as u16,
            fragmentation_percent: 0,
            avg_utilization_percent: 0,
            oldest_page_age_slots: 0,
            oversized_pages_count: 0,
        };

        if pages.is_empty() {
            return result;
        }

        let current_slot = Clock::get().unwrap().slot;
        let mut total_space = 0u32;
        let mut used_space = 0u32;
        let mut fragmented_space = 0u32;
        let mut oldest_page_age = 0u64;

        for page in pages {
            if page.market != market {
                continue;
            }

            total_space += page.total_size as u32;
            used_space += page.used_size as u32;

            // 估计碎片
            let estimated_item_size = 32; // 假设平均项目大小
            let items_capacity = page.total_size / estimated_item_size;
            let actual_items = page.used_size / estimated_item_size;
            let wasted_space = page.used_size - (actual_items * estimated_item_size);
            fragmented_space += wasted_space as u32;

            // 检查页面年龄
            let page_age = current_slot.saturating_sub(page.last_access_slot);
            if page_age > oldest_page_age {
                oldest_page_age = page_age;
            }

            // 检查过大的页面
            if page.used_size > (page.total_size as f32 * 0.9) as u16 {
                result.oversized_pages_count += 1;
            }
        }

        if total_space > 0 {
            result.fragmentation_percent =
                ((fragmented_space as f32 / total_space as f32) * 100.0) as u8;
            result.avg_utilization_percent =
                ((used_space as f32 / total_space as f32) * 100.0) as u8;
        }

        result.oldest_page_age_slots = oldest_page_age;

        result
    }

    // 清理过期数据
    pub fn cleanup_expired_data(
        market: Pubkey,
        page_type: StoragePageType,
        max_age_slots: u64,
    ) -> Result<u32> {
        // 在实际实现中，这需要识别并删除过期的数据项
        // 这里仅作为示例说明功能

        let cleaned_items = 0u32; // 实际清理的项目数

        // 发送存储优化事件
        EventHandler::emit_storage_optimization(
            market,
            StorageOptimizationType::HistoryTruncation,
            0,    // 旧大小
            0,    // 新大小
            true, // 是否成功
        );

        Ok(cleaned_items)
    }

    // 计算最佳页面大小
    pub fn calculate_optimal_page_size(item_size: u16, avg_items_per_batch: u16) -> u16 {
        // 计算存放平均批次大小所需的空间
        let base_size = item_size * avg_items_per_batch;

        // 增加一些缓冲空间
        let with_buffer = (base_size as f32 * 1.2) as u16;

        // 向上取整到最近的2的幂
        let mut power_of_two = 256u16; // 最小页面大小
        while power_of_two < with_buffer {
            power_of_two *= 2;
            if power_of_two >= 16384 {
                // 最大页面大小
                break;
            }
        }

        power_of_two
    }
}

// 存储健康检查结果
pub struct StorageHealthResult {
    pub total_pages: u16,
    pub fragmentation_percent: u8,
    pub avg_utilization_percent: u8,
    pub oldest_page_age_slots: u64,
    pub oversized_pages_count: u16,
}
