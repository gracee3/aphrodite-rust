use aphrodite_core::rendering::ChartSpec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Planet position from ephemeris
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetPosition {
    pub lon: f64,
    pub lat: f64,
    #[serde(rename = "speedLon", skip_serializing_if = "Option::is_none")]
    pub speed_lon: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrograde: Option<bool>,
}

/// House positions from ephemeris
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HousePositions {
    pub system: String,
    #[serde(default)]
    pub cusps: HashMap<String, f64>, // "1".."12"
    #[serde(default)]
    pub angles: HashMap<String, f64>, // asc, mc, ic, dc
}

/// Positions for a single layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerPositions {
    #[serde(default)]
    pub planets: HashMap<String, PlanetPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub houses: Option<HousePositions>,
}

/// Layer response with positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResponse {
    pub id: String,
    pub kind: String, // "natal", "transit", "progressed"
    #[serde(rename = "dateTime")]
    pub date_time: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<crate::schemas::request::Location>,
    pub positions: LayerPositions,
}

/// Ephemeris response - only positions and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemerisResponse {
    #[serde(default)]
    pub layers: HashMap<String, LayerResponse>,
    pub settings: crate::schemas::request::ChartSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vedic: Option<serde_json::Value>, // Placeholder for Phase 6
}

/// ChartSpec response - complete chart specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSpecResponse {
    pub spec: ChartSpec,
    pub ephemeris: EphemerisResponse, // For backward compatibility
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// API info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfoResponse {
    pub name: String,
    pub version: String,
    pub description: String,
}

