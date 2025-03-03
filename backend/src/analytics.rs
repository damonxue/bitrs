use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use crate::tokenomics::TokenEconomics;
use crate::models::{TokenValidation, RewardValidation, BuybackValidation};
use crate::ws::WsServer;

#[derive(Debug, Serialize, Clone)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_volume_24h: u64,
    pub total_fees_collected: u64,
    pub total_rewards_distributed: u64,
    pub total_tokens_burned: u64,
    pub active_lp_count: u64,
    pub avg_apr: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct PoolMetrics {
    pub pool_id: String,
    pub volume_24h: u64,
    pub tvl: u64,
    pub apr: f64,
    pub lp_count: u64,
    pub reward_rate: f64,
}

pub struct Analytics {
    token_economics: Arc<TokenEconomics>,
    metrics: Arc<RwLock<SystemMetrics>>,
    pool_metrics: Arc<RwLock<Vec<PoolMetrics>>>,
    ws_server: Arc<WsServer>,
}

impl Analytics {
    pub fn new(token_economics: Arc<TokenEconomics>, ws_server: Arc<WsServer>) -> Self {
        Self {
            token_economics,
            metrics: Arc::new(RwLock::new(SystemMetrics {
                timestamp: Utc::now(),
                total_volume_24h: 0,
                total_fees_collected: 0,
                total_rewards_distributed: 0,
                total_tokens_burned: 0,
                active_lp_count: 0,
                avg_apr: 0.0,
            })),
            pool_metrics: Arc::new(RwLock::new(Vec::new())),
            ws_server,
        }
    }

    pub async fn start_monitoring(&self) {
        let metrics = self.metrics.clone();
        let pool_metrics = self.pool_metrics.clone();
        let token_economics = self.token_economics.clone();
        let ws_server = self.ws_server.clone();

        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::update_metrics(
                    metrics.clone(),
                    pool_metrics.clone(),
                    token_economics.clone(),
                    ws_server.clone(),
                ).await {
                    log::error!("Error updating metrics: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await; // Update every 5 minutes
            }
        });
    }

    async fn update_metrics(
        metrics: Arc<RwLock<SystemMetrics>>,
        pool_metrics: Arc<RwLock<Vec<PoolMetrics>>>,
        token_economics: Arc<TokenEconomics>,
        ws_server: Arc<WsServer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Update system-wide metrics
        let mut current_metrics = metrics.write().await;
        current_metrics.timestamp = Utc::now();
        
        // Fetch and update pool metrics
        let mut current_pool_metrics = pool_metrics.write().await;
        current_pool_metrics.clear();

        // Add pool metrics collection logic here
        // This would involve querying each pool's state
        
        // Broadcast updated metrics
        let metrics_clone = current_metrics.clone();
        ws_server.broadcast_metrics(metrics_clone);

        Ok(())
    }

    pub async fn get_system_metrics(&self) -> SystemMetrics {
        // Clone the data inside the lock to avoid returning a reference to locked data
        self.metrics.read().await.clone()
    }

    pub async fn get_pool_metrics(&self) -> Vec<PoolMetrics> {
        // Clone the data inside the lock to avoid returning a reference to locked data
        self.pool_metrics.read().await.clone()
    }

    pub async fn record_trade(&self, volume: u64, fees: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_volume_24h += volume;
        metrics.total_fees_collected += fees;
    }

    pub async fn record_reward_distribution(&self, amount: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_rewards_distributed += amount;
    }

    pub async fn record_token_burn(&self, amount: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_tokens_burned += amount;
    }
}