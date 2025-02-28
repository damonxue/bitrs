import React, { useEffect, useState } from 'react';
import { useWallet } from '@/context/WalletContext';
import { web3 } from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';

interface Proposal {
  pubkey: PublicKey;
  account: {
    proposer: PublicKey;
    title: string;
    description: string;
    executionDeadline: number;
    status: 'Active' | 'Executed' | 'Cancelled';
    yesVotes: number;
    noVotes: number;
    executed: boolean;
  };
}

const Governance = () => {
  const { wallet } = useWallet();
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [newProposal, setNewProposal] = useState({
    title: '',
    description: '',
    executionDeadline: new Date(),
  });

  useEffect(() => {
    if (wallet) {
      fetchProposals();
    }
  }, [wallet]);

  const fetchProposals = async () => {
    // Implement proposal fetching logic
  };

  const handleCreateProposal = async (e: React.FormEvent) => {
    e.preventDefault();
    // Implement proposal creation logic
  };

  const handleVote = async (proposal: Proposal, vote: boolean) => {
    // Implement voting logic
  };

  const handleExecuteProposal = async (proposal: Proposal) => {
    // Implement proposal execution logic
  };

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-6">Governance</h1>

      {/* Create Proposal Form */}
      <div className="bg-white p-4 rounded-lg shadow mb-6">
        <h2 className="text-xl font-semibold mb-4">Create Proposal</h2>
        <form onSubmit={handleCreateProposal} className="space-y-4">
          <div>
            <label className="block mb-1">Title</label>
            <input
              type="text"
              value={newProposal.title}
              onChange={(e) => setNewProposal({...newProposal, title: e.target.value})}
              className="w-full p-2 border rounded"
              required
            />
          </div>
          <div>
            <label className="block mb-1">Description</label>
            <textarea
              value={newProposal.description}
              onChange={(e) => setNewProposal({...newProposal, description: e.target.value})}
              className="w-full p-2 border rounded"
              rows={4}
              required
            />
          </div>
          <div>
            <label className="block mb-1">Execution Deadline</label>
            <input
              type="datetime-local"
              value={newProposal.executionDeadline.toISOString().slice(0, 16)}
              onChange={(e) => setNewProposal({...newProposal, executionDeadline: new Date(e.target.value)})}
              className="w-full p-2 border rounded"
              required
            />
          </div>
          <button
            type="submit"
            className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
            disabled={!wallet}
          >
            Create Proposal
          </button>
        </form>
      </div>

      {/* Proposals List */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Active Proposals</h2>
        {proposals.map((proposal) => (
          <div key={proposal.pubkey.toString()} className="bg-white p-4 rounded-lg shadow">
            <h3 className="text-lg font-semibold">{proposal.account.title}</h3>
            <p className="text-gray-600 mt-2">{proposal.account.description}</p>
            <div className="mt-4">
              <span className="text-sm text-gray-500">
                Yes: {proposal.account.yesVotes} | No: {proposal.account.noVotes}
              </span>
            </div>
            <div className="mt-4 space-x-4">
              {proposal.account.status === 'Active' && (
                <>
                  <button
                    onClick={() => handleVote(proposal, true)}
                    className="bg-green-500 text-white px-4 py-2 rounded hover:bg-green-600"
                    disabled={!wallet}
                  >
                    Vote Yes
                  </button>
                  <button
                    onClick={() => handleVote(proposal, false)}
                    className="bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600"
                    disabled={!wallet}
                  >
                    Vote No
                  </button>
                  {proposal.account.yesVotes > proposal.account.noVotes && (
                    <button
                      onClick={() => handleExecuteProposal(proposal)}
                      className="bg-purple-500 text-white px-4 py-2 rounded hover:bg-purple-600"
                      disabled={!wallet}
                    >
                      Execute
                    </button>
                  )}
                </>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default Governance;