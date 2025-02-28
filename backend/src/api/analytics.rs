use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;
use crate::analytics::{SystemMetrics, PoolMetrics};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
struct AnalyticsResponse {
    timestamp: DateTime<Utc>,
    metrics: SystemMetrics,
    pools: Vec<PoolMetrics>,
}

#[get("/analytics")]
async fn get_analytics(state: web::Data<AppState>) -> impl Responder {
    let metrics = state.analytics.get_system_metrics().await;
    let pool_metrics = state.analytics.get_pool_metrics().await;

    let response = AnalyticsResponse {
        timestamp: Utc::now(),
        metrics,
        pools: pool_metrics,
    };

    HttpResponse::Ok().json(response)
}

#[get("/analytics/pools/{pool_id}")]
async fn get_pool_analytics(
    pool_id: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let pool_metrics = state.analytics.get_pool_metrics().await;
    if let Some(metrics) = pool_metrics.iter().find(|p| p.pool_id == *pool_id) {
        HttpResponse::Ok().json(metrics)
    } else {
        HttpResponse::NotFound().finish()
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_analytics)
       .service(get_pool_analytics);
}