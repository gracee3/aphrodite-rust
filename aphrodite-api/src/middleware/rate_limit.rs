use tower_governor::governor::{GovernorConfigBuilder, GovernorConfig};
use tower_governor::GovernorLayer;
use tower_governor::key_extractor::PeerIpKeyExtractor;
use std::sync::Arc;

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

/// Create a rate limit config (caller should create the layer)
pub fn rate_limit_config(config: RateLimitConfig) -> Arc<GovernorConfig<PeerIpKeyExtractor, governor::middleware::NoOpMiddleware>> {
    // Calculate per_second, ensuring it's at least 1
    let per_second = ((config.requests_per_minute as f64) / 60.0).ceil().max(1.0) as u64;
    
    Arc::new(
        GovernorConfigBuilder::default()
            .per_second(per_second)
            .burst_size(config.requests_per_minute)
            .finish()
            .expect("Failed to create rate limit config: invalid configuration")
    )
}

/// Create a rate limit layer for an endpoint
pub fn rate_limit_layer(config: RateLimitConfig) -> GovernorLayer<'static, PeerIpKeyExtractor, governor::middleware::NoOpMiddleware> {
    // Calculate per_second, ensuring it's at least 1
    let per_second = ((config.requests_per_minute as f64) / 60.0).ceil().max(1.0) as u64;
    
    // Use Box::leak to create a 'static reference
    let governor_conf = Box::leak(Box::new(
        GovernorConfigBuilder::default()
            .per_second(per_second)
            .burst_size(config.requests_per_minute)
            .finish()
            .expect("Failed to create rate limit layer: invalid configuration")
    ));

    GovernorLayer {
        config: governor_conf,
    }
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

