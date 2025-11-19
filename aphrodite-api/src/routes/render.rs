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
    let mut service = state.chart_service.lock().await;
    let response = service.get_positions(&request).await?;
    Ok(Json(response))
}

/// Render ChartSpec endpoint
pub async fn render_chartspec(
    State(state): State<AppState>,
    Json(request): Json<RenderRequest>,
) -> Result<Json<ChartSpecResponse>, ApiError> {
    let mut service = state.chart_service.lock().await;
    let spec = service.get_chartspec(&request, None).await?;
    
    // Also get ephemeris response for backward compatibility
    let ephemeris = service.get_positions(&request).await?;
    
    Ok(Json(ChartSpecResponse {
        spec,
        ephemeris,
    }))
}

