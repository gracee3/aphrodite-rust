use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::middleware::rate_limit::{rate_limit_layer, limits};
use crate::services::ChartServicePool;

mod health;
mod render;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub service_pool: Arc<ChartServicePool>,
}

/// Create the main router with all required state
pub fn create_router() -> Router {
    // Initialize service pool
    let config = crate::config::Config::from_env();
    let service_pool = ChartServicePool::new(
        config.service_pool_size,
        config.swiss_ephemeris_path.map(std::path::PathBuf::from),
        config.cache_size,
        config.default_wheel_json_path,
    )
    .expect("Failed to create service pool");

    let state = AppState {
        service_pool: Arc::new(service_pool),
    };

    Router::new()
        .route("/", get(health::api_info))
        .route("/health", get(health::health_check))
        .route("/api/v1/render", post(render::render_ephemeris).layer(rate_limit_layer(limits::render())))
        .route("/api/v1/render/chartspec", post(render::render_chartspec).layer(rate_limit_layer(limits::chartspec())))
        .with_state(state)
}

