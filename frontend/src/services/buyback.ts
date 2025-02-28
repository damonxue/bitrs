import api from './api';

export interface BuybackStats {
  total_burned: number;
  last_buyback_timestamp: number;
  accumulated_fees: number;
  next_buyback_estimate: number;
  current_market_price: number;
}

export const buybackApi = {
  getStats: async (): Promise<BuybackStats> => {
    const response = await api.get('/stats/buyback');
    return response.data;
  },
};