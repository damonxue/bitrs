import React, { useEffect, useState } from 'react';
import { ammApi } from '@/services/api';
import { rewardsApi } from '@/services/rewards';
import { Pool } from '@/types/amm';
import { RewardInfo, PoolRewards } from '@/services/rewards';
import { useWallet } from '@/context/WalletContext';

const Pools = () => {
  const [pools, setPools] = useState<Pool[]>([]);
  const [poolRewards, setPoolRewards] = useState<PoolRewards[]>([]);
  const [userRewards, setUserRewards] = useState<RewardInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const { wallet } = useWallet();

  useEffect(() => {
    loadPoolsAndRewards();
  }, []);

  useEffect(() => {
    if (wallet?.publicKey) {
      loadUserRewards();
    }
  }, [wallet?.publicKey]);

  const loadPoolsAndRewards = async () => {
    try {
      const [poolData, rewardsData] = await Promise.all([
        ammApi.getPools(),
        rewardsApi.getPoolRewards(),
      ]);
      setPools(poolData);
      setPoolRewards(rewardsData);
    } catch (error) {
      console.error('Failed to load pools:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadUserRewards = async () => {
    if (!wallet?.publicKey) return;
    try {
      const rewards = await rewardsApi.getUserRewards(wallet.publicKey.toString());
      setUserRewards(rewards);
    } catch (error) {
      console.error('Failed to load user rewards:', error);
    }
  };

  const handleClaimRewards = async () => {
    if (!wallet?.publicKey) return;
    try {
      await rewardsApi.claimRewards(wallet.publicKey.toString());
      await loadUserRewards();
    } catch (error) {
      console.error('Failed to claim rewards:', error);
    }
  };

  if (loading) {
    return <div>Loading pools...</div>;
  }

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Liquidity Pools</h1>
      
      {userRewards && (
        <div className="mb-6 bg-blue-50 p-4 rounded-lg">
          <h2 className="text-lg font-semibold mb-2">Your Rewards</h2>
          <div className="flex justify-between items-center">
            <div>
              <p>Pending Rewards: {userRewards.pending_rewards}</p>
              <p>Claimed Rewards: {userRewards.claimed_rewards}</p>
              <p>Next Distribution: {new Date(userRewards.next_distribution * 1000).toLocaleString()}</p>
            </div>
            <button
              onClick={handleClaimRewards}
              disabled={userRewards.pending_rewards <= 0}
              className="bg-green-500 text-white px-4 py-2 rounded disabled:bg-gray-300"
            >
              Claim Rewards
            </button>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {pools.map((pool) => {
          const poolReward = poolRewards.find(r => r.pool_id === pool.id);
          return (
            <div key={pool.id} className="border rounded-lg p-4 shadow">
              <h2 className="text-xl font-semibold">{pool.name}</h2>
              <div className="mt-2">
                <p>Total Value Locked: ${pool.tvl}</p>
                <p>APR: {poolReward?.apr || 0}%</p>
                <p>Daily Rewards: {poolReward?.daily_rewards || 0}</p>
                <p>Your Position: {pool.userPosition || '0'}</p>
              </div>
              <div className="mt-4 flex gap-2">
                <button 
                  className="bg-blue-500 text-white px-4 py-2 rounded"
                  onClick={() => {/* Open add liquidity modal */}}
                >
                  Add Liquidity
                </button>
                <button 
                  className="bg-red-500 text-white px-4 py-2 rounded"
                  onClick={() => {/* Open remove liquidity modal */}}
                >
                  Remove Liquidity
                </button>
              </div>
              <div className="mt-4 text-sm text-gray-600">
                <p>Total Staked: {poolReward?.total_staked || 0}</p>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default Pools;