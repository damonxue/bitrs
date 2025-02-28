// 流动性池定义
export interface Pool {
  pool_id: string;
  token_a: string;
  token_b: string;
  reserve_a: number;
  reserve_b: number;
  fee_rate: number;
  lp_token_supply: number;
}

// 交换请求
export interface SwapQuoteRequest {
  pool_id: string;
  token_in: string;
  amount_in: number;
}

// 交换报价响应
export interface SwapQuoteResponse {
  token_out: string;
  amount_out: number;
  price_impact: number;
  fee: number;
}