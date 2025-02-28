use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::cross_chain_monitor::{CrossChainMonitor, Chain};
use crate::price_monitor::PriceMonitor;
use crate::errors::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSegment {
    pub chain: Chain,
    pub dex: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: f64,
    pub amount_out: f64,
    pub fee: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRoute {
    pub segments: Vec<RouteSegment>,
    pub total_output: f64,
    pub total_fees: f64,
    pub estimated_time: u32, // seconds
    pub success_probability: f64,
}

pub struct SmartOrderRouter {
    cross_chain_monitor: Arc<CrossChainMonitor>,
    price_monitor: Arc<PriceMonitor>,
    route_cache: Arc<RwLock<Vec<(TradeRoute, u64)>>>, // Route and timestamp
}

impl SmartOrderRouter {
    pub fn new(
        cross_chain_monitor: Arc<CrossChainMonitor>,
        price_monitor: Arc<PriceMonitor>,
    ) -> Self {
        Self {
            cross_chain_monitor,
            price_monitor,
            route_cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn find_best_route(
        &self,
        token_in: &str,
        token_out: &str,
        amount: f64,
        max_hops: u8,
        include_cross_chain: bool,
    ) -> Result<TradeRoute, ApiError> {
        // Check cache first
        if let Some(route) = self.check_cache(token_in, token_out, amount).await {
            return Ok(route);
        }

        let mut routes = Vec::new();

        // Get single-chain routes
        let single_chain_routes = self.find_single_chain_routes(
            token_in,
            token_out,
            amount,
            max_hops,
        ).await?;
        routes.extend(single_chain_routes);

        // Get cross-chain routes if enabled
        if include_cross_chain {
            let cross_chain_routes = self.find_cross_chain_routes(
                token_in,
                token_out,
                amount,
                max_hops,
            ).await?;
            routes.extend(cross_chain_routes);
        }

        // Select the best route based on output amount and other factors
        let best_route = self.select_best_route(routes).await?;
        
        // Cache the route
        self.cache_route(best_route.clone()).await;

        Ok(best_route)
    }

    async fn check_cache(
        &self,
        token_in: &str,
        token_out: &str,
        amount: f64,
    ) -> Option<TradeRoute> {
        let cache = self.route_cache.read().await;
        let current_time = chrono::Utc::now().timestamp() as u64;
        
        // Look for recent cached routes (less than 10 seconds old)
        cache.iter()
            .filter(|(_, timestamp)| current_time - timestamp < 10)
            .find(|(route, _)| {
                let first_segment = route.segments.first()?;
                let last_segment = route.segments.last()?;
                first_segment.token_in == token_in &&
                last_segment.token_out == token_out &&
                (first_segment.amount_in - amount).abs() / amount < 0.01 // Within 1% of requested amount
            })
            .map(|(route, _)| route.clone())
    }

    async fn cache_route(&self, route: TradeRoute) {
        let mut cache = self.route_cache.write().await;
        let current_time = chrono::Utc::now().timestamp() as u64;
        
        // Remove old entries
        cache.retain(|(_, timestamp)| current_time - timestamp < 30);
        
        // Add new route
        cache.push((route, current_time));
    }

    async fn find_single_chain_routes(
        &self,
        token_in: &str,
        token_out: &str,
        amount: f64,
        max_hops: u8,
    ) -> Result<Vec<TradeRoute>, ApiError> {
        let mut routes = Vec::new();

        // Get current prices and liquidity from price monitor
        let price_data = self.price_monitor.get_price_history(60).await;
        
        // Implementation of pathfinding algorithm for single-chain routes
        // This would typically use Dijkstra's or similar algorithm to find optimal paths

        Ok(routes)
    }

    async fn find_cross_chain_routes(
        &self,
        token_in: &str,
        token_out: &str,
        amount: f64,
        max_hops: u8,
    ) -> Result<Vec<TradeRoute>, ApiError> {
        let mut routes = Vec::new();

        // Get cross-chain price data
        let cross_chain_prices = self.cross_chain_monitor.get_latest_prices().await;
        
        // Get current arbitrage opportunities
        let arbitrage_ops = self.cross_chain_monitor.get_arbitrage_opportunities().await;

        // Implementation of cross-chain routing algorithm
        // This would consider bridge fees and latency

        Ok(routes)
    }

    async fn select_best_route(
        &self,
        routes: Vec<TradeRoute>,
    ) -> Result<TradeRoute, ApiError> {
        if routes.is_empty() {
            return Err(ApiError::NotFound("No valid routes found".to_string()));
        }

        // Score each route based on multiple factors:
        // - Total output amount
        // - Total fees
        // - Estimated execution time
        // - Success probability
        let scored_routes: Vec<(f64, TradeRoute)> = routes.into_iter()
            .map(|route| {
                let score = self.calculate_route_score(&route);
                (score, route)
            })
            .collect();

        // Return the route with the highest score
        Ok(scored_routes
            .into_iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, route)| route)
            .unwrap())
    }

    fn calculate_route_score(&self, route: &TradeRoute) -> f64 {
        let output_weight = 0.4;
        let fee_weight = 0.3;
        let time_weight = 0.2;
        let probability_weight = 0.1;

        let output_score = route.total_output;
        let fee_score = 1.0 / (1.0 + route.total_fees);
        let time_score = 1.0 / (1.0 + (route.estimated_time as f64 / 60.0));
        let probability_score = route.success_probability;

        output_score * output_weight +
        fee_score * fee_weight +
        time_score * time_weight +
        probability_score * probability_weight
    }
}