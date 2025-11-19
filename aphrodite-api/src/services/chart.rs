use crate::error::ApiError;
use crate::schemas::request::{ChartSettings, LayerConfig, RenderRequest, Subject, VedicConfig};
use crate::schemas::response::{
    EphemerisResponse, HousePositions, LayerPositions, LayerResponse, PlanetPosition,
    VedicPayload, VedicLayerData, NakshatraLayer, WesternLayerData,
};
use aphrodite_core::aspects::{AspectCalculator, AspectSettings};
use aphrodite_core::ephemeris::{
    EphemerisSettings, GeoLocation, LayerContext, SwissEphemerisAdapter,
};
use aphrodite_core::layout::{load_wheel_definition_from_json, WheelAssembler};
use aphrodite_core::rendering::ChartSpecGenerator;
use aphrodite_core::vedic::{
    annotate_layer_nakshatras, build_varga_layers, identify_yogas,
    compute_vimshottari_dasha, compute_yogini_dasha, compute_ashtottari_dasha, compute_kalachakra_dasha,
    DashaLevel, VimshottariResponse,
};
use aphrodite_core::western::{
    DignitiesService, get_decan_info_from_longitude,
};
use chrono::{DateTime, Utc};
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Mutex;

/// Chart calculation service
pub struct ChartService {
    adapter: SwissEphemerisAdapter,
    ephemeris_path: Option<PathBuf>,
    cache: Mutex<LruCache<String, EphemerisResponse>>,
    default_wheel_json: String,
}

impl ChartService {
    /// Create a new chart service
    pub fn new(ephemeris_path: Option<PathBuf>, cache_size: usize, default_wheel_json_path: Option<String>) -> Result<Self, ApiError> {
        let path_for_adapter = ephemeris_path.clone();
        let mut adapter = SwissEphemerisAdapter::new(path_for_adapter)
            .map_err(|e| ApiError::InternalError(format!("Failed to create adapter: {}", e)))?; // Keep manual conversion here as it's a creation error
        let cache = Mutex::new(LruCache::new(
            NonZeroUsize::new(cache_size.max(1)).unwrap()
        ));
        
        // Load default wheel JSON from file or use embedded fallback
        let default_wheel_json = if let Some(path) = default_wheel_json_path {
            std::fs::read_to_string(&path)
                .unwrap_or_else(|_| {
                    // Fallback to embedded default if file not found
                    Self::embedded_default_wheel_json()
                })
        } else {
            Self::embedded_default_wheel_json()
        };
        
        Ok(Self { 
            adapter,
            ephemeris_path,
            cache,
            default_wheel_json,
        })
    }
    
    /// Get embedded default wheel JSON (fallback)
    fn embedded_default_wheel_json() -> String {
        r#"
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
        "#.to_string()
    }

    /// Generate a cache key from request parameters
    fn generate_cache_key(&self, request: &RenderRequest, settings: &ChartSettings) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        
        // Hash subjects
        for subject in &request.subjects {
            subject.id.hash(&mut hasher);
            if let Some(dt) = &subject.birth_date_time {
                dt.to_rfc3339().hash(&mut hasher);
            }
            if let Some(loc) = &subject.location {
                loc.lat.to_bits().hash(&mut hasher);
                loc.lon.to_bits().hash(&mut hasher);
            }
        }
        
        // Hash layer config
        for (key, value) in &request.layer_config {
            key.hash(&mut hasher);
            value.kind.hash(&mut hasher);
            if let Some(subject_id) = &value.subject_id {
                subject_id.hash(&mut hasher);
            }
            if let Some(dt) = &value.explicit_date_time {
                dt.hash(&mut hasher);
            }
            if let Some(loc) = &value.location {
                loc.lat.to_bits().hash(&mut hasher);
                loc.lon.to_bits().hash(&mut hasher);
            }
        }
        
        // Hash settings
        settings.zodiac_type.hash(&mut hasher);
        settings.house_system.hash(&mut hasher);
        if let Some(ayanamsa) = &settings.ayanamsa {
            ayanamsa.hash(&mut hasher);
        }
        settings.include_objects.hash(&mut hasher);
        
        // Hash settings_override (merged settings)
        for (key, value) in &request.settings_override {
            key.hash(&mut hasher);
            // Hash the JSON value as string for simplicity
            if let Some(s) = value.as_str() {
                s.hash(&mut hasher);
            } else if let Some(n) = value.as_f64() {
                n.to_bits().hash(&mut hasher);
            } else if let Some(b) = value.as_bool() {
                b.hash(&mut hasher);
            }
        }
        
        format!("ephemeris:{}", hasher.finish())
    }

    /// Merge settings_override into settings
    fn merge_settings_override(
        settings: &mut ChartSettings,
        settings_override: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ApiError> {
        for (key, value) in settings_override {
            match key.as_str() {
                "zodiacType" => {
                    if let Some(zodiac) = value.as_str() {
                        settings.zodiac_type = zodiac.to_string();
                    } else {
                        return Err(ApiError::ValidationError(
                            format!("zodiacType must be a string, got: {:?}", value)
                        ));
                    }
                }
                "houseSystem" => {
                    if let Some(house_system) = value.as_str() {
                        settings.house_system = house_system.to_string();
                    } else {
                        return Err(ApiError::ValidationError(
                            format!("houseSystem must be a string, got: {:?}", value)
                        ));
                    }
                }
                "ayanamsa" => {
                    if value.is_null() {
                        settings.ayanamsa = None;
                    } else if let Some(ayanamsa) = value.as_str() {
                        settings.ayanamsa = Some(ayanamsa.to_string());
                    } else {
                        return Err(ApiError::ValidationError(
                            format!("ayanamsa must be a string or null, got: {:?}", value)
                        ));
                    }
                }
                "orbSettings" => {
                    if let Some(obj) = value.as_object() {
                        if let Some(v) = obj.get("conjunction") {
                            if let Some(f) = v.as_f64() {
                                settings.orb_settings.conjunction = f;
                            }
                        }
                        if let Some(v) = obj.get("opposition") {
                            if let Some(f) = v.as_f64() {
                                settings.orb_settings.opposition = f;
                            }
                        }
                        if let Some(v) = obj.get("trine") {
                            if let Some(f) = v.as_f64() {
                                settings.orb_settings.trine = f;
                            }
                        }
                        if let Some(v) = obj.get("square") {
                            if let Some(f) = v.as_f64() {
                                settings.orb_settings.square = f;
                            }
                        }
                        if let Some(v) = obj.get("sextile") {
                            if let Some(f) = v.as_f64() {
                                settings.orb_settings.sextile = f;
                            }
                        }
                    } else {
                        return Err(ApiError::ValidationError(
                            format!("orbSettings must be an object, got: {:?}", value)
                        ));
                    }
                }
                "includeObjects" => {
                    if let Some(arr) = value.as_array() {
                        settings.include_objects = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                    } else {
                        return Err(ApiError::ValidationError(
                            format!("includeObjects must be an array, got: {:?}", value)
                        ));
                    }
                }
                "vedicConfig" => {
                    if value.is_null() {
                        settings.vedic_config = None;
                    } else if let Some(obj) = value.as_object() {
                        // Deserialize vedic config from JSON value
                        match serde_json::from_value::<VedicConfig>(value.clone()) {
                            Ok(vedic_config) => {
                                settings.vedic_config = Some(vedic_config);
                            }
                            Err(e) => {
                                return Err(ApiError::ValidationError(
                                    format!("Invalid vedicConfig: {}", e)
                                ));
                            }
                        }
                    } else {
                        return Err(ApiError::ValidationError(
                            format!("vedicConfig must be an object or null, got: {:?}", value)
                        ));
                    }
                }
                _ => {
                    // Unknown key - ignore or return error?
                    // For now, we'll ignore unknown keys to allow future extensions
                }
            }
        }
        Ok(())
    }

    /// Get ephemeris positions for a render request
    pub async fn get_positions(
        &mut self,
        request: &RenderRequest,
    ) -> Result<EphemerisResponse, ApiError> {
        // Merge settings
        let mut settings = request.settings.clone();
        self.merge_settings_override(&mut settings, &request.settings_override)?;

        // Check cache
        let cache_key = self.generate_cache_key(request, &settings);
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(cached_response) = cache.get(&cache_key) {
                return Ok(cached_response.clone());
            }
        }

        // Resolve layer contexts
        let layer_contexts = self.resolve_layer_contexts(&request.subjects, &request.layer_config, &settings)?;

        // Calculate positions - wrap CPU-bound work in spawn_blocking
        // Create a temporary adapter in the blocking task to avoid moving &mut self.adapter
        let layer_contexts_clone = layer_contexts.clone();
        let ephemeris_path = self.ephemeris_path.clone();
        let positions_by_layer = tokio::task::spawn_blocking(move || {
            let mut temp_adapter = SwissEphemerisAdapter::new(Some(ephemeris_path))
                .map_err(|e| ApiError::InternalError(format!("Failed to create temp adapter: {}", e)))?; // Keep manual conversion here
            let mut positions_by_layer = HashMap::new();
            for ctx in &layer_contexts_clone {
                let positions = temp_adapter
                    .calc_positions(ctx.datetime, ctx.location.clone(), &ctx.settings)?; // Use From trait
                positions_by_layer.insert(ctx.layer_id.clone(), positions);
            }
            Ok::<HashMap<String, aphrodite_core::ephemeris::LayerPositions>, ApiError>(positions_by_layer)
        })
        .await
        .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))??;

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

        // Calculate Vedic data if requested
        let vedic = if let Some(vedic_config) = &settings.vedic_config {
            Some(self.calculate_vedic_data(
                &positions_by_layer,
                &layer_contexts,
                vedic_config,
            )?)
        } else {
            None
        };

        // Calculate Western data (dignities and decans)
        let western = self.calculate_western_data(&positions_by_layer)?;

        let response = EphemerisResponse {
            layers: layers_response,
            settings: settings.clone(),
            vedic,
            western: if western.is_empty() { None } else { Some(western) },
        };

        // Insert into cache
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(cache_key, response.clone());
        }

        Ok(response)
    }

    /// Get ChartSpec for a render request
    /// Returns both the ChartSpec and the EphemerisResponse to avoid duplicate calculations
    pub async fn get_chartspec(
        &mut self,
        request: &RenderRequest,
        wheel_json: Option<&str>,
    ) -> Result<(aphrodite_core::rendering::ChartSpec, EphemerisResponse), ApiError> {
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
        // Use provided wheel_json, or fall back to configured default
        let wheel_json_str = wheel_json.unwrap_or(&self.default_wheel_json);

        let wheel_def_with_presets = load_wheel_definition_from_json(wheel_json_str)?; // Use From trait

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

        Ok((spec, ephemeris_response))
    }

    /// Calculate Vedic data (nakshatras, vargas, yogas, dashas)
    fn calculate_vedic_data(
        &self,
        positions_by_layer: &HashMap<String, aphrodite_core::ephemeris::LayerPositions>,
        layer_contexts: &[LayerContext],
        vedic_config: &crate::schemas::request::VedicConfig,
    ) -> Result<VedicPayload, ApiError> {
        let mut vedic_layers: HashMap<String, VedicLayerData> = HashMap::new();

        for ctx in layer_contexts {
            if let Some(positions) = positions_by_layer.get(&ctx.layer_id) {
                let mut layer_data = VedicLayerData {
                    layer_id: ctx.layer_id.clone(),
                    nakshatras: None,
                    vargas: HashMap::new(),
                    yogas: vec![],
                };

                // Calculate nakshatras if requested
                if vedic_config.include_nakshatras {
                    let placements = annotate_layer_nakshatras(
                        positions,
                        vedic_config.include_angles_in_nakshatra,
                        vedic_config.nakshatra_objects.as_ref(),
                    );
                    layer_data.nakshatras = Some(NakshatraLayer {
                        layer_id: ctx.layer_id.clone(),
                        placements,
                    });
                }

                // Calculate vargas if requested
                if !vedic_config.vargas.is_empty() {
                    let varga_layers = build_varga_layers(
                        &ctx.layer_id,
                        positions,
                        &vedic_config.vargas,
                    );
                    layer_data.vargas = varga_layers;
                }

                // Calculate yogas if requested
                if vedic_config.include_yogas {
                    layer_data.yogas = identify_yogas(positions);
                }

                vedic_layers.insert(ctx.layer_id.clone(), layer_data);
            }
        }

        // Calculate dashas if requested
        let dashas = if vedic_config.include_dashas && !vedic_config.dasha_systems.is_empty() {
            // Find natal layer for dasha calculation
            let natal_layer = layer_contexts.iter()
                .find(|ctx| ctx.kind == "natal")
                .and_then(|ctx| positions_by_layer.get(&ctx.layer_id));

            if let Some(natal_positions) = natal_layer {
                let natal_context = layer_contexts.iter()
                    .find(|ctx| ctx.kind == "natal")
                    .ok_or_else(|| ApiError::ValidationError("Natal layer required for dasha calculation".to_string()))?;

                let depth = match vedic_config.dashas_depth.as_str() {
                    "mahadasha" => DashaLevel::Mahadasha,
                    "antardasha" => DashaLevel::Antardasha,
                    "pratyantardasha" => DashaLevel::Pratyantardasha,
                    _ => DashaLevel::Pratyantardasha,
                };

                // Calculate first requested dasha system
                let dasha_system = vedic_config.dasha_systems.first()
                    .ok_or_else(|| ApiError::ValidationError("No dasha system specified".to_string()))?;

                let periods = match dasha_system.as_str() {
                    "vimshottari" => compute_vimshottari_dasha(natal_context.datetime, natal_positions, depth)
                        .map_err(|e| ApiError::CalculationError(format!("Vimshottari dasha error: {}", e)))?,
                    "yogini" => compute_yogini_dasha(natal_context.datetime, natal_positions, depth)
                        .map_err(|e| ApiError::CalculationError(format!("Yogini dasha error: {}", e)))?,
                    "ashtottari" => compute_ashtottari_dasha(natal_context.datetime, natal_positions, depth)
                        .map_err(|e| ApiError::CalculationError(format!("Ashtottari dasha error: {}", e)))?,
                    "kalachakra" => compute_kalachakra_dasha(natal_context.datetime, natal_positions, depth)
                        .map_err(|e| ApiError::CalculationError(format!("Kalachakra dasha error: {}", e)))?,
                    _ => return Err(ApiError::ValidationError(format!("Unknown dasha system: {}", dasha_system))),
                };

                Some(VimshottariResponse {
                    system: dasha_system.clone(),
                    depth,
                    birth_date_time: natal_context.datetime,
                    periods,
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(VedicPayload {
            layers: vedic_layers,
            dashas,
        })
    }

    /// Calculate Western data (dignities and decans)
    fn calculate_western_data(
        &self,
        positions_by_layer: &HashMap<String, aphrodite_core::ephemeris::LayerPositions>,
    ) -> Result<HashMap<String, WesternLayerData>, ApiError> {
        let mut western_layers: HashMap<String, WesternLayerData> = HashMap::new();
        let dignities_service = DignitiesService;
        let default_exact_exaltations = dignities_service.get_default_exact_exaltations();

        for (layer_id, positions) in positions_by_layer {
            let mut dignities: HashMap<String, Vec<aphrodite_core::western::DignityResult>> = HashMap::new();
            let mut decans: HashMap<String, aphrodite_core::western::DecanInfo> = HashMap::new();

            // Calculate dignities for all planets
            for (planet_id, planet_pos) in &positions.planets {
                let planet_dignities = dignities_service.get_dignities(
                    planet_id,
                    planet_pos.lon,
                    Some(&default_exact_exaltations),
                );
                if !planet_dignities.is_empty() {
                    dignities.insert(planet_id.clone(), planet_dignities);
                }

                // Calculate decan info
                let decan_info = get_decan_info_from_longitude(planet_pos.lon);
                decans.insert(planet_id.clone(), decan_info);
            }

            western_layers.insert(layer_id.clone(), WesternLayerData {
                layer_id: layer_id.clone(),
                dignities,
                decans,
            });
        }

        Ok(western_layers)
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

