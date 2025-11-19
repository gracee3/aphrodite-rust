use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

/// API error types
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::CalculationError(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::CalculationError(_) => "CALCULATION_ERROR",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            ApiError::InternalError(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let correlation_id = Uuid::new_v4().to_string();
        let status = self.status_code();
        let error_response = json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
                "correlation_id": correlation_id,
            }
        });

        tracing::error!(
            error = %self,
            correlation_id = %correlation_id,
            "API error occurred"
        );

        (status, Json(error_response)).into_response()
    }
}

/// Convert core library errors to API errors
impl From<aphrodite_core::ephemeris::adapter::EphemerisError> for ApiError {
    fn from(err: aphrodite_core::ephemeris::adapter::EphemerisError) -> Self {
        match err {
            aphrodite_core::ephemeris::adapter::EphemerisError::FileNotFound { path, message } => {
                ApiError::InternalError(format!("Ephemeris file not found at {}: {}", path, message))
            }
            aphrodite_core::ephemeris::adapter::EphemerisError::InvalidHouseSystem { system, valid } => {
                ApiError::ValidationError(format!(
                    "Invalid house system: {}. Valid systems: {:?}",
                    system, valid
                ))
            }
            aphrodite_core::ephemeris::adapter::EphemerisError::InvalidAyanamsa { ayanamsa, valid } => {
                ApiError::ValidationError(format!(
                    "Invalid ayanamsa: {}. Valid ayanamsas: {:?}",
                    ayanamsa, valid
                ))
            }
            aphrodite_core::ephemeris::adapter::EphemerisError::CalculationFailed {
                planet_id,
                datetime,
                message,
            } => ApiError::CalculationError(format!(
                "Failed to calculate position for {} at {}: {}",
                planet_id, datetime, message
            )),
            aphrodite_core::ephemeris::adapter::EphemerisError::HouseCalculationFailed { message } => {
                ApiError::CalculationError(format!("House calculation failed: {}", message))
            }
        }
    }
}

impl From<aphrodite_core::layout::WheelDefinitionError> for ApiError {
    fn from(err: aphrodite_core::layout::WheelDefinitionError) -> Self {
        match err {
            aphrodite_core::layout::WheelDefinitionError::InvalidJson(msg) => {
                ApiError::ValidationError(format!("Invalid wheel definition JSON: {}", msg))
            }
            aphrodite_core::layout::WheelDefinitionError::ValidationError(msg) => {
                ApiError::ValidationError(format!("Wheel definition validation error: {}", msg))
            }
            aphrodite_core::layout::WheelDefinitionError::MissingField(field) => {
                ApiError::ValidationError(format!("Missing required field in wheel definition: {}", field))
            }
            aphrodite_core::layout::WheelDefinitionError::InvalidFieldValue(msg) => {
                ApiError::ValidationError(format!("Invalid field value in wheel definition: {}", msg))
            }
        }
    }
}

