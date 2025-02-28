import React, { useEffect, useState } from 'react';
import { buybackApi, BuybackStats as BuybackStatsType } from '@/services/buyback';

const BuybackStats: React.FC = () => {
  const [stats, setStats] = useState<BuybackStatsType | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadStats();
    const interval = setInterval(loadStats, 60000); // Refresh every minute
    return () => clearInterval(interval);
  }, []);

  const loadStats = async () => {
    try {
      const data = await buybackApi.getStats();
      setStats(data);
    } catch (error) {
      console.error('Failed to load buyback stats:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) return <div>Loading buyback stats...</div>;
  if (!stats) return null;

  const formatNumber = (num: number) => {
    return new Intl.NumberFormat().format(num);
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  return (
    <div className="bg-white rounded-lg shadow p-4">
      <h2 className="text-lg font-semibold mb-4">Token Buyback Statistics</h2>
      <div className="grid grid-cols-2 gap-4">
        <div>
          <p className="text-sm text-gray-600">Total Tokens Burned</p>
          <p className="font-medium">{formatNumber(stats.total_burned)}</p>
        </div>
        <div>
          <p className="text-sm text-gray-600">Current Market Price</p>
          <p className="font-medium">${stats.current_market_price.toFixed(4)}</p>
        </div>
        <div>
          <p className="text-sm text-gray-600">Last Buyback</p>
          <p className="font-medium">{formatTimestamp(stats.last_buyback_timestamp)}</p>
        </div>
        <div>
          <p className="text-sm text-gray-600">Accumulated Fees</p>
          <p className="font-medium">{formatNumber(stats.accumulated_fees)}</p>
        </div>
        <div className="col-span-2">
          <p className="text-sm text-gray-600">Next Estimated Buyback</p>
          <p className="font-medium">{formatTimestamp(stats.next_buyback_estimate)}</p>
        </div>
      </div>
    </div>
  );
};

export default BuybackStats;