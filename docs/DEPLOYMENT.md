# BitRS Deployment Guide

## Prerequisites

- Rust 1.70.0 or higher
- Node.js 18.x or higher
- Solana CLI tools
- PostgreSQL 14.x or higher
- Docker (optional)

## Environment Setup

### 1. Solana Configuration
```bash
# Install Solana CLI tools
sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"

# Configure Solana CLI
solana config set --url https://api.devnet.solana.com
```

### 2. Database Setup
```bash
# Create database
createdb bitrs_dev

# Run migrations
cd backend
cargo run --bin migrate
```

### 3. Smart Contracts Deployment

```bash
# Build programs
cd programs
cargo build-bpf

# Deploy programs
solana program deploy target/deploy/dex_core.so
solana program deploy target/deploy/liquidity_pools.so
solana program deploy target/deploy/staking.so
solana program deploy target/deploy/bridge.so
```

## Component Deployment

### Backend Service

1. Configuration
```bash
cd backend
cp .env.example .env
# Edit .env with your settings
```

2. Build and Run
```bash
cargo build --release
./target/release/bitrs-backend
```

### Frontend Application

1. Install Dependencies
```bash
cd frontend
npm install
```

2. Build and Run
```bash
npm run build
npm start
```

### Relayer Service

1. Configuration
```bash
cd relayer
cp config.example.toml config.toml
# Edit config.toml with your settings
```

2. Build and Run
```bash
cargo build --release
./target/release/bitrs-relayer
```

## Docker Deployment

### Build Images
```bash
docker-compose build
```

### Run Services
```bash
docker-compose up -d
```

## Monitoring Setup

1. Install Prometheus and Grafana
2. Import dashboard templates from `monitoring/dashboards`
3. Configure alerting rules

## Security Considerations

1. Configure firewalls
2. Set up SSL certificates
3. Implement rate limiting
4. Enable monitoring alerts

## Maintenance

### Database Backups
```bash
# Setup automatic backups
pg_dump bitrs_dev > backup.sql
```

### Log Rotation
```bash
# Configure logrotate
sudo vim /etc/logrotate.d/bitrs
```

### Health Checks
```bash
# Add to crontab
*/5 * * * * curl -f http://localhost:8080/health
```

## Troubleshooting

### Common Issues

1. Connection Timeouts
   - Check network configuration
   - Verify RPC endpoints

2. Database Errors
   - Verify connection string
   - Check migrations

3. Smart Contract Errors
   - Verify program IDs
   - Check account permissions