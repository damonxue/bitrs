import React, { useState } from 'react';
import { useWallet } from '@/context/WalletContext';
import TransactionPreview from '@/components/common/TransactionPreview';
import { Transaction } from '@solana/web3.js';

const TradeForm = () => {
  const { wallet } = useWallet();
  const [orderType, setOrderType] = useState('limit');
  const [price, setPrice] = useState('');
  const [amount, setAmount] = useState('');
  const [slippage, setSlippage] = useState('0.5');
  const [showPreview, setShowPreview] = useState(false);
  const [pendingTx, setPendingTx] = useState<Transaction | null>(null);
  const [expectedOutput, setExpectedOutput] = useState<{
    token: string;
    amount: number;
    slippage: number;
  } | null>(null);
  const [icebergParams, setIcebergParams] = useState({
    totalAmount: '',
    visibleAmount: '',
  });

  const handleOrderTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setOrderType(e.target.value);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!wallet) return;

    try {
      // Add nonce for MEV protection
      const nonce = crypto.getRandomValues(new Uint8Array(32));
      
      // Calculate expected output based on current market conditions
      const output = {
        token: 'SOL',
        amount: parseFloat(amount),
        slippage: parseFloat(slippage),
      };
      
      // Create order data for commitment
      const orderData = {
        type: orderType,
        price: orderType === 'market' ? null : parseFloat(price),
        amount: parseFloat(amount),
        iceberg: orderType === 'iceberg' ? icebergParams : null,
        nonce: Array.from(nonce),
      };

      // Create and prepare the transaction with MEV protection
      const transaction = await createProtectedTransaction(orderData);
      
      setExpectedOutput(output);
      setPendingTx(transaction);
      setShowPreview(true);
    } catch (error) {
      console.error('Error preparing trade:', error);
    }
  };

  const handleConfirmTrade = async () => {
    if (!pendingTx || !wallet) return;

    try {
      // Submit the transaction
      // This would be replaced with actual transaction submission logic
      setShowPreview(false);
      setPendingTx(null);
    } catch (error) {
      console.error('Error executing trade:', error);
    }
  };

  const handleCancelTrade = () => {
    setShowPreview(false);
    setPendingTx(null);
  };

  return (
    <div className="p-4 bg-white rounded-lg shadow">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block mb-2">Order Type:</label>
          <select
            value={orderType}
            onChange={handleOrderTypeChange}
            className="w-full p-2 border rounded"
          >
            <option value="limit">Limit</option>
            <option value="market">Market</option>
            <option value="iceberg">Iceberg</option>
          </select>
        </div>

        {orderType !== 'market' && (
          <div>
            <label className="block mb-2">Price:</label>
            <input
              type="text"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              className="w-full p-2 border rounded"
              placeholder="Enter price"
            />
          </div>
        )}

        <div>
          <label className="block mb-2">Amount:</label>
          <input
            type="text"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="w-full p-2 border rounded"
            placeholder="Enter amount"
          />
        </div>

        <div>
          <label className="block mb-2">Slippage Tolerance (%):</label>
          <input
            type="number"
            value={slippage}
            onChange={(e) => setSlippage(e.target.value)}
            className="w-full p-2 border rounded"
            step="0.1"
            min="0.1"
            max="5"
          />
        </div>

        {orderType === 'iceberg' && (
          <div className="space-y-4">
            <div>
              <label className="block mb-2">Total Amount:</label>
              <input
                type="text"
                value={icebergParams.totalAmount}
                onChange={(e) => setIcebergParams({
                  ...icebergParams,
                  totalAmount: e.target.value
                })}
                className="w-full p-2 border rounded"
                placeholder="Enter total amount"
              />
            </div>
            <div>
              <label className="block mb-2">Visible Amount:</label>
              <input
                type="text"
                value={icebergParams.visibleAmount}
                onChange={(e) => setIcebergParams({
                  ...icebergParams,
                  visibleAmount: e.target.value
                })}
                className="w-full p-2 border rounded"
                placeholder="Enter visible amount"
              />
            </div>
          </div>
        )}

        <button
          type="submit"
          className="w-full bg-blue-500 text-white p-2 rounded hover:bg-blue-600 disabled:bg-gray-300"
          disabled={!wallet || !amount || (orderType !== 'market' && !price)}
        >
          Review Trade
        </button>
      </form>

      {showPreview && pendingTx && expectedOutput && (
        <TransactionPreview
          transaction={pendingTx}
          expectedOutput={expectedOutput}
          onConfirm={handleConfirmTrade}
          onCancel={handleCancelTrade}
        />
      )}
    </div>
  );
};

export default TradeForm;