use crate::error::ApiError;
use crate::schemas::request::{ChartSettings, LayerConfig, RenderRequest, Subject};
use crate::schemas::response::{
    EphemerisResponse, HousePositions, LayerPositions, LayerResponse, PlanetPosition,
};
use aphrodite_core::aspects::{AspectCalculator, AspectSettings};
use aphrodite_core::ephemeris::{
    EphemerisSettings, GeoLocation, LayerContext, SwissEphemerisAdapter,
};
use aphrodite_core::layout::{load_wheel_definition_from_json, WheelAssembler};
use aphrodite_core::rendering::ChartSpecGenerator;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;

/// Chart calculation service
pub struct ChartService {
    adapter: SwissEphemerisAdapter,
}

impl ChartService {
    /// Create a new chart service
    pub fn new(ephemeris_path: Option<PathBuf>) -> Result<Self, ApiError> {
        let mut adapter = SwissEphemerisAdapter::new(ephemeris_path)
            .map_err(|e| ApiError::InternalError(format!("Failed to create adapter: {}", e)))?;
        Ok(Self { adapter })
    }

    /// Get ephemeris positions for a render request
    pub async fn get_positions(
        &mut self,
        request: &RenderRequest,
    ) -> Result<EphemerisResponse, ApiError> {
        // Merge settings
        let mut settings = request.settings.clone();
        for (key, value) in &request.settings_override {
            // Simple merge - in production, use a proper merge strategy
            if key == "zodiacType" {
                if let Some(zodiac) = value.as_str() {
                    settings.zodiac_type = zodiac.to_string();
                }
            }
            // Add more field merges as needed
        }

        // Resolve layer contexts
        let layer_contexts = self.resolve_layer_contexts(&request.subjects, &request.layer_config, &settings)?;

        // Calculate positions
        let mut positions_by_layer = HashMap::new();
        for ctx in &layer_contexts {
            let positions = self
                .adapter
                .calc_positions(ctx.datetime, ctx.location.clone(), &ctx.settings)
                .map_err(|e| ApiError::CalculationError(format!("Failed to calculate positions: {}", e)))?;
            positions_by_layer.insert(ctx.layer_id.clone(), positions);
        }

        // Build response
        let mut layers_response = HashMap::new();
        for ctx in layer_contexts {
            if let Some(positions) = positions_by_layer.get(&ctx.layer_id) {
                let planets: HashMap<String, PlanetPosition> = positions
                    .planets
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            PlanetPosition {
                                lon: v.lon,
                                lat: v.lat,
                                speed_lon: Some(v.speed_lon),
                                retrograde: Some(v.retrograde),
                            },
                        )
                    })
                    .collect();

                let houses = positions.houses.as_ref().map(|h| HousePositions {
                    system: h.system.clone(),
                    cusps: h.cusps.clone(),
                    angles: h.angles.clone(),
                });

                layers_response.insert(
                    ctx.layer_id.clone(),
                    LayerResponse {
                        id: ctx.layer_id.clone(),
                        kind: ctx.kind.clone(),
                        date_time: ctx.datetime,
                        location: ctx.location.as_ref().map(|loc| crate::schemas::request::Location {
                            name: None,
                            lat: loc.lat,
                            lon: loc.lon,
                        }),
                        positions: LayerPositions {
                            planets,
                            houses,
                        },
                    },
                );
            }
        }

        Ok(EphemerisResponse {
            layers: layers_response,
            settings: settings.clone(),
            vedic: None, // Phase 6
        })
    }

    /// Get ChartSpec for a render request
    pub async fn get_chartspec(
        &mut self,
        request: &RenderRequest,
        wheel_json: Option<&str>,
    ) -> Result<aphrodite_core::rendering::ChartSpec, ApiError> {
        // Get ephemeris positions first
        let ephemeris_response = self.get_positions(request).await?;

        // Get settings from request
        let settings = &request.settings;

        // Convert to core types for aspect calculation
        let mut positions_by_layer = HashMap::new();
        for (layer_id, layer) in &ephemeris_response.layers {
            let mut planets = HashMap::new();
            for (planet_id, planet_pos) in &layer.positions.planets {
                planets.insert(
                    planet_id.clone(),
                    aphrodite_core::ephemeris::PlanetPosition {
                        lon: planet_pos.lon,
                        lat: planet_pos.lat,
                        speed_lon: planet_pos.speed_lon.unwrap_or(0.0),
                        retrograde: planet_pos.retrograde.unwrap_or(false),
                    },
                );
            }

            let houses = layer.positions.houses.as_ref().map(|h| {
                aphrodite_core::ephemeris::HousePositions {
                    system: h.system.clone(),
                    cusps: h.cusps.clone(),
                    angles: h.angles.clone(),
                }
            });

            positions_by_layer.insert(
                layer_id.clone(),
                aphrodite_core::ephemeris::LayerPositions { planets, houses },
            );
        }

        // Get settings from ephemeris response
        let settings = &ephemeris_response.settings;

        // Calculate aspects
        let calculator = AspectCalculator::new();
        let orb_settings: HashMap<String, f64> = [
            ("conjunction".to_string(), settings.orb_settings.conjunction),
            ("opposition".to_string(), settings.orb_settings.opposition),
            ("trine".to_string(), settings.orb_settings.trine),
            ("square".to_string(), settings.orb_settings.square),
            ("sextile".to_string(), settings.orb_settings.sextile),
        ]
        .into_iter()
        .collect();

        let aspect_settings = AspectSettings {
            orb_settings,
            include_objects: settings.include_objects.clone(),
            only_major: None,
        };

        let aspect_sets = calculator.compute_all_aspect_sets(&positions_by_layer, &aspect_settings);

        // Load wheel definition
        // For now, use a simple default wheel JSON
        let default_wheel_json = wheel_json.unwrap_or(r#"
        {
          "name": "Standard Natal Wheel",
          "rings": [
            {
              "slug": "ring_signs",
              "type": "signs",
              "label": "Zodiac Signs",
              "orderIndex": 0,
              "radiusInner": 0.85,
              "radiusOuter": 1.0,
              "dataSource": { "kind": "static_zodiac" }
            },
            {
              "slug": "ring_houses",
              "type": "houses",
              "label": "Houses",
              "orderIndex": 1,
              "radiusInner": 0.75,
              "radiusOuter": 0.85,
              "dataSource": { "kind": "layer_houses", "layerId": "natal" }
            },
            {
              "slug": "ring_planets",
              "type": "planets",
              "label": "Natal Planets",
              "orderIndex": 2,
              "radiusInner": 0.55,
              "radiusOuter": 0.75,
              "dataSource": { "kind": "layer_planets", "layerId": "natal" }
            }
          ]
        }
        "#);

        let wheel_def_with_presets = load_wheel_definition_from_json(default_wheel_json)
            .map_err(|e| ApiError::ValidationError(format!("Invalid wheel definition: {}", e)))?;

        // Assemble wheel
        let wheel = WheelAssembler::build_wheel(
            &wheel_def_with_presets.wheel,
            &positions_by_layer,
            &aspect_sets,
            if settings.include_objects.is_empty() {
                None
            } else {
                Some(&settings.include_objects)
            },
        );

        // Generate ChartSpec
        let generator = ChartSpecGenerator::new();
        let spec = generator.generate(&wheel, &aspect_sets, 800.0, 800.0);

        Ok(spec)
    }

    /// Resolve layer contexts from request
    fn resolve_layer_contexts(
        &self,
        subjects: &[Subject],
        layer_config: &HashMap<String, LayerConfig>,
        settings: &ChartSettings,
    ) -> Result<Vec<LayerContext>, ApiError> {
        let mut contexts = Vec::new();

        for (layer_id, config) in layer_config {
            let dt_utc = match config.kind.as_str() {
                "natal" => {
                    let subject_id = config
                        .subject_id
                        .as_ref()
                        .ok_or_else(|| {
                            ApiError::ValidationError(format!(
                                "Layer '{}': natal layer must specify a 'subjectId'",
                                layer_id
                            ))
                        })?;

                    let subject = subjects
                        .iter()
                        .find(|s| s.id == *subject_id)
                        .ok_or_else(|| {
                            ApiError::ValidationError(format!(
                                "Layer '{}': subjectId '{}' not found",
                                layer_id, subject_id
                            ))
                        })?;

                    let birth_dt = subject
                        .birth_date_time
                        .as_ref()
                        .ok_or_else(|| {
                            ApiError::ValidationError(format!(
                                "Layer '{}': subject '{}' missing 'birthDateTime'",
                                layer_id, subject_id
                            ))
                        })?;

                    parse_datetime(birth_dt, subject.birth_timezone.as_deref())?
                }
                "transit" => {
                    config
                        .explicit_date_time
                        .as_ref()
                        .ok_or_else(|| {
                            ApiError::ValidationError(format!(
                                "Layer '{}': transit layer must specify 'explicitDateTime'",
                                layer_id
                            ))
                        })
                        .and_then(|dt| parse_datetime(dt, None))?
                }
                _ => {
                    return Err(ApiError::ValidationError(format!(
                        "Layer '{}': unsupported layer kind '{}'",
                        layer_id, config.kind
                    )));
                }
            };

            let location = config
                .location
                .as_ref()
                .or_else(|| {
                    // Try to get from subject
                    if let Some(subject_id) = &config.subject_id {
                        subjects
                            .iter()
                            .find(|s| s.id == *subject_id)
                            .and_then(|s| s.location.as_ref())
                    } else {
                        None
                    }
                })
                .map(|loc| GeoLocation {
                    lat: loc.lat,
                    lon: loc.lon,
                });

            let ephemeris_settings = EphemerisSettings {
                zodiac_type: settings.zodiac_type.clone(),
                ayanamsa: settings.ayanamsa.clone(),
                house_system: settings.house_system.clone(),
                include_objects: settings.include_objects.clone(),
            };

            contexts.push(LayerContext {
                layer_id: layer_id.clone(),
                kind: config.kind.clone(),
                datetime: dt_utc,
                location,
                settings: ephemeris_settings,
            });
        }

        Ok(contexts)
    }
}

/// Parse datetime string to UTC
fn parse_datetime(dt_str: &str, tz_str: Option<&str>) -> Result<DateTime<Utc>, ApiError> {
    // Simple parser - in production, use a more robust date parser
    let dt = chrono::DateTime::parse_from_rfc3339(dt_str)
        .or_else(|_| {
            // Try ISO 8601 format
            dt_str.parse::<DateTime<Utc>>()
        })
        .map_err(|e| ApiError::ValidationError(format!("Failed to parse datetime '{}': {}", dt_str, e)))?;

    Ok(dt.with_timezone(&Utc))
}

