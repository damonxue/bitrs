import React from 'react';
import { SystemMetrics as SystemMetricsType } from '@/services/analytics';

interface Props {
  metrics: SystemMetricsType;
}

const SystemMetrics: React.FC<Props> = ({ metrics }) => {
  const formatNumber = (num: number) => {
    return new Intl.NumberFormat().format(num);
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <div className="bg-white rounded-lg shadow p-4">
        <h3 className="text-lg font-semibold text-gray-700">24h Volume</h3>
        <p className="text-2xl font-bold">${formatNumber(metrics.total_volume_24h)}</p>
      </div>

      <div className="bg-white rounded-lg shadow p-4">
        <h3 className="text-lg font-semibold text-gray-700">Total Fees</h3>
        <p className="text-2xl font-bold">${formatNumber(metrics.total_fees_collected)}</p>
      </div>

      <div className="bg-white rounded-lg shadow p-4">
        <h3 className="text-lg font-semibold text-gray-700">Rewards Distributed</h3>
        <p className="text-2xl font-bold">${formatNumber(metrics.total_rewards_distributed)}</p>
      </div>

      <div className="bg-white rounded-lg shadow p-4">
        <h3 className="text-lg font-semibold text-gray-700">Tokens Burned</h3>
        <p className="text-2xl font-bold">{formatNumber(metrics.total_tokens_burned)}</p>
      </div>

      <div className="bg-white rounded-lg shadow p-4">
        <h3 className="text-lg font-semibold text-gray-700">Active LPs</h3>
        <p className="text-2xl font-bold">{formatNumber(metrics.active_lp_count)}</p>
      </div>

      <div className="bg-white rounded-lg shadow p-4">
        <h3 className="text-lg font-semibold text-gray-700">Average APR</h3>
        <p className="text-2xl font-bold">{metrics.avg_apr.toFixed(2)}%</p>
      </div>
    </div>
  );
};

export default SystemMetrics;