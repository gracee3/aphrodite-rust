use axum::Json;
use crate::schemas::response::{ApiInfoResponse, HealthResponse};

/// API info endpoint
pub async fn api_info() -> Json<ApiInfoResponse> {
    Json(ApiInfoResponse {
        name: "Aphrodite API".to_string(),
        version: "0.1.0".to_string(),
        description: "Rust-based astrology charting API".to_string(),
    })
}

/// Health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
    })
}

