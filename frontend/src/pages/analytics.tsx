import React, { useEffect, useState } from 'react';
import { analyticsApi, AnalyticsData } from '@/services/analytics';
import SystemMetrics from '@/components/analytics/SystemMetrics';
import dynamic from 'next/dynamic';

// Dynamically import chart components to avoid SSR issues
const Chart = dynamic(() => import('react-chartjs-2').then(mod => mod.Line), { ssr: false });

const Analytics = () => {
  const [data, setData] = useState<AnalyticsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedPool, setSelectedPool] = useState<string | null>(null);

  useEffect(() => {
    loadAnalytics();
    const interval = setInterval(loadAnalytics, 60000); // Refresh every minute
    return () => clearInterval(interval);
  }, []);

  const loadAnalytics = async () => {
    try {
      const analyticsData = await analyticsApi.getAnalytics();
      setData(analyticsData);
    } catch (error) {
      console.error('Failed to load analytics:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className="flex justify-center items-center min-h-screen">Loading analytics...</div>;
  }

  if (!data) {
    return <div className="text-center text-red-500">Failed to load analytics data</div>;
  }

  return (
    <div className="container mx-auto p-4 space-y-8">
      <h1 className="text-3xl font-bold mb-6">Analytics Dashboard</h1>

      {/* System Metrics */}
      <div className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">System Metrics</h2>
        <SystemMetrics metrics={data.metrics} />
      </div>

      {/* Pool Metrics */}
      <div className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">Pool Performance</h2>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {data.pools.map((pool) => (
            <div key={pool.pool_id} className="bg-white rounded-lg shadow p-4">
              <h3 className="text-lg font-semibold mb-2">{pool.pool_id}</h3>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-sm text-gray-600">24h Volume</p>
                  <p className="text-lg font-medium">${new Intl.NumberFormat().format(pool.volume_24h)}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">TVL</p>
                  <p className="text-lg font-medium">${new Intl.NumberFormat().format(pool.tvl)}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">APR</p>
                  <p className="text-lg font-medium">{pool.apr.toFixed(2)}%</p>
                </div>
                <div>
                  <p className="text-sm text-gray-600">Active LPs</p>
                  <p className="text-lg font-medium">{pool.lp_count}</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Historical Data */}
      <div className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">Historical Performance</h2>
        <div className="bg-white rounded-lg shadow p-4">
          {/* Add historical data visualization here */}
        </div>
      </div>
    </div>
  );
};

export default Analytics;