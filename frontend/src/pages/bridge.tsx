import React, { useState, useEffect } from 'react';
import { bridgeApi, assetApi } from '@/services/api';
import { useWallet } from '@/context/WalletContext';
import type { Token } from '@/types/asset';

const Bridge = () => {
  const { wallet } = useWallet();
  const [tokens, setTokens] = useState<Token[]>([]);
  const [selectedToken, setSelectedToken] = useState('');
  const [amount, setAmount] = useState('');
  const [fromChain, setFromChain] = useState('solana');
  const [toChain, setToChain] = useState('ethereum');
  const [loading, setLoading] = useState(false);
  const [bridgeStatus, setBridgeStatus] = useState('');

  const chains = ['solana', 'ethereum', 'bsc'];

  useEffect(() => {
    loadTokens();
    checkBridgeStatus();
  }, []);

  const loadTokens = async () => {
    try {
      const tokenList = await assetApi.getTokens();
      setTokens(tokenList);
    } catch (error) {
      console.error('Failed to load tokens:', error);
    }
  };

  const checkBridgeStatus = async () => {
    try {
      const { status } = await bridgeApi.getBridgeStatus();
      setBridgeStatus(status);
    } catch (error) {
      console.error('Failed to check bridge status:', error);
    }
  };

  const handleTransfer = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!wallet || !amount || !selectedToken) return;

    setLoading(true);
    try {
      const result = await bridgeApi.bridgeTransfer({
        fromChain,
        toChain,
        token: selectedToken,
        amount,
        recipient: wallet.publicKey.toString(),
      });
      console.log('Bridge transfer initiated:', result.tx_hash);
      // Show success message or transaction hash
    } catch (error) {
      console.error('Bridge transfer failed:', error);
      // Show error message
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Cross-Chain Bridge</h1>
      
      <div className="bg-blue-100 p-4 rounded mb-4">
        Bridge Status: {bridgeStatus || 'Checking...'}
      </div>

      <form onSubmit={handleTransfer} className="max-w-md mx-auto">
        <div className="mb-4">
          <label className="block mb-2">From Chain</label>
          <select 
            value={fromChain}
            onChange={(e) => setFromChain(e.target.value)}
            className="w-full p-2 border rounded"
          >
            {chains.map(chain => (
              <option key={chain} value={chain}>{chain}</option>
            ))}
          </select>
        </div>

        <div className="mb-4">
          <label className="block mb-2">To Chain</label>
          <select 
            value={toChain}
            onChange={(e) => setToChain(e.target.value)}
            className="w-full p-2 border rounded"
          >
            {chains.map(chain => (
              <option key={chain} value={chain}>{chain}</option>
            ))}
          </select>
        </div>

        <div className="mb-4">
          <label className="block mb-2">Token</label>
          <select 
            value={selectedToken}
            onChange={(e) => setSelectedToken(e.target.value)}
            className="w-full p-2 border rounded"
          >
            <option value="">Select Token</option>
            {tokens.map(token => (
              <option key={token.address} value={token.address}>
                {token.symbol}
              </option>
            ))}
          </select>
        </div>

        <div className="mb-4">
          <label className="block mb-2">Amount</label>
          <input
            type="text"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="w-full p-2 border rounded"
            placeholder="Enter amount"
          />
        </div>

        <button
          type="submit"
          disabled={loading || !wallet}
          className="w-full bg-blue-500 text-white p-2 rounded disabled:bg-gray-300"
        >
          {loading ? 'Processing...' : 'Transfer'}
        </button>
      </form>
    </div>
  );
};

export default Bridge;