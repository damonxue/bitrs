// 市场/交易对定义
export interface Market {
  market_id: string;
  base_token: string;
  quote_token: string;
  base_decimals: number;
  quote_decimals: number;
}

// 订单方向
export enum OrderSide {
  Buy = 'buy',
  Sell = 'sell'
}

// 订单类型
export interface Order {
  order_id: string;
  price: number;
  size: number;
  side: OrderSide;
  user: string;
  timestamp: number;
}

// 订单簿
export interface OrderBook {
  market_id: string;
  bids: Order[];  // 买单
  asks: Order[];  // 卖单
}

// 交易
export interface Trade {
  market_id: string;
  price: number;
  size: number;
  side: OrderSide;
  timestamp: number;
  bid_order_id: string;
  ask_order_id: string;
  bid_user?: string;
  ask_user?: string;
}

// 下单请求
export interface OrderRequest {
  market_id: string;
  price: number;
  size: number;
  side: OrderSide;
  user: string;
  signature?: string;
}