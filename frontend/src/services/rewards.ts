import api from './api';

export interface RewardInfo {
  pending_rewards: number;
  claimed_rewards: number;
  apr: number;
  next_distribution: number;
}

export interface PoolRewards {
  pool_id: string;
  daily_rewards: number;
  apr: number;
  total_staked: number;
}

export const rewardsApi = {
  getUserRewards: async (wallet: string): Promise<RewardInfo> => {
    const response = await api.get(`/rewards/${wallet}`);
    return response.data;
  },

  getPoolRewards: async (): Promise<PoolRewards[]> => {
    const response = await api.get('/pools/rewards');
    return response.data;
  },

  claimRewards: async (wallet: string): Promise<{ tx_hash: string }> => {
    const response = await api.post(`/rewards/${wallet}/claim`);
    return response.data;
  },
};

export default rewardsApi;