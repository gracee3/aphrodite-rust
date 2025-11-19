use axum::{extract::State, Json};
use crate::error::ApiError;
use crate::routes::AppState;
use crate::schemas::request::RenderRequest;
use crate::schemas::response::{ChartSpecResponse, EphemerisResponse};

/// Render ephemeris positions endpoint
pub async fn render_ephemeris(
    State(state): State<AppState>,
    Json(request): Json<RenderRequest>,
) -> Result<Json<EphemerisResponse>, ApiError> {
    let service = state.service_pool.get_service();
    let mut service = service.lock().await;
    let response = service.get_positions(&request).await?;
    Ok(Json(response))
}

/// Render ChartSpec endpoint
pub async fn render_chartspec(
    State(state): State<AppState>,
    Json(request): Json<RenderRequest>,
) -> Result<Json<ChartSpecResponse>, ApiError> {
    let service = state.service_pool.get_service();
    let mut service = service.lock().await;
    let (spec, ephemeris) = service.get_chartspec(&request, None).await?;
    
    Ok(Json(ChartSpecResponse {
        spec,
        ephemeris,
    }))
}

