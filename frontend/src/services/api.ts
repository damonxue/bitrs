import axios from 'axios';
import { Market, OrderBook, Trade, OrderRequest } from '@/types/market';
import { Pool, SwapQuoteRequest, SwapQuoteResponse } from '@/types/amm';
import { Token, Balance } from '@/types/asset';

const API_URL = process.env.API_URL || 'http://localhost:8080/api/v1';

const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 订单簿相关API
export const marketApi = {
  // 获取所有市场
  getMarkets: async (): Promise<Market[]> => {
    const response = await api.get('/orderbook/markets');
    return response.data;
  },

  // 获取特定市场的订单簿
  getOrderBook: async (marketId: string): Promise<OrderBook> => {
    const response = await api.get(`/orderbook/${marketId}`);
    return response.data;
  },

  // 提交订单
  placeOrder: async (order: OrderRequest): Promise<{ tx_hash: string }> => {
    const response = await api.post('/orderbook/order', order);
    return response.data;
  },

  // 取消订单
  cancelOrder: async (orderId: string): Promise<{ success: boolean }> => {
    const response = await api.delete(`/orderbook/order/${orderId}`);
    return response.data;
  },

  // 获取最近交易
  getRecentTrades: async (marketId: string): Promise<Trade[]> => {
    const response = await api.get(`/trades/recent/${marketId}`);
    return response.data;
  },
};

// AMM相关API
export const ammApi = {
  // 获取所有流动性池
  getPools: async (): Promise<Pool[]> => {
    const response = await api.get('/amm/pools');
    return response.data;
  },

  // 获取特定流动性池信息
  getPool: async (poolId: string): Promise<Pool> => {
    const response = await api.get(`/amm/pool/${poolId}`);
    return response.data;
  },

  // 获取交易报价
  getSwapQuote: async (request: SwapQuoteRequest): Promise<SwapQuoteResponse> => {
    const response = await api.post('/amm/swap/quote', request);
    return response.data;
  },
};

// 资产相关API
export const assetApi = {
  // 获取支持的代币列表
  getTokens: async (): Promise<Token[]> => {
    const response = await api.get('/assets/tokens');
    return response.data;
  },

  // 获取钱包余额
  getBalance: async (wallet: string): Promise<Balance[]> => {
    const response = await api.get(`/assets/balance/${wallet}`);
    return response.data;
  },

  // 获取用户交易历史
  getUserTrades: async (wallet: string): Promise<Trade[]> => {
    const response = await api.get(`/trades/history/${wallet}`);
    return response.data;
  },
};

// Bridge相关API
export const bridgeApi = {
  // 获取跨链桥状态
  getBridgeStatus: async (): Promise<{ status: string }> => {
    const response = await api.get('/bridge/status');
    return response.data;
  },

  // 发起跨链转账
  bridgeTransfer: async (params: {
    fromChain: string;
    toChain: string;
    token: string;
    amount: string;
    recipient: string;
  }): Promise<{ tx_hash: string }> => {
    const response = await api.post('/bridge/transfer', params);
    return response.data;
  },

  // 获取跨链交易历史
  getBridgeHistory: async (wallet: string): Promise<any[]> => {
    const response = await api.get(`/bridge/history/${wallet}`);
    return response.data;
  },
};

export default api;