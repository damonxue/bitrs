import api from './api';

export interface SystemMetrics {
  timestamp: string;
  total_volume_24h: number;
  total_fees_collected: number;
  total_rewards_distributed: number;
  total_tokens_burned: number;
  active_lp_count: number;
  avg_apr: number;
}

export interface PoolMetrics {
  pool_id: string;
  volume_24h: number;
  tvl: number;
  apr: number;
  lp_count: number;
  reward_rate: number;
}

export interface AnalyticsData {
  timestamp: string;
  metrics: SystemMetrics;
  pools: PoolMetrics[];
}

export const analyticsApi = {
  getAnalytics: async (): Promise<AnalyticsData> => {
    const response = await api.get('/api/v1/analytics');
    return response.data;
  },

  getPoolAnalytics: async (poolId: string): Promise<PoolMetrics> => {
    const response = await api.get(`/api/v1/analytics/pools/${poolId}`);
    return response.data;
  },
};