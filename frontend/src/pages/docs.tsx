import React from 'react';
import { useRouter } from 'next/router';

const ApiDocs = () => {
  const router = useRouter();
  const { section = 'overview' } = router.query;

  const endpoints = {
    orderbook: [
      {
        method: 'GET',
        path: '/api/v1/orderbook/markets',
        description: 'Get all available markets',
        response: `{
  "markets": [
    {
      "id": "SOL-USDC",
      "baseToken": "SOL",
      "quoteToken": "USDC",
      "minOrderSize": "0.1"
    }
  ]
}`,
      },
      {
        method: 'GET',
        path: '/api/v1/orderbook/{marketId}',
        description: 'Get order book for a specific market',
        response: `{
  "bids": [...],
  "asks": [...]
}`,
      },
      {
        method: 'POST',
        path: '/api/v1/orderbook/order',
        description: 'Place a new order',
        body: `{
  "marketId": "SOL-USDC",
  "side": "buy",
  "price": "100.5",
  "amount": "1.5",
  "type": "limit"
}`,
      },
    ],
    amm: [
      {
        method: 'GET',
        path: '/api/v1/amm/pools',
        description: 'Get all liquidity pools',
        response: `{
  "pools": [
    {
      "id": "SOL-USDC",
      "tokenA": "SOL",
      "tokenB": "USDC",
      "reserveA": "1000.5",
      "reserveB": "100500.25",
      "fee": "0.003"
    }
  ]
}`,
      },
      {
        method: 'POST',
        path: '/api/v1/amm/swap/quote',
        description: 'Get swap quote',
        body: `{
  "poolId": "SOL-USDC",
  "tokenIn": "SOL",
  "amountIn": "1.0"
}`,
      },
    ],
    bridge: [
      {
        method: 'GET',
        path: '/api/v1/bridge/status',
        description: 'Get bridge status',
        response: `{
  "status": "active",
  "supportedChains": ["ethereum", "solana"],
  "lastBlock": 12345678
}`,
      },
      {
        method: 'POST',
        path: '/api/v1/bridge/transfer',
        description: 'Initiate cross-chain transfer',
        body: `{
  "fromChain": "ethereum",
  "toChain": "solana",
  "token": "USDC",
  "amount": "100.0",
  "recipient": "address"
}`,
      },
    ],
    governance: [
      {
        method: 'GET',
        path: '/api/v1/governance/proposals',
        description: 'Get all proposals',
        response: `{
  "proposals": [
    {
      "id": "1",
      "title": "Proposal Title",
      "status": "active",
      "votesFor": "1000000",
      "votesAgainst": "500000"
    }
  ]
}`,
      },
      {
        method: 'POST',
        path: '/api/v1/governance/vote',
        description: 'Cast vote on proposal',
        body: `{
  "proposalId": "1",
  "vote": true,
  "amount": "100.0"
}`,
      },
    ],
  };

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-3xl font-bold mb-8">API Documentation</h1>

      <div className="flex gap-8">
        {/* Navigation Sidebar */}
        <div className="w-64">
          <div className="bg-white rounded-lg shadow p-4">
            <h2 className="text-lg font-semibold mb-4">Sections</h2>
            <nav className="space-y-2">
              <a
                href="#overview"
                className={`block p-2 rounded ${
                  section === 'overview' ? 'bg-blue-50 text-blue-600' : ''
                }`}
              >
                Overview
              </a>
              {Object.keys(endpoints).map((key) => (
                <a
                  key={key}
                  href={`#${key}`}
                  className={`block p-2 rounded capitalize ${
                    section === key ? 'bg-blue-50 text-blue-600' : ''
                  }`}
                >
                  {key}
                </a>
              ))}
            </nav>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1">
          <div className="bg-white rounded-lg shadow p-6">
            {section === 'overview' ? (
              <div>
                <h2 className="text-2xl font-bold mb-4">API Overview</h2>
                <p className="mb-4">
                  The DEX API provides endpoints for trading, liquidity provision,
                  cross-chain transfers, and governance. All endpoints are secured
                  and require appropriate authentication.
                </p>
                <h3 className="text-xl font-semibold mb-2">Authentication</h3>
                <p className="mb-4">
                  Include your API key in the headers of all requests:
                  <br />
                  <code className="bg-gray-100 p-1 rounded">
                    Authorization: Bearer YOUR_API_KEY
                  </code>
                </p>
                <h3 className="text-xl font-semibold mb-2">Rate Limits</h3>
                <p>
                  - Public endpoints: 100 requests per minute
                  <br />
                  - Trading endpoints: 300 requests per minute
                  <br />
                  - Websocket connections: 5 concurrent connections
                </p>
              </div>
            ) : (
              <div>
                <h2 className="text-2xl font-bold mb-6 capitalize">{section} API</h2>
                {endpoints[section as keyof typeof endpoints]?.map((endpoint, index) => (
                  <div key={index} className="mb-8 pb-8 border-b last:border-0">
                    <div className="flex items-center gap-4 mb-4">
                      <span className={`px-2 py-1 rounded text-white font-medium
                        ${endpoint.method === 'GET' ? 'bg-green-500' :
                          endpoint.method === 'POST' ? 'bg-blue-500' :
                          endpoint.method === 'DELETE' ? 'bg-red-500' : 'bg-gray-500'
                        }`}
                      >
                        {endpoint.method}
                      </span>
                      <code className="font-mono">{endpoint.path}</code>
                    </div>
                    <p className="mb-4 text-gray-600">{endpoint.description}</p>
                    {endpoint.body && (
                      <div className="mb-4">
                        <h4 className="font-semibold mb-2">Request Body:</h4>
                        <pre className="bg-gray-100 p-4 rounded overflow-auto">
                          {endpoint.body}
                        </pre>
                      </div>
                    )}
                    {endpoint.response && (
                      <div>
                        <h4 className="font-semibold mb-2">Response:</h4>
                        <pre className="bg-gray-100 p-4 rounded overflow-auto">
                          {endpoint.response}
                        </pre>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default ApiDocs;