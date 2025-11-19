use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use tower::Service;
use tower_governor::{Governor, GovernorConfigBuilder};

/// Rate limit configuration per endpoint
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
}

impl RateLimitConfig {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
        }
    }
}

/// Create a rate limit layer for an endpoint
pub fn rate_limit_layer(config: RateLimitConfig) -> Governor<GovernorConfigBuilder> {
    let quota = Quota::per_minute(NonZeroU32::new(config.requests_per_minute).unwrap());
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(quota)
            .finish()
            .unwrap(),
    );

    Governor::new(governor_conf)
}

/// Default rate limits
pub mod limits {
    use super::RateLimitConfig;

    pub fn render() -> RateLimitConfig {
        RateLimitConfig::new(50) // 50 requests per minute
    }

    pub fn chartspec() -> RateLimitConfig {
        RateLimitConfig::new(50) // 50 requests per minute
    }

    pub fn health() -> RateLimitConfig {
        RateLimitConfig::new(100) // 100 requests per minute
    }
}

