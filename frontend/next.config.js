/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  env: {
    API_URL: process.env.API_URL || 'http://localhost:8080/api/v1',
    SOLANA_RPC_URL: process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com',
    SOLANA_NETWORK: process.env.SOLANA_NETWORK || 'devnet',
  }
}

module.exports = nextConfig