// 代币定义
export interface Token {
  mint: string;
  symbol: string;
  name: string;
  decimals: number;
  logo_uri?: string;
}

// 资产余额
export interface Balance {
  token: string;
  amount: number;
}