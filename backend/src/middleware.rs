use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, Ready};
use futures::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use crate::analytics::Analytics;
use crate::rate_limiter::RateLimiter;

pub struct MonitoringMiddleware {
    analytics: Arc<Analytics>,
    rate_limiter: Arc<RateLimiter>,
}

impl MonitoringMiddleware {
    pub fn new(analytics: Arc<Analytics>) -> Self {
        // Configure different rate limits for different endpoint types
        let rate_limiter = Arc::new(RateLimiter::new(100, Duration::from_secs(60))); // Default limit

        Self { 
            analytics,
            rate_limiter,
        }
    }

    fn get_rate_limit_key(&self, req: &ServiceRequest) -> String {
        // Combine IP and path for rate limiting
        let ip = req.connection_info().realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        let path = req.path().to_string();
        format!("{}:{}", ip, path)
    }

    fn get_rate_limit_config(&self, path: &str) -> (u32, Duration) {
        // Configure different rate limits based on endpoint type
        if path.starts_with("/api/v1/orderbook") {
            (300, Duration::from_secs(60)) // Higher limit for trading
        } else if path.starts_with("/api/v1/analytics") {
            (60, Duration::from_secs(60)) // Lower limit for analytics
        } else {
            (100, Duration::from_secs(60)) // Default limit
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for MonitoringMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MonitoringMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MonitoringMiddlewareService {
            service,
            analytics: self.analytics.clone(),
            rate_limiter: self.rate_limiter.clone(),
        })
    }
}

pub struct MonitoringMiddlewareService<S> {
    service: S,
    analytics: Arc<Analytics>,
    rate_limiter: Arc<RateLimiter>,
}

impl<S, B> Service<ServiceRequest> for MonitoringMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let analytics = self.analytics.clone();
        let rate_limiter = self.rate_limiter.clone();
        let start_time = std::time::Instant::now();
        let path = req.path().to_string();
        let method = req.method().to_string();
        let rate_limit_key = format!("{}:{}", 
            req.connection_info().realip_remote_addr().unwrap_or("unknown"),
            path
        );

        Box::pin(async move {
            // Check rate limit
            if !rate_limiter.is_allowed(&rate_limit_key).await {
                let remaining = rate_limiter.get_remaining(&rate_limit_key).await;
                return Ok(ServiceRequest::into_response(req, HttpResponse::TooManyRequests()
                    .append_header(("X-RateLimit-Remaining", remaining.to_string()))
                    .finish()
                    .into_body()));
            }

            // Process request
            let response = self.service.call(req).await?;
            let duration = start_time.elapsed();
            
            // Record metrics
            analytics.record_request_metrics(
                &path,
                &method,
                response.status().as_u16(),
                duration.as_millis() as u64,
            ).await;

            Ok(response)
        })
    }
}