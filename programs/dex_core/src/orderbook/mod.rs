use crate::core::SelfTradeBehavior;
use crate::ErrorCode;
use anchor_lang::prelude::*;
use std::cmp;

// 订单方向枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub enum Side {
    Bid, // 买单
    Ask, // 卖单
}

// 订单类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub enum OrderType {
    Limit,             // 限价单
    Market,            // 市价单
    PostOnly,          // 只做挂单
    ImmediateOrCancel, // 立即成交或取消
    FillOrKill,        // 完全成交或取消
}

// 订单结构
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Order {
    pub order_id: u128,                                 // 订单ID
    pub owner: Pubkey,                                  // 订单所有者
    pub side: Side,                                     // 订单方向
    pub price: u64,                                     // 价格
    pub quantity: u64,                                  // 总量
    pub remaining_quantity: u64,                        // 剩余量
    pub order_type: OrderType,                          // 订单类型
    pub timestamp: i64,                                 // 时间戳
    pub client_order_id: u64,                           // 客户端订单ID
    pub max_ts_valid: i64,                              // 最大有效时间戳 (0表示永不过期)
    pub self_trade_behavior: Option<SelfTradeBehavior>, // 自成交行为
}

impl Order {
    pub fn new(
        order_id: u128,
        owner: Pubkey,
        side: Side,
        price: u64,
        quantity: u64,
        order_type: OrderType,
    ) -> Self {
        Self {
            order_id,
            owner,
            side,
            price,
            quantity,
            remaining_quantity: quantity,
            order_type,
            timestamp: Clock::get().unwrap().unix_timestamp,
            client_order_id: 0,
            max_ts_valid: 0, // 默认永不过期
            self_trade_behavior: None,
        }
    }

    // 检查订单是否已过期
    pub fn is_expired(&self, current_ts: i64) -> bool {
        self.max_ts_valid > 0 && current_ts > self.max_ts_valid
    }
}

// 交易结构
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Trade {
    pub maker_order_id: u128, // 挂单方订单ID
    pub taker_order_id: u128, // 吃单方订单ID
    pub price: u64,           // 成交价格
    pub quantity: u64,        // 成交数量
    pub maker: Pubkey,        // 挂单方
    pub base_quantity: u64,   // 基础代币数量
    pub quote_quantity: u64,  // 报价代币数量
    pub maker_fee: u64,       // 挂单方手续费
    pub taker_fee: u64,       // 吃单方手续费
    pub timestamp: i64,       // 时间戳
}

// 订单簿结构 - 零拷贝优化
#[account(zero_copy)]
#[repr(packed)]
pub struct OrderBook {
    pub market: Pubkey, // 所属市场

    // 存储价格树的根节点索引
    pub bid_price_tree_root: u32, // 买单价格树根节点
    pub ask_price_tree_root: u32, // 卖单价格树根节点

    // 价格节点和订单节点数组
    pub price_nodes: [PriceNode; 256],  // 支持256个不同价格
    pub order_nodes: [OrderNode; 1024], // 支持1024个订单

    // 管理空闲节点的索引
    pub free_price_nodes: [u32; 256],
    pub free_order_nodes: [u32; 1024],
    pub free_price_nodes_count: u32,
    pub free_order_nodes_count: u32,

    // 统计信息
    pub bid_orders_count: u32,
    pub ask_orders_count: u32,
    pub bid_volume: u64,
    pub ask_volume: u64,

    // 最后更新信息
    pub last_update_slot: u64,
    pub last_purge_slot: u64, // 上次清理过期订单的slot
}

// 价格节点 - 表示单个价格层级
#[zero_copy]
#[repr(packed)]
pub struct PriceNode {
    pub price: u64,        // 价格
    pub quantity: u64,     // 该价格下的总数量
    pub orders_count: u32, // 该价格下的订单数量
    pub first_order: u32,  // 指向该价格下第一个订单
    pub parent: u32,       // 父节点索引
    pub left: u32,         // 左子树索引
    pub right: u32,        // 右子树索引
    pub next_price: u32,   // 链表中的下一个价格
    pub prev_price: u32,   // 链表中的上一个价格
}

// 订单节点 - 表示单个订单
#[zero_copy]
#[repr(packed)]
pub struct OrderNode {
    pub order_id: u128,    // 订单ID
    pub owner: Pubkey,     // 所有者
    pub quantity: u64,     // 剩余数量
    pub price_index: u32,  // 所属价格节点索引
    pub next: u32,         // 同一价格下的下一个订单
    pub prev: u32,         // 同一价格下的上一个订单
    pub timestamp: i64,    // 时间戳
    pub max_ts_valid: i64, // 最大有效时间戳
}

impl OrderBook {
    pub const LEN: usize = 8
        + 32
        + 4
        + 4
        + (256 * std::mem::size_of::<PriceNode>())
        + (1024 * std::mem::size_of::<OrderNode>())
        + (256 * 4)
        + (1024 * 4)
        + 4
        + 4
        + 4
        + 4
        + 8
        + 8
        + 8
        + 8;

    // 初始化订单簿
    pub fn initialize(&mut self) {
        // 初始化根节点为无效值
        self.bid_price_tree_root = u32::MAX;
        self.ask_price_tree_root = u32::MAX;

        // 初始化空闲节点索引
        for i in 0..self.free_price_nodes.len() {
            self.free_price_nodes[i] = (self.free_price_nodes.len() - 1 - i) as u32;
        }
        for i in 0..self.free_order_nodes.len() {
            self.free_order_nodes[i] = (self.free_order_nodes.len() - 1 - i) as u32;
        }

        self.free_price_nodes_count = self.free_price_nodes.len() as u32;
        self.free_order_nodes_count = self.free_order_nodes.len() as u32;

        // 初始化统计信息
        self.bid_orders_count = 0;
        self.ask_orders_count = 0;
        self.bid_volume = 0;
        self.ask_volume = 0;

        // 初始化最后更新信息
        self.last_update_slot = 0;
        self.last_purge_slot = 0;
    }

    // 处理订单
    pub fn process_order(
        &mut self,
        order: Order,
        self_trade_behavior: SelfTradeBehavior,
    ) -> Result<(Vec<Trade>, Option<Order>)> {
        // 更新最后更新时间
        self.last_update_slot = Clock::get()?.slot;

        // 定期清理过期订单
        if self.last_update_slot.saturating_sub(self.last_purge_slot) > 100 {
            self.purge_expired_orders()?;
        }

        // 创建可变订单副本
        let mut remaining_order = order.clone();
        let mut trades = Vec::new();

        // 根据订单类型处理
        match order.order_type {
            OrderType::Market => {
                // 市价单总是尝试立即成交，不添加到订单簿
                self.match_order(&mut remaining_order, &mut trades, self_trade_behavior)?;
                // 市价单未成交部分被取消
                if remaining_order.remaining_quantity > 0 {
                    remaining_order.remaining_quantity = 0;
                }
            }
            OrderType::Limit => {
                // 限价单先尝试成交，剩余部分添加到订单簿
                self.match_order(&mut remaining_order, &mut trades, self_trade_behavior)?;
                if remaining_order.remaining_quantity > 0 {
                    // 添加剩余部分到订单簿
                    self.add_order(&mut remaining_order)?;
                }
            }
            OrderType::PostOnly => {
                // PostOnly如果会立即成交则被拒绝
                if self.would_match(remaining_order.side, remaining_order.price) {
                    return Ok((trades, None));
                }
                // 添加到订单簿
                self.add_order(&mut remaining_order)?;
            }
            OrderType::ImmediateOrCancel => {
                // IoC尝试立即成交，未成交部分被取消
                self.match_order(&mut remaining_order, &mut trades, self_trade_behavior)?;
                if remaining_order.remaining_quantity > 0 {
                    remaining_order.remaining_quantity = 0;
                }
            }
            OrderType::FillOrKill => {
                // FoK需要全部成交，否则全部取消
                if !self.can_fill_completely(
                    remaining_order.side,
                    remaining_order.price,
                    remaining_order.remaining_quantity,
                ) {
                    // 不能完全成交，取消订单
                    return Ok((Vec::new(), None));
                }

                // 尝试撮合
                self.match_order(&mut remaining_order, &mut trades, self_trade_behavior)?;

                // 检查是否全部成交
                if remaining_order.remaining_quantity > 0 {
                    // 未能全部成交，清除所有交易并返回取消
                    trades.clear();
                    return Ok((trades, None));
                }
            }
        }

        // 如果订单完全成交，返回None表示没有剩余订单
        if remaining_order.remaining_quantity == 0 {
            Ok((trades, None))
        } else {
            Ok((trades, Some(remaining_order)))
        }
    }

    // 匹配订单
    fn match_order(
        &mut self,
        order: &mut Order,
        trades: &mut Vec<Trade>,
        self_trade_behavior: SelfTradeBehavior,
    ) -> Result<()> {
        let current_ts = Clock::get()?.unix_timestamp;
        let total_quantity = order.remaining_quantity;

        match order.side {
            Side::Bid => {
                self.match_bid_order(order, trades, current_ts, self_trade_behavior)?;
            }
            Side::Ask => {
                self.match_ask_order(order, trades, current_ts, self_trade_behavior)?;
            }
        }

        // 更新统计信息
        let matched_quantity = total_quantity - order.remaining_quantity;
        if matched_quantity > 0 {
            match order.side {
                Side::Bid => {
                    self.ask_volume = self.ask_volume.saturating_sub(matched_quantity);
                }
                Side::Ask => {
                    self.bid_volume = self.bid_volume.saturating_sub(matched_quantity);
                }
            }
        }

        Ok(())
    }

    // 匹配买单
    fn match_bid_order(
        &mut self,
        order: &mut Order,
        trades: &mut Vec<Trade>,
        current_ts: i64,
        self_trade_behavior: SelfTradeBehavior,
    ) -> Result<()> {
        if self.ask_price_tree_root == u32::MAX {
            return Ok(()); // 没有卖单
        }

        // 获取最低价格的卖单
        let mut best_ask_idx = self.find_min_price_node(self.ask_price_tree_root)?;

        while best_ask_idx != u32::MAX && order.remaining_quantity > 0 {
            let best_ask = &self.price_nodes[best_ask_idx as usize];

            // 如果卖单价格高于买单价格，中止匹配
            if best_ask.price > order.price {
                break;
            }

            // 获取该价格下的第一个订单
            let mut order_idx = best_ask.first_order;
            let mut prev_order_idx = u32::MAX;

            while order_idx != u32::MAX && order.remaining_quantity > 0 {
                let order_node = &self.order_nodes[order_idx as usize];

                // 检查订单是否过期
                if order_node.max_ts_valid > 0 && current_ts > order_node.max_ts_valid {
                    // 移除过期订单
                    let next_order_idx = order_node.next;
                    self.remove_order_node(best_ask_idx, order_idx, prev_order_idx)?;
                    order_idx = next_order_idx;
                    continue;
                }

                // 检查是否为自成交
                if order_node.owner == order.owner {
                    match self_trade_behavior {
                        SelfTradeBehavior::DecrementTake => {
                            // 减少吃单数量，不成交
                            order.remaining_quantity = 0;
                            break;
                        }
                        SelfTradeBehavior::CancelProvide => {
                            // 取消挂单
                            let next_order_idx = order_node.next;
                            self.remove_order_node(best_ask_idx, order_idx, prev_order_idx)?;
                            order_idx = next_order_idx;
                            continue;
                        }
                        SelfTradeBehavior::AbortTransaction => {
                            // 中止整个交易
                            return Err(ErrorCode::SelfTrade.into());
                        }
                    }
                }

                // 计算成交数量
                let trade_quantity = std::cmp::min(order.remaining_quantity, order_node.quantity);

                // 创建交易记录
                let trade = Trade {
                    maker_order_id: order_node.order_id,
                    taker_order_id: order.order_id,
                    price: best_ask.price,
                    quantity: trade_quantity,
                    maker: order_node.owner,
                    base_quantity: trade_quantity,
                    quote_quantity: trade_quantity * best_ask.price,
                    maker_fee: 0, // 在外部计算
                    taker_fee: 0, // 在外部计算
                    timestamp: current_ts,
                };

                // 添加到交易列表
                trades.push(trade);

                // 更新订单剩余数量
                order.remaining_quantity -= trade_quantity;

                // 更新或移除卖单
                if trade_quantity >= order_node.quantity {
                    // 卖单完全成交，移除
                    let next_order_idx = order_node.next;
                    self.remove_order_node(best_ask_idx, order_idx, prev_order_idx)?;
                    order_idx = next_order_idx;
                } else {
                    // 卖单部分成交，更新数量
                    let new_quantity = order_node.quantity - trade_quantity;
                    self.update_order_quantity(order_idx, new_quantity)?;

                    // 移动到下一个订单
                    prev_order_idx = order_idx;
                    order_idx = order_node.next;
                }
            }

            // 检查该价格是否还有订单
            if self.price_nodes[best_ask_idx as usize].orders_count == 0 {
                // 没有订单，移除价格节点
                self.remove_price_node(best_ask_idx, Side::Ask)?;

                // 获取新的最低价格卖单
                if self.ask_price_tree_root != u32::MAX {
                    best_ask_idx = self.find_min_price_node(self.ask_price_tree_root)?;
                } else {
                    break; // 没有更多卖单
                }
            } else {
                // 移动到下一个价格节点
                best_ask_idx = self.price_nodes[best_ask_idx as usize].next_price;
            }
        }

        Ok(())
    }

    // 匹配卖单
    fn match_ask_order(
        &mut self,
        order: &mut Order,
        trades: &mut Vec<Trade>,
        current_ts: i64,
        self_trade_behavior: SelfTradeBehavior,
    ) -> Result<()> {
        if self.bid_price_tree_root == u32::MAX {
            return Ok(()); // 没有买单
        }

        // 获取最高价格的买单
        let mut best_bid_idx = self.find_max_price_node(self.bid_price_tree_root)?;

        while best_bid_idx != u32::MAX && order.remaining_quantity > 0 {
            let best_bid = &self.price_nodes[best_bid_idx as usize];

            // 如果买单价格低于卖单价格，中止匹配
            if best_bid.price < order.price {
                break;
            }

            // 获取该价格下的第一个订单
            let mut order_idx = best_bid.first_order;
            let mut prev_order_idx = u32::MAX;

            while order_idx != u32::MAX && order.remaining_quantity > 0 {
                let order_node = &self.order_nodes[order_idx as usize];

                // 检查订单是否过期
                if order_node.max_ts_valid > 0 && current_ts > order_node.max_ts_valid {
                    // 移除过期订单
                    let next_order_idx = order_node.next;
                    self.remove_order_node(best_bid_idx, order_idx, prev_order_idx)?;
                    order_idx = next_order_idx;
                    continue;
                }

                // 检查是否为自成交
                if order_node.owner == order.owner {
                    match self_trade_behavior {
                        SelfTradeBehavior::DecrementTake => {
                            // 减少吃单数量，不成交
                            order.remaining_quantity = 0;
                            break;
                        }
                        SelfTradeBehavior::CancelProvide => {
                            // 取消挂单
                            let next_order_idx = order_node.next;
                            self.remove_order_node(best_bid_idx, order_idx, prev_order_idx)?;
                            order_idx = next_order_idx;
                            continue;
                        }
                        SelfTradeBehavior::AbortTransaction => {
                            // 中止整个交易
                            return Err(ErrorCode::SelfTrade.into());
                        }
                    }
                }

                // 计算成交数量
                let trade_quantity = std::cmp::min(order.remaining_quantity, order_node.quantity);

                // 创建交易记录
                let trade = Trade {
                    maker_order_id: order_node.order_id,
                    taker_order_id: order.order_id,
                    price: best_bid.price,
                    quantity: trade_quantity,
                    maker: order_node.owner,
                    base_quantity: trade_quantity,
                    quote_quantity: trade_quantity * best_bid.price,
                    maker_fee: 0, // 在外部计算
                    taker_fee: 0, // 在外部计算
                    timestamp: current_ts,
                };

                // 添加到交易列表
                trades.push(trade);

                // 更新订单剩余数量
                order.remaining_quantity -= trade_quantity;

                // 更新或移除买单
                if trade_quantity >= order_node.quantity {
                    // 买单完全成交，移除
                    let next_order_idx = order_node.next;
                    self.remove_order_node(best_bid_idx, order_idx, prev_order_idx)?;
                    order_idx = next_order_idx;
                } else {
                    // 买单部分成交，更新数量
                    let new_quantity = order_node.quantity - trade_quantity;
                    self.update_order_quantity(order_idx, new_quantity)?;

                    // 移动到下一个订单
                    prev_order_idx = order_idx;
                    order_idx = order_node.next;
                }
            }

            // 检查该价格是否还有订单
            if self.price_nodes[best_bid_idx as usize].orders_count == 0 {
                // 没有订单，移除价格节点
                self.remove_price_node(best_bid_idx, Side::Bid)?;

                // 获取新的最高价格买单
                if self.bid_price_tree_root != u32::MAX {
                    best_bid_idx = self.find_max_price_node(self.bid_price_tree_root)?;
                } else {
                    break; // 没有更多买单
                }
            } else {
                // 移动到下一个价格节点
                best_bid_idx = self.price_nodes[best_bid_idx as usize].prev_price;
            }
        }

        Ok(())
    }

    // 添加订单到订单簿
    fn add_order(&mut self, order: &mut Order) -> Result<()> {
        // 检查空闲节点是否足够
        if self.free_order_nodes_count == 0 {
            return Err(ErrorCode::OrderBookFull.into());
        }

        // 分配订单节点
        let order_idx = self.allocate_order_node()?;
        let order_node = &mut self.order_nodes[order_idx as usize];
        order_node.order_id = order.order_id;
        order_node.owner = order.owner;
        order_node.quantity = order.remaining_quantity;
        order_node.timestamp = order.timestamp;
        order_node.max_ts_valid = order.max_ts_valid;
        order_node.next = u32::MAX;
        order_node.prev = u32::MAX;

        // 查找或创建价格节点
        let price_idx = self.find_or_create_price_node(order.price, order.side)?;

        // 更新订单节点的价格索引
        order_node.price_index = price_idx;

        // 将订单节点添加到价格节点的订单链表中 (按时间优先)
        let price_node = &mut self.price_nodes[price_idx as usize];
        if price_node.first_order == u32::MAX {
            // 第一个订单
            price_node.first_order = order_idx;
        } else {
            // 添加到链表头部
            let first_order_idx = price_node.first_order;
            order_node.next = first_order_idx;
            self.order_nodes[first_order_idx as usize].prev = order_idx;
            price_node.first_order = order_idx;
        }

        // 更新价格节点统计
        price_node.quantity += order.remaining_quantity;
        price_node.orders_count += 1;

        // 更新全局统计
        match order.side {
            Side::Bid => {
                self.bid_orders_count += 1;
                self.bid_volume += order.remaining_quantity;
            }
            Side::Ask => {
                self.ask_orders_count += 1;
                self.ask_volume += order.remaining_quantity;
            }
        }

        Ok(())
    }

    // 检查是否可以完全成交
    pub fn can_fill_completely(&self, side: Side, price: u64, quantity: u64) -> bool {
        let mut available_quantity = 0;

        match side {
            Side::Bid => {
                if self.ask_price_tree_root == u32::MAX {
                    return false; // 没有卖单
                }

                // 获取最低价格的卖单
                let mut price_idx = match self.find_min_price_node(self.ask_price_tree_root) {
                    Ok(idx) => idx,
                    Err(_) => return false,
                };

                // 遍历所有卖单价格
                while price_idx != u32::MAX {
                    let price_node = &self.price_nodes[price_idx as usize];

                    // 如果卖单价格高于买单价格，终止遍历
                    if price_node.price > price {
                        break;
                    }

                    // 累加该价格下的所有订单数量
                    available_quantity += price_node.quantity;

                    // 检查是否已经足够
                    if available_quantity >= quantity {
                        return true;
                    }

                    // 移动到下一个价格节点
                    price_idx = price_node.next_price;
                }
            }
            Side::Ask => {
                if self.bid_price_tree_root == u32::MAX {
                    return false; // 没有买单
                }

                // 获取最高价格的买单
                let mut price_idx = match self.find_max_price_node(self.bid_price_tree_root) {
                    Ok(idx) => idx,
                    Err(_) => return false,
                };

                // 遍历所有买单价格
                while price_idx != u32::MAX {
                    let price_node = &self.price_nodes[price_idx as usize];

                    // 如果买单价格低于卖单价格，终止遍历
                    if price_node.price < price {
                        break;
                    }

                    // 累加该价格下的所有订单数量
                    available_quantity += price_node.quantity;

                    // 检查是否已经足够
                    if available_quantity >= quantity {
                        return true;
                    }

                    // 移动到下一个价格节点
                    price_idx = price_node.prev_price;
                }
            }
        }

        false
    }

    // 检查是否会立即成交
    pub fn would_match(&self, side: Side, price: u64) -> bool {
        match side {
            Side::Bid => {
                if self.ask_price_tree_root == u32::MAX {
                    return false; // 没有卖单
                }

                // 获取最低价格的卖单
                match self.find_min_price_node(self.ask_price_tree_root) {
                    Ok(idx) => {
                        let min_ask_price = self.price_nodes[idx as usize].price;
                        min_ask_price <= price // 如果买单价格高于或等于最低卖单价格，则会成交
                    }
                    Err(_) => false,
                }
            }
            Side::Ask => {
                if self.bid_price_tree_root == u32::MAX {
                    return false; // 没有买单
                }

                // 获取最高价格的买单
                match self.find_max_price_node(self.bid_price_tree_root) {
                    Ok(idx) => {
                        let max_bid_price = self.price_nodes[idx as usize].price;
                        max_bid_price >= price // 如果卖单价格低于或等于最高买单价格，则会成交
                    }
                    Err(_) => false,
                }
            }
        }
    }

    // 取消订单
    pub fn cancel_order(&mut self, order_id: u128, side: Side) -> Result<Order> {
        // 更新最后更新时间
        self.last_update_slot = Clock::get()?.slot;

        // 根据订单方向选择价格树根节点
        let root_idx = match side {
            Side::Bid => self.bid_price_tree_root,
            Side::Ask => self.ask_price_tree_root,
        };

        if root_idx == u32::MAX {
            return Err(ErrorCode::OrderNotFound.into());
        }

        // 遍历价格树查找订单
        let mut found = false;
        let mut price_idx = u32::MAX;
        let mut order_idx = u32::MAX;
        let mut prev_order_idx = u32::MAX;
        let mut price = 0;
        let mut quantity = 0;
        let mut owner = Pubkey::default();

        // 查找订单的函数
        self.find_order(
            root_idx,
            order_id,
            &mut found,
            &mut price_idx,
            &mut order_idx,
            &mut prev_order_idx,
            &mut price,
            &mut quantity,
            &mut owner,
        )?;

        if !found {
            return Err(ErrorCode::OrderNotFound.into());
        }

        // 创建订单对象
        let order = Order::new(
            order_id,
            owner,
            side,
            price,
            quantity,
            OrderType::Limit, // 默认为限价单
        );

        // 从订单簿中移除订单
        self.remove_order_node(price_idx, order_idx, prev_order_idx)?;

        // 检查价格节点是否为空
        if self.price_nodes[price_idx as usize].orders_count == 0 {
            self.remove_price_node(price_idx, side)?;
        }

        Ok(order)
    }

    // 查找订单
    fn find_order(
        &self,
        root_idx: u32,
        order_id: u128,
        found: &mut bool,
        price_idx: &mut u32,
        order_idx: &mut u32,
        prev_order_idx: &mut u32,
        price: &mut u64,
        quantity: &mut u64,
        owner: &mut Pubkey,
    ) -> Result<()> {
        // 使用非递归方法遍历价格树
        let mut stack = Vec::new();
        let mut current = root_idx;

        while !*found && (current != u32::MAX || !stack.is_empty()) {
            while current != u32::MAX {
                stack.push(current);
                current = self.price_nodes[current as usize].left;
            }

            if !stack.is_empty() {
                current = stack.pop().unwrap();

                // 检查当前价格节点下的所有订单
                let price_node = &self.price_nodes[current as usize];
                let mut curr_order_idx = price_node.first_order;
                let mut prev_idx = u32::MAX;

                while curr_order_idx != u32::MAX {
                    let order_node = &self.order_nodes[curr_order_idx as usize];

                    if order_node.order_id == order_id {
                        // 找到订单
                        *found = true;
                        *price_idx = current;
                        *order_idx = curr_order_idx;
                        *prev_order_idx = prev_idx;
                        *price = price_node.price;
                        *quantity = order_node.quantity;
                        *owner = order_node.owner;
                        return Ok(());
                    }

                    // 移动到下一个订单
                    prev_idx = curr_order_idx;
                    curr_order_idx = order_node.next;
                }

                // 继续遍历右子树
                current = price_node.right;
            }
        }

        Ok(())
    }

    // 查找或创建价格节点
    fn find_or_create_price_node(&mut self, price: u64, side: Side) -> Result<u32> {
        // 选择合适的价格树根节点
        let root_ptr = match side {
            Side::Bid => &mut self.bid_price_tree_root,
            Side::Ask => &mut self.ask_price_tree_root,
        };

        // 如果树为空，创建新的根节点
        if *root_ptr == u32::MAX {
            let new_idx = self.allocate_price_node()?;
            let node = &mut self.price_nodes[new_idx as usize];
            node.price = price;
            node.quantity = 0;
            node.orders_count = 0;
            node.first_order = u32::MAX;
            node.parent = u32::MAX;
            node.left = u32::MAX;
            node.right = u32::MAX;
            node.next_price = u32::MAX;
            node.prev_price = u32::MAX;

            *root_ptr = new_idx;
            return Ok(new_idx);
        }

        // 查找合适的位置
        let mut current = *root_ptr;
        let mut parent = u32::MAX;
        let mut is_left = false;

        while current != u32::MAX {
            let node = &self.price_nodes[current as usize];

            if price == node.price {
                // 找到匹配的价格节点
                return Ok(current);
            }

            parent = current;

            if (side == Side::Bid && price > node.price)
                || (side == Side::Ask && price < node.price)
            {
                // 向左走
                current = node.left;
                is_left = true;
            } else {
                // 向右走
                current = node.right;
                is_left = false;
            }
        }

        // 创建新节点
        let new_idx = self.allocate_price_node()?;
        let new_node = &mut self.price_nodes[new_idx as usize];
        new_node.price = price;
        new_node.quantity = 0;
        new_node.orders_count = 0;
        new_node.first_order = u32::MAX;
        new_node.parent = parent;
        new_node.left = u32::MAX;
        new_node.right = u32::MAX;
        new_node.next_price = u32::MAX;
        new_node.prev_price = u32::MAX;

        // 连接到父节点
        if parent != u32::MAX {
            if is_left {
                self.price_nodes[parent as usize].left = new_idx;
            } else {
                self.price_nodes[parent as usize].right = new_idx;
            }
        }

        // 更新价格链表
        self.update_price_list(new_idx, side)?;

        Ok(new_idx)
    }

    // 更新价格链表
    fn update_price_list(&mut self, new_idx: u32, side: Side) -> Result<()> {
        let new_price = self.price_nodes[new_idx as usize].price;

        match side {
            Side::Bid => {
                // 买单价格从高到低排序
                // 查找第一个价格低于新价格的节点
                let mut current = self.bid_price_tree_root;
                let mut prev = u32::MAX;
                let mut found = false;

                while current != u32::MAX && !found {
                    let node = &self.price_nodes[current as usize];

                    if node.price < new_price {
                        found = true;
                    } else {
                        prev = current;
                        current = node.next_price;
                    }
                }

                // 更新链表
                if prev == u32::MAX {
                    // 新节点是链表的头部
                    self.price_nodes[new_idx as usize].next_price = self.bid_price_tree_root;
                    if self.bid_price_tree_root != u32::MAX {
                        self.price_nodes[self.bid_price_tree_root as usize].prev_price = new_idx;
                    }
                    // 不需要更新根节点，因为根节点是通过平衡树确定的
                } else {
                    // 新节点在链表中间
                    self.price_nodes[new_idx as usize].next_price =
                        self.price_nodes[prev as usize].next_price;
                    self.price_nodes[new_idx as usize].prev_price = prev;
                    if self.price_nodes[prev as usize].next_price != u32::MAX {
                        self.price_nodes[self.price_nodes[prev as usize].next_price as usize]
                            .prev_price = new_idx;
                    }
                    self.price_nodes[prev as usize].next_price = new_idx;
                }
            }
            Side::Ask => {
                // 卖单价格从低到高排序
                // 查找第一个价格高于新价格的节点
                let mut current = self.ask_price_tree_root;
                let mut prev = u32::MAX;
                let mut found = false;

                while current != u32::MAX && !found {
                    let node = &self.price_nodes[current as usize];

                    if node.price > new_price {
                        found = true;
                    } else {
                        prev = current;
                        current = node.next_price;
                    }
                }

                // 更新链表
                if prev == u32::MAX {
                    // 新节点是链表的头部
                    self.price_nodes[new_idx as usize].next_price = self.ask_price_tree_root;
                    if self.ask_price_tree_root != u32::MAX {
                        self.price_nodes[self.ask_price_tree_root as usize].prev_price = new_idx;
                    }
                    // 不需要更新根节点，因为根节点是通过平衡树确定的
                } else {
                    // 新节点在链表中间
                    self.price_nodes[new_idx as usize].next_price =
                        self.price_nodes[prev as usize].next_price;
                    self.price_nodes[new_idx as usize].prev_price = prev;
                    if self.price_nodes[prev as usize].next_price != u32::MAX {
                        self.price_nodes[self.price_nodes[prev as usize].next_price as usize]
                            .prev_price = new_idx;
                    }
                    self.price_nodes[prev as usize].next_price = new_idx;
                }
            }
        }

        Ok(())
    }

    // 分配价格节点
    fn allocate_price_node(&mut self) -> Result<u32> {
        if self.free_price_nodes_count == 0 {
            return Err(ErrorCode::OrderBookFull.into());
        }

        let node_idx = self.free_price_nodes[(self.free_price_nodes_count - 1) as usize];
        self.free_price_nodes_count -= 1;

        Ok(node_idx)
    }

    // 分配订单节点
    fn allocate_order_node(&mut self) -> Result<u32> {
        if self.free_order_nodes_count == 0 {
            return Err(ErrorCode::OrderBookFull.into());
        }

        let node_idx = self.free_order_nodes[(self.free_order_nodes_count - 1) as usize];
        self.free_order_nodes_count -= 1;

        Ok(node_idx)
    }

    // 移除订单节点
    fn remove_order_node(
        &mut self,
        price_idx: u32,
        order_idx: u32,
        prev_order_idx: u32,
    ) -> Result<()> {
        // 获取订单节点和价格节点
        let order_node = &self.order_nodes[order_idx as usize];
        let next_order_idx = order_node.next;
        let order_quantity = order_node.quantity;

        // 更新链表
        if prev_order_idx == u32::MAX {
            // 是价格节点的第一个订单
            self.price_nodes[price_idx as usize].first_order = next_order_idx;
            if next_order_idx != u32::MAX {
                self.order_nodes[next_order_idx as usize].prev = u32::MAX;
            }
        } else {
            // 不是第一个订单
            self.order_nodes[prev_order_idx as usize].next = next_order_idx;
            if next_order_idx != u32::MAX {
                self.order_nodes[next_order_idx as usize].prev = prev_order_idx;
            }
        }

        // 更新价格节点统计
        self.price_nodes[price_idx as usize].quantity -= order_quantity;
        self.price_nodes[price_idx as usize].orders_count -= 1;

        // 更新全局统计
        let side = if self.bid_price_tree_root == price_idx
            || self.is_in_tree(self.bid_price_tree_root, price_idx)
        {
            Side::Bid
        } else {
            Side::Ask
        };

        match side {
            Side::Bid => {
                self.bid_orders_count -= 1;
                self.bid_volume -= order_quantity;
            }
            Side::Ask => {
                self.ask_orders_count -= 1;
                self.ask_volume -= order_quantity;
            }
        }

        // 回收订单节点
        self.free_order_nodes[self.free_order_nodes_count as usize] = order_idx;
        self.free_order_nodes_count += 1;

        Ok(())
    }

    // 检查节点是否在树中
    fn is_in_tree(&self, root: u32, node_idx: u32) -> bool {
        if root == u32::MAX {
            return false;
        }

        if root == node_idx {
            return true;
        }

        self.is_in_tree(self.price_nodes[root as usize].left, node_idx)
            || self.is_in_tree(self.price_nodes[root as usize].right, node_idx)
    }

    // 移除价格节点
    fn remove_price_node(&mut self, node_idx: u32, side: Side) -> Result<()> {
        // 选择合适的价格树根节点
        let root_ptr = match side {
            Side::Bid => &mut self.bid_price_tree_root,
            Side::Ask => &mut self.ask_price_tree_root,
        };

        if *root_ptr == u32::MAX {
            return Ok(());
        }

        // 更新价格链表
        let node = &self.price_nodes[node_idx as usize];
        let prev_price_idx = node.prev_price;
        let next_price_idx = node.next_price;

        if prev_price_idx != u32::MAX {
            self.price_nodes[prev_price_idx as usize].next_price = next_price_idx;
        }

        if next_price_idx != u32::MAX {
            self.price_nodes[next_price_idx as usize].prev_price = prev_price_idx;
        }

        // 删除节点并重新平衡树
        *root_ptr = self.delete_node_from_tree(*root_ptr, node_idx)?;

        // 回收价格节点
        self.free_price_nodes[self.free_price_nodes_count as usize] = node_idx;
        self.free_price_nodes_count += 1;

        Ok(())
    }

    // 从树中删除节点
    fn delete_node_from_tree(&mut self, root: u32, node_idx: u32) -> Result<u32> {
        if root == u32::MAX {
            return Ok(u32::MAX);
        }

        if node_idx < root {
            self.price_nodes[root as usize].left =
                self.delete_node_from_tree(self.price_nodes[root as usize].left, node_idx)?;
        } else if node_idx > root {
            self.price_nodes[root as usize].right =
                self.delete_node_from_tree(self.price_nodes[root as usize].right, node_idx)?;
        } else {
            // 找到要删除的节点

            // 没有子节点或只有一个子节点
            if self.price_nodes[root as usize].left == u32::MAX {
                return Ok(self.price_nodes[root as usize].right);
            } else if self.price_nodes[root as usize].right == u32::MAX {
                return Ok(self.price_nodes[root as usize].left);
            }

            // 有两个子节点
            // 找到右子树中的最小节点作为替代
            let mut successor = self.price_nodes[root as usize].right;
            while self.price_nodes[successor as usize].left != u32::MAX {
                successor = self.price_nodes[successor as usize].left;
            }

            // 复制继任者的数据到当前节点
            self.price_nodes[root as usize].price = self.price_nodes[successor as usize].price;
            self.price_nodes[root as usize].quantity =
                self.price_nodes[successor as usize].quantity;
            self.price_nodes[root as usize].orders_count =
                self.price_nodes[successor as usize].orders_count;
            self.price_nodes[root as usize].first_order =
                self.price_nodes[successor as usize].first_order;

            // 从右子树中删除继任者
            self.price_nodes[root as usize].right =
                self.delete_node_from_tree(self.price_nodes[root as usize].right, successor)?;
        }

        Ok(root)
    }

    // 查找最小价格节点
    fn find_min_price_node(&self, root: u32) -> Result<u32> {
        if root == u32::MAX {
            return Err(ErrorCode::OrderNotFound.into());
        }

        let mut current = root;
        while self.price_nodes[current as usize].left != u32::MAX {
            current = self.price_nodes[current as usize].left;
        }

        Ok(current)
    }

    // 查找最大价格节点
    fn find_max_price_node(&self, root: u32) -> Result<u32> {
        if root == u32::MAX {
            return Err(ErrorCode::OrderNotFound.into());
        }

        let mut current = root;
        while self.price_nodes[current as usize].right != u32::MAX {
            current = self.price_nodes[current as usize].right;
        }

        Ok(current)
    }

    // 更新订单数量
    fn update_order_quantity(&mut self, order_idx: u32, new_quantity: u64) -> Result<()> {
        let order_node = &mut self.order_nodes[order_idx as usize];
        let price_idx = order_node.price_index;
        let old_quantity = order_node.quantity;

        // 更新订单数量
        order_node.quantity = new_quantity;

        // 更新价格节点统计
        self.price_nodes[price_idx as usize].quantity =
            self.price_nodes[price_idx as usize].quantity - old_quantity + new_quantity;

        // 更新全局统计
        let side = if self.bid_price_tree_root == price_idx
            || self.is_in_tree(self.bid_price_tree_root, price_idx)
        {
            Side::Bid
        } else {
            Side::Ask
        };

        match side {
            Side::Bid => {
                self.bid_volume = self.bid_volume - old_quantity + new_quantity;
            }
            Side::Ask => {
                self.ask_volume = self.ask_volume - old_quantity + new_quantity;
            }
        }

        Ok(())
    }

    // 清理过期订单
    fn purge_expired_orders(&mut self) -> Result<()> {
        self.last_purge_slot = Clock::get()?.slot;
        let current_ts = Clock::get()?.unix_timestamp;

        // 清理买单
        if self.bid_price_tree_root != u32::MAX {
            self.purge_expired_orders_for_side(self.bid_price_tree_root, current_ts, Side::Bid)?;
        }

        // 清理卖单
        if self.ask_price_tree_root != u32::MAX {
            self.purge_expired_orders_for_side(self.ask_price_tree_root, current_ts, Side::Ask)?;
        }

        Ok(())
    }

    // 清理指定方向的过期订单
    fn purge_expired_orders_for_side(
        &mut self,
        root: u32,
        current_ts: i64,
        side: Side,
    ) -> Result<()> {
        if root == u32::MAX {
            return Ok(());
        }

        // 非递归实现，使用栈
        let mut stack = Vec::new();
        let mut current = root;

        while current != u32::MAX || !stack.is_empty() {
            while current != u32::MAX {
                stack.push(current);
                current = self.price_nodes[current as usize].left;
            }

            if !stack.is_empty() {
                current = stack.pop().unwrap();

                // 处理当前价格节点下的所有订单
                let price_idx = current;
                let mut order_idx = self.price_nodes[price_idx as usize].first_order;
                let mut prev_order_idx = u32::MAX;

                while order_idx != u32::MAX {
                    let order_node = &self.order_nodes[order_idx as usize];
                    let next_order_idx = order_node.next;

                    // 检查订单是否过期
                    if order_node.max_ts_valid > 0 && current_ts > order_node.max_ts_valid {
                        // 移除过期订单
                        self.remove_order_node(price_idx, order_idx, prev_order_idx)?;
                        order_idx = next_order_idx;
                    } else {
                        // 移动到下一个订单
                        prev_order_idx = order_idx;
                        order_idx = next_order_idx;
                    }
                }

                // 检查价格节点是否为空
                if self.price_nodes[price_idx as usize].orders_count == 0 {
                    // 保存右子树，因为在删除节点后，树结构可能改变
                    let right = self.price_nodes[price_idx as usize].right;

                    // 从树中移除空价格节点
                    self.remove_price_node(price_idx, side)?;

                    // 继续处理右子树
                    current = right;
                } else {
                    // 继续处理右子树
                    current = self.price_nodes[price_idx as usize].right;
                }
            }
        }

        Ok(())
    }

    // 获取市场深度
    pub fn get_market_depth(&self, side: Side, limit: u8) -> Result<Vec<(u64, u64)>> {
        let mut result = Vec::new();
        let mut count = 0;

        match side {
            Side::Bid => {
                if self.bid_price_tree_root == u32::MAX {
                    return Ok(result);
                }

                // 从最高价格买单开始
                let mut price_idx = match self.find_max_price_node(self.bid_price_tree_root) {
                    Ok(idx) => idx,
                    Err(_) => return Ok(result),
                };

                // 遍历价格链表
                while price_idx != u32::MAX && count < limit {
                    let price_node = &self.price_nodes[price_idx as usize];
                    result.push((price_node.price, price_node.quantity));
                    count += 1;
                    price_idx = price_node.next_price;
                }
            }
            Side::Ask => {
                if self.ask_price_tree_root == u32::MAX {
                    return Ok(result);
                }

                // 从最低价格卖单开始
                let mut price_idx = match self.find_min_price_node(self.ask_price_tree_root) {
                    Ok(idx) => idx,
                    Err(_) => return Ok(result),
                };

                // 遍历价格链表
                while price_idx != u32::MAX && count < limit {
                    let price_node = &self.price_nodes[price_idx as usize];
                    result.push((price_node.price, price_node.quantity));
                    count += 1;
                    price_idx = price_node.next_price;
                }
            }
        }

        Ok(result)
    }

    // 获取最优价格
    pub fn get_best_price(&self, side: Side) -> Option<u64> {
        match side {
            Side::Bid => {
                if self.bid_price_tree_root == u32::MAX {
                    return None;
                }

                match self.find_max_price_node(self.bid_price_tree_root) {
                    Ok(idx) => Some(self.price_nodes[idx as usize].price),
                    Err(_) => None,
                }
            }
            Side::Ask => {
                if self.ask_price_tree_root == u32::MAX {
                    return None;
                }

                match self.find_min_price_node(self.ask_price_tree_root) {
                    Ok(idx) => Some(self.price_nodes[idx as usize].price),
                    Err(_) => None,
                }
            }
        }
    }

    // 获取当前价差
    pub fn get_spread(&self) -> Option<u64> {
        let best_bid = self.get_best_price(Side::Bid);
        let best_ask = self.get_best_price(Side::Ask);

        match (best_bid, best_ask) {
            (Some(bid), Some(ask)) => {
                if ask > bid {
                    Some(ask - bid)
                } else {
                    Some(0)
                }
            }
            _ => None,
        }
    }
}
