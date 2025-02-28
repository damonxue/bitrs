import { useQuery } from 'react-query';
import { marketApi } from '@/services/api';
import { Market, OrderBook, Trade } from '@/types/market';

// 获取所有市场列表
export const useMarketData = () => {
  return useQuery<Market[], Error>('markets', () => marketApi.getMarkets(), {
    staleTime: 60 * 1000, // 1分钟内不重新获取
    refetchOnWindowFocus: false,
  });
};

// 获取特定市场的订单簿
export const useOrderBook = (marketId: string | undefined) => {
  return useQuery<OrderBook, Error>(
    ['orderbook', marketId],
    () => marketId ? marketApi.getOrderBook(marketId) : Promise.reject('无效的市场ID'),
    {
      enabled: !!marketId, // 仅在marketId存在时获取数据
      refetchInterval: 5000, // 每5秒刷新一次
      staleTime: 2000, // 2秒内不重新获取
    }
  );
};

// 获取特定市场的最近交易
export const useRecentTrades = (marketId: string | undefined) => {
  return useQuery<Trade[], Error>(
    ['trades', marketId],
    () => marketId ? marketApi.getRecentTrades(marketId) : Promise.reject('无效的市场ID'),
    {
      enabled: !!marketId,
      refetchInterval: 5000, // 每5秒刷新一次
      staleTime: 2000, // 2秒内不重新获取
    }
  );
};