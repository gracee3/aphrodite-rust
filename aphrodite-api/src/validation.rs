use crate::error::ApiError;
use crate::schemas::request::{ChartSettings, LayerConfig, RenderRequest, Subject};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Valid house systems
const VALID_HOUSE_SYSTEMS: &[&str] = &[
    "placidus",
    "whole_sign",
    "koch",
    "equal",
    "regiomontanus",
    "campanus",
    "alcabitius",
    "morinus",
];

/// Valid ayanamsas
const VALID_AYANAMSAS: &[&str] = &[
    "lahiri",
    "chitrapaksha",
    "fagan_bradley",
    "de_luce",
    "raman",
    "krishnamurti",
    "yukteshwar",
    "djwhal_khul",
    "true_citra",
    "true_revati",
    "aryabhata",
    "aryabhata_mean_sun",
];

/// Valid planet/object names
const VALID_PLANETS: &[&str] = &[
    "sun",
    "moon",
    "mercury",
    "venus",
    "mars",
    "jupiter",
    "saturn",
    "uranus",
    "neptune",
    "pluto",
    "chiron",
    "north_node",
    "south_node",
];

/// Valid layer kinds
const VALID_LAYER_KINDS: &[&str] = &["natal", "transit", "progressed"];

/// Date range limits (reasonable bounds for astrology calculations)
const MIN_YEAR: i32 = -1000; // 1000 BCE
const MAX_YEAR: i32 = 3000;  // 3000 CE

/// Orb setting limits
const MIN_ORB: f64 = 0.0;
const MAX_ORB: f64 = 30.0;

/// Request validator
pub struct RequestValidator;

impl RequestValidator {
    /// Validate a complete render request
    pub fn validate_request(request: &RenderRequest) -> Result<(), ApiError> {
        Self::validate_subjects(&request.subjects)?;
        Self::validate_settings(&request.settings)?;
        Self::validate_layer_config(&request.layer_config, &request.subjects)?;
        Ok(())
    }

    /// Validate subjects
    pub fn validate_subjects(subjects: &[Subject]) -> Result<(), ApiError> {
        if subjects.is_empty() {
            return Err(ApiError::ValidationError(
                "At least one subject is required".to_string(),
            ));
        }

        let mut subject_ids = std::collections::HashSet::new();
        for (idx, subject) in subjects.iter().enumerate() {
            // Validate subject ID
            if subject.id.is_empty() {
                return Err(ApiError::ValidationError(format!(
                    "Subject[{}].id cannot be empty",
                    idx
                )));
            }

            if subject_ids.contains(&subject.id) {
                return Err(ApiError::ValidationError(format!(
                    "Duplicate subject ID: {}",
                    subject.id
                )));
            }
            subject_ids.insert(subject.id.clone());

            // Validate birth date if provided
            if let Some(birth_dt_str) = &subject.birth_date_time {
                let birth_dt = Self::parse_and_validate_datetime(birth_dt_str)
                    .map_err(|e| ApiError::ValidationError(format!(
                        "Subject[{}].birthDateTime: {}",
                        idx, e
                    )))?;
                Self::validate_date_range(birth_dt)?;
            }

            // Validate location if provided
            if let Some(loc) = &subject.location {
                Self::validate_location(loc.lat, loc.lon)
                    .map_err(|e| ApiError::ValidationError(format!(
                        "Subject[{}].location: {}",
                        idx, e
                    )))?;
            }
        }

        Ok(())
    }

    /// Validate chart settings
    pub fn validate_settings(settings: &ChartSettings) -> Result<(), ApiError> {
        // Validate zodiac type
        if settings.zodiac_type != "tropical" && settings.zodiac_type != "sidereal" {
            return Err(ApiError::ValidationError(format!(
                "Invalid zodiacType: {}. Must be 'tropical' or 'sidereal'",
                settings.zodiac_type
            )));
        }

        // Validate house system
        if !VALID_HOUSE_SYSTEMS.contains(&settings.house_system.as_str()) {
            return Err(ApiError::ValidationError(format!(
                "Invalid houseSystem: {}. Valid systems: {:?}",
                settings.house_system, VALID_HOUSE_SYSTEMS
            )));
        }

        // Validate ayanamsa if provided
        if let Some(ayanamsa) = &settings.ayanamsa {
            if !VALID_AYANAMSAS.contains(&ayanamsa.as_str()) {
                return Err(ApiError::ValidationError(format!(
                    "Invalid ayanamsa: {}. Valid ayanamsas: {:?}",
                    ayanamsa, VALID_AYANAMSAS
                )));
            }
        }

        // Validate orb settings
        Self::validate_orb_setting("conjunction", settings.orb_settings.conjunction)?;
        Self::validate_orb_setting("opposition", settings.orb_settings.opposition)?;
        Self::validate_orb_setting("trine", settings.orb_settings.trine)?;
        Self::validate_orb_setting("square", settings.orb_settings.square)?;
        Self::validate_orb_setting("sextile", settings.orb_settings.sextile)?;

        // Validate include objects
        for (idx, obj) in settings.include_objects.iter().enumerate() {
            if !VALID_PLANETS.contains(&obj.as_str()) {
                return Err(ApiError::ValidationError(format!(
                    "Invalid includeObjects[{}]: {}. Valid objects: {:?}",
                    idx, obj, VALID_PLANETS
                )));
            }
        }

        Ok(())
    }

    /// Validate layer configuration
    pub fn validate_layer_config(
        layer_config: &HashMap<String, LayerConfig>,
        subjects: &[Subject],
    ) -> Result<(), ApiError> {
        if layer_config.is_empty() {
            return Err(ApiError::ValidationError(
                "At least one layer must be configured".to_string(),
            ));
        }

        let subject_ids: std::collections::HashSet<_> =
            subjects.iter().map(|s| &s.id).collect();

        for (layer_id, config) in layer_config {
            // Validate layer kind
            if !VALID_LAYER_KINDS.contains(&config.kind.as_str()) {
                return Err(ApiError::ValidationError(format!(
                    "Layer '{}': Invalid kind '{}'. Valid kinds: {:?}",
                    layer_id, config.kind, VALID_LAYER_KINDS
                )));
            }

            // Validate based on layer kind
            match config.kind.as_str() {
                "natal" => {
                    if let Some(subject_id) = &config.subject_id {
                        if !subject_ids.contains(subject_id) {
                            return Err(ApiError::ValidationError(format!(
                                "Layer '{}': subjectId '{}' not found in subjects",
                                layer_id, subject_id
                            )));
                        }
                    } else {
                        return Err(ApiError::ValidationError(format!(
                            "Layer '{}': natal layer must specify a subjectId",
                            layer_id
                        )));
                    }
                }
                "transit" => {
                    if config.explicit_date_time.is_none() {
                        return Err(ApiError::ValidationError(format!(
                            "Layer '{}': transit layer must specify explicitDateTime",
                            layer_id
                        )));
                    }
                    if let Some(dt_str) = &config.explicit_date_time {
                        let dt = Self::parse_and_validate_datetime(dt_str)
                            .map_err(|e| ApiError::ValidationError(format!(
                                "Layer '{}'.explicitDateTime: {}",
                                layer_id, e
                            )))?;
                        Self::validate_date_range(dt)?;
                    }
                }
                "progressed" => {
                    // Similar to transit, requires explicitDateTime
                    if config.explicit_date_time.is_none() {
                        return Err(ApiError::ValidationError(format!(
                            "Layer '{}': progressed layer must specify explicitDateTime",
                            layer_id
                        )));
                    }
                }
                _ => {}
            }

            // Validate location if provided
            if let Some(loc) = &config.location {
                Self::validate_location(loc.lat, loc.lon)
                    .map_err(|e| ApiError::ValidationError(format!(
                        "Layer '{}'.location: {}",
                        layer_id, e
                    )))?;
            }
        }

        Ok(())
    }

    /// Validate a single orb setting
    fn validate_orb_setting(name: &str, value: f64) -> Result<(), ApiError> {
        if value < MIN_ORB || value > MAX_ORB {
            return Err(ApiError::ValidationError(format!(
                "orbSettings.{} must be between {} and {} degrees, got {}",
                name, MIN_ORB, MAX_ORB, value
            )));
        }
        if !value.is_finite() {
            return Err(ApiError::ValidationError(format!(
                "orbSettings.{} must be a finite number, got {}",
                name, value
            )));
        }
        Ok(())
    }

    /// Validate location coordinates
    fn validate_location(lat: f64, lon: f64) -> Result<(), String> {
        if !lat.is_finite() {
            return Err("latitude must be a finite number".to_string());
        }
        if !lon.is_finite() {
            return Err("longitude must be a finite number".to_string());
        }
        if lat < -90.0 || lat > 90.0 {
            return Err(format!("latitude must be between -90 and 90, got {}", lat));
        }
        if lon < -180.0 || lon > 180.0 {
            return Err(format!("longitude must be between -180 and 180, got {}", lon));
        }
        Ok(())
    }

    /// Parse and validate datetime string
    fn parse_and_validate_datetime(dt_str: &str) -> Result<DateTime<Utc>, String> {
        let dt = chrono::DateTime::parse_from_rfc3339(dt_str)
            .or_else(|_| dt_str.parse::<DateTime<Utc>>().map(|dt| dt.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())))
            .map_err(|e| format!("Failed to parse datetime '{}': {}", dt_str, e))?
            .with_timezone(&Utc);

        Ok(dt)
    }

    /// Validate date is within reasonable range
    fn validate_date_range(dt: DateTime<Utc>) -> Result<(), ApiError> {
        use chrono::Datelike;
        let year = dt.year();
        if year < MIN_YEAR || year > MAX_YEAR {
            return Err(ApiError::ValidationError(format!(
                "Date year {} is outside valid range ({} to {})",
                year, MIN_YEAR, MAX_YEAR
            )));
        }
        Ok(())
    }
}

