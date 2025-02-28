import React from 'react';
import { Transaction } from '@solana/web3.js';

interface TransactionPreviewProps {
  transaction: Transaction;
  expectedOutput?: {
    token: string;
    amount: number;
    slippage: number;
  };
  onConfirm: () => void;
  onCancel: () => void;
}

const TransactionPreview: React.FC<TransactionPreviewProps> = ({
  transaction,
  expectedOutput,
  onConfirm,
  onCancel,
}) => {
  const formatInstruction = (ix: any) => {
    return {
      program: ix.programId.toString(),
      data: ix.data ? `0x${Buffer.from(ix.data).toString('hex')}` : 'No data',
      accounts: ix.keys.map((key: any) => ({
        address: key.pubkey.toString(),
        isSigner: key.isSigner,
        isWritable: key.isWritable,
      })),
    };
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center">
      <div className="bg-white rounded-lg p-6 max-w-2xl w-full mx-4">
        <h2 className="text-xl font-bold mb-4">Transaction Preview</h2>
        
        {expectedOutput && (
          <div className="mb-4 p-4 bg-blue-50 rounded">
            <h3 className="font-semibold">Expected Output</h3>
            <p>Token: {expectedOutput.token}</p>
            <p>Amount: {expectedOutput.amount}</p>
            <p>Maximum Slippage: {expectedOutput.slippage}%</p>
          </div>
        )}

        <div className="mb-4">
          <h3 className="font-semibold mb-2">Transaction Details</h3>
          <div className="bg-gray-50 p-4 rounded overflow-auto max-h-60">
            {transaction.instructions.map((ix, index) => {
              const formatted = formatInstruction(ix);
              return (
                <div key={index} className="mb-4 last:mb-0">
                  <p className="font-mono text-sm">Program: {formatted.program}</p>
                  <p className="font-mono text-sm">Data: {formatted.data}</p>
                  <details>
                    <summary className="cursor-pointer text-sm text-gray-600">
                      Show Accounts ({formatted.accounts.length})
                    </summary>
                    <ul className="mt-2 space-y-1">
                      {formatted.accounts.map((acc, i) => (
                        <li key={i} className="text-sm font-mono">
                          {acc.address}
                          {acc.isSigner && ' (Signer)'}
                          {acc.isWritable && ' (Writable)'}
                        </li>
                      ))}
                    </ul>
                  </details>
                </div>
              );
            })}
          </div>
        </div>

        <div className="flex space-x-4">
          <button
            onClick={onConfirm}
            className="bg-green-500 text-white px-4 py-2 rounded hover:bg-green-600"
          >
            Confirm Transaction
          </button>
          <button
            onClick={onCancel}
            className="bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
};

export default TransactionPreview;