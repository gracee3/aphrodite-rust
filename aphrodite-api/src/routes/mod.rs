use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::services::ChartService;

mod health;
mod render;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub chart_service: Arc<Mutex<ChartService>>,
}

/// Create the main router
pub fn create_router() -> Router<AppState> {
    // Initialize chart service
    let config = crate::config::Config::from_env();
    let chart_service = ChartService::new(
        config.swiss_ephemeris_path.map(std::path::PathBuf::from),
    )
    .expect("Failed to create chart service");

    let state = AppState {
        chart_service: Arc::new(Mutex::new(chart_service)),
    };

    Router::new()
        .route("/", get(health::api_info))
        .route("/health", get(health::health_check))
        .route("/api/render", post(render::render_ephemeris))
        .route("/api/render/chartspec", post(render::render_chartspec))
        .with_state(state)
}

