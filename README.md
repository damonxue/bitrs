# BitRS - Decentralized Exchange Platform

BitRS is a comprehensive decentralized exchange (DEX) platform that integrates multi-chain trading, liquidity provision, staking, and cross-chain functionality. The project consists of multiple components, including a frontend user interface, backend services, smart contract programs, and a relayer.

## Project Structure

```
├── backend/          # Backend services implemented in Rust
├── frontend/         # Frontend application built with Next.js
├── programs/         # Solana smart contract programs
├── relayer/          # Cross-chain communication relayer
└── docs/             # Project documentation
```

## Core Components

### Backend Services (Rust)
- API endpoints for trading and account management
- Order routing and matching engine
- Price oracle and monitoring
- Cross-chain asset tracking
- Trade analytics and reporting

### Frontend Application (Next.js)
- User-friendly trading interface
- Portfolio management dashboard
- Trading history and analytics
- Liquidity provider interface

### Smart Contracts (Solana)
- Automated Market Maker (AMM)
- Cross-chain bridge integration
- Core DEX functionality
- Staking and rewards system

### Relayer
- Cross-chain communication handling
- Secure asset transfer verification
- Transaction validation

## Installation & Deployment

### Environment Setup
1. Copy the environment variable example file and configure it
   ```bash
   cp .env.example .env
   ```

2. Deploy services using Docker
   ```bash
   docker-compose up -d
   ```

For more detailed deployment instructions, refer to the deployment documentation.

## System Architecture

The project employs a modular architecture design, with components communicating via APIs and an event system. For a complete architecture overview, please refer to the architecture documentation.

## Development

### Backend Development
```bash
cd backend
cargo build
cargo run
```

### Frontend Development
```bash
cd frontend
npm install
npm run dev
```

## License

[License information to be added]