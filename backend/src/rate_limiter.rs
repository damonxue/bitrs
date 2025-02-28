use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct RateWindow {
    count: u32,
    start_time: Instant,
}

pub struct RateLimiter {
    windows: Arc<RwLock<HashMap<String, RateWindow>>>,
    max_requests: u32,
    window_size: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_size: Duration) -> Self {
        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_size,
        }
    }

    pub async fn is_allowed(&self, key: &str) -> bool {
        let mut windows = self.windows.write().await;
        let now = Instant::now();

        // Clean up expired windows
        windows.retain(|_, window| {
            now.duration_since(window.start_time) <= self.window_size
        });

        let window = windows
            .entry(key.to_string())
            .or_insert_with(|| RateWindow {
                count: 0,
                start_time: now,
            });

        if now.duration_since(window.start_time) > self.window_size {
            // Reset window if it's expired
            window.count = 1;
            window.start_time = now;
            true
        } else if window.count < self.max_requests {
            // Increment counter if within limits
            window.count += 1;
            true
        } else {
            false
        }
    }

    pub async fn get_remaining(&self, key: &str) -> u32 {
        let windows = self.windows.read().await;
        if let Some(window) = windows.get(key) {
            let now = Instant::now();
            if now.duration_since(window.start_time) > self.window_size {
                self.max_requests
            } else {
                self.max_requests.saturating_sub(window.count)
            }
        } else {
            self.max_requests
        }
    }
}