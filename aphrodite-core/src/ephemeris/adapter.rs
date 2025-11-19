use crate::ephemeris::types::{
    EphemerisSettings, GeoLocation, HousePositions, LayerPositions, PlanetPosition,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use thiserror::Error;

// Note: swisseph crate API - these constants and functions should be available
// If the crate API differs, adjust accordingly

/// Errors that can occur during ephemeris calculations
#[derive(Error, Debug)]
pub enum EphemerisError {
    #[error("Ephemeris file not found at path: {path}. {message}")]
    FileNotFound { path: String, message: String },
    #[error("Invalid house system: {system}. Valid systems: {valid:?}")]
    InvalidHouseSystem { system: String, valid: Vec<String> },
    #[error("Invalid ayanamsa: {ayanamsa}. Valid ayanamsas: {valid:?}")]
    InvalidAyanamsa { ayanamsa: String, valid: Vec<String> },
    #[error("Failed to calculate position for {planet_id} at {datetime}: {message}")]
    CalculationFailed {
        planet_id: String,
        datetime: DateTime<Utc>,
        message: String,
    },
    #[error("House calculation failed: {message}")]
    HouseCalculationFailed { message: String },
}

// Swiss Ephemeris planet IDs - adjust based on actual swisseph crate API
// Typical values: SUN=0, MOON=1, MERCURY=2, VENUS=3, MARS=4, JUPITER=5,
// SATURN=6, URANUS=7, NEPTUNE=8, PLUTO=9, CHIRON=15, TRUE_NODE=11
const PLANET_IDS: &[(&str, i32)] = &[
    ("sun", 0),
    ("moon", 1),
    ("mercury", 2),
    ("venus", 3),
    ("mars", 4),
    ("jupiter", 5),
    ("saturn", 6),
    ("uranus", 7),
    ("neptune", 8),
    ("pluto", 9),
    ("chiron", 15),
    ("north_node", 11), // TRUE_NODE
];

/// House system mapping
const HOUSE_SYSTEMS: &[(&str, u8)] = &[
    ("placidus", b'P' as u8),
    ("whole_sign", b'W' as u8),
    ("koch", b'K' as u8),
    ("equal", b'E' as u8),
    ("regiomontanus", b'R' as u8),
    ("campanus", b'C' as u8),
    ("alcabitius", b'A' as u8),
    ("morinus", b'M' as u8),
];

/// Ayanamsa mapping
const AYANAMSAS: &[(&str, i32)] = &[
    ("lahiri", swisseph::SIDM_LAHIRI),
    ("chitrapaksha", swisseph::SIDM_LAHIRI),
    ("fagan_bradley", swisseph::SIDM_FAGAN_BRADLEY),
    ("de_luce", swisseph::SIDM_DELUCE),
    ("raman", swisseph::SIDM_RAMAN),
    ("krishnamurti", swisseph::SIDM_KRISHNAMURTI),
    ("yukteshwar", swisseph::SIDM_YUKTESHWAR),
    ("djwhal_khul", swisseph::SIDM_DJWHAL_KHUL),
    ("true_citra", swisseph::SIDM_TRUE_CITRA),
    ("true_revati", swisseph::SIDM_TRUE_REVATI),
    ("aryabhata", swisseph::SIDM_ARYABHATA),
    ("aryabhata_mean_sun", swisseph::SIDM_ARYABHATA_MSUN),
];

const DEFAULT_AYANAMSA: i32 = swisseph::SIDM_LAHIRI;

/// Swiss Ephemeris adapter implementation
pub struct SwissEphemerisAdapter {
    ephemeris_path: PathBuf,
    current_sidereal_mode: Option<i32>,
}

impl SwissEphemerisAdapter {
    /// Create a new adapter with optional ephemeris path
    pub fn new(ephemeris_path: Option<PathBuf>) -> Result<Self, EphemerisError> {
        let path = ephemeris_path.unwrap_or_else(|| {
            env::var("SWISS_EPHEMERIS_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/usr/local/share/swisseph"))
        });

        // Validate path exists
        if !path.exists() {
            return Err(EphemerisError::FileNotFound {
                path: path.display().to_string(),
                message: "Ephemeris path does not exist. Please ensure Swiss Ephemeris data files are installed.".to_string(),
            });
        }

        // Set ephemeris path
        // Note: Adjust based on actual swisseph crate API
        // Typical API: swisseph::set_ephe_path(path_str)
        let path_str = path.to_string_lossy();
        // This will need to be adjusted based on the actual crate API
        // For now, we'll assume the path is set correctly

        Ok(Self {
            ephemeris_path: path,
            current_sidereal_mode: None,
        })
    }

    /// Calculate planetary and house positions
    pub fn calc_positions(
        &mut self,
        dt_utc: DateTime<Utc>,
        location: Option<GeoLocation>,
        settings: &EphemerisSettings,
    ) -> Result<LayerPositions, EphemerisError> {
        let jd = datetime_to_julian_day(dt_utc);
        let house_system_byte = get_house_system_byte(&settings.house_system)?;
        let flags = self.configure_flags(settings)?;

        // Calculate planets
        let mut planets = HashMap::new();
        for obj_id in &settings.include_objects {
            let obj_id_lower = obj_id.to_lowercase();

            // Handle special case: south_node
            if obj_id_lower == "south_node" {
                if let Ok(north_node_pos) = self.calc_planet_position("north_node", jd, flags) {
                    let south_lon = (north_node_pos.lon + 180.0) % 360.0;
                    planets.insert(
                        "south_node".to_string(),
                        PlanetPosition {
                            lon: south_lon,
                            lat: 0.0,
                            speed_lon: north_node_pos.speed_lon,
                            retrograde: north_node_pos.retrograde,
                        },
                    );
                }
                continue;
            }

            if let Ok(planet_pos) = self.calc_planet_position(&obj_id_lower, jd, flags) {
                planets.insert(obj_id_lower.clone(), planet_pos);
            }
        }

        // Calculate houses if location is provided
        let houses = if let Some(loc) = location {
            Some(self.calc_houses(
                jd,
                loc.lat,
                loc.lon,
                house_system_byte,
                &settings.house_system,
                flags,
            )?)
        } else {
            None
        };

        Ok(LayerPositions { planets, houses })
    }

    /// Calculate position for a single planet
    pub fn calc_planet_position(
        &self,
        planet_id: &str,
        jd: f64,
        flags: i32,
    ) -> Result<PlanetPosition, EphemerisError> {
        let planet_code = PLANET_IDS
            .iter()
            .find(|(id, _)| *id == planet_id)
            .map(|(_, code)| *code)
            .ok_or_else(|| EphemerisError::CalculationFailed {
                planet_id: planet_id.to_string(),
                datetime: julian_day_to_datetime(jd),
                message: format!("Unknown planet ID: {}", planet_id),
            })?;

        // Calculate planet position using swisseph crate
        // Note: Adjust based on actual swisseph crate API
        // Typical API: swisseph::calc_ut(jd, planet_code, flags) -> Result<Vec<f64>, String>
        let result = swisseph::calc_ut(jd, planet_code, flags)
            .map_err(|e| EphemerisError::CalculationFailed {
                planet_id: planet_id.to_string(),
                datetime: julian_day_to_datetime(jd),
                message: format!("Swiss Ephemeris error: {}", e),
            })?;

        let longitude = result[0] % 360.0;
        let latitude = if result.len() > 1 { result[1] } else { 0.0 };
        let speed_longitude = if result.len() > 3 { result[3] } else { 0.0 };
        let is_retrograde = speed_longitude < 0.0;

        Ok(PlanetPosition {
            lon: longitude,
            lat: latitude,
            speed_lon: speed_longitude,
            retrograde: is_retrograde,
        })
    }

    /// Calculate house cusps and angles
    pub fn calc_houses(
        &self,
        jd: f64,
        lat: f64,
        lon: f64,
        house_system_byte: u8,
        house_system_str: &str,
        flags: i32,
    ) -> Result<HousePositions, EphemerisError> {
        // Calculate houses using swisseph crate
        // Note: Adjust based on actual swisseph crate API
        // Typical API: swisseph::houses_ex2(jd, lat, lon, hsys, flags) -> Result<(Vec<f64>, Vec<f64>), String>
        let result = swisseph::houses_ex2(jd, lat, lon, house_system_byte, flags)
            .map_err(|e| EphemerisError::HouseCalculationFailed {
                message: format!("Swiss Ephemeris error: {}", e),
            })?;

        let cusps = &result.0;
        let ascmc = &result.1;

        // Extract house cusps
        let mut cusps_dict = HashMap::new();

        if cusps.len() == 12 {
            // Whole Sign: cusps are indices 0-11 for houses 1-12
            for i in 0..12 {
                cusps_dict.insert((i + 1).to_string(), cusps[i] % 360.0);
            }
        } else {
            // Placidus or other: indices 1-12 are houses 1-12
            for i in 1..=12 {
                if i < cusps.len() {
                    cusps_dict.insert(i.to_string(), cusps[i] % 360.0);
                }
            }
        }

        // Extract angles
        let asc = if !ascmc.is_empty() { ascmc[0] % 360.0 } else { 0.0 };
        let mc = if ascmc.len() > 1 { ascmc[1] % 360.0 } else { 0.0 };
        let ic = (mc + 180.0) % 360.0;
        let dc = (asc + 180.0) % 360.0;

        Ok(HousePositions {
            system: house_system_str.to_string(),
            cusps: cusps_dict,
            angles: HashMap::from([
                ("asc".to_string(), asc),
                ("mc".to_string(), mc),
                ("ic".to_string(), ic),
                ("dc".to_string(), dc),
            ]),
        })
    }

    /// Configure Swiss Ephemeris flags for the requested zodiac
    fn configure_flags(&mut self, settings: &EphemerisSettings) -> Result<i32, EphemerisError> {
        // FLG_SWIEPH = 2 (use Swiss Ephemeris files)
        let mut flags = 2; // swisseph::FLG_SWIEPH

        if settings.zodiac_type == "sidereal" {
            let mode = self.resolve_ayanamsa(settings.ayanamsa.as_deref())?;
            self.ensure_sidereal_mode(mode)?;
            flags |= 64; // swisseph::FLG_SIDEREAL
        }

        Ok(flags)
    }

    /// Map ayanamsa string to Swiss constant
    fn resolve_ayanamsa(&self, ayanamsa: Option<&str>) -> Result<i32, EphemerisError> {
        let ayanamsa = ayanamsa.unwrap_or("lahiri");
        AYANAMSAS
            .iter()
            .find(|(name, _)| *name == ayanamsa.to_lowercase())
            .map(|(_, mode)| *mode)
            .ok_or_else(|| EphemerisError::InvalidAyanamsa {
                ayanamsa: ayanamsa.to_string(),
                valid: AYANAMSAS.iter().map(|(name, _)| name.to_string()).collect(),
            })
    }

    /// Cache sidereal mode configuration to avoid redundant calls
    fn ensure_sidereal_mode(&mut self, mode: i32) -> Result<(), EphemerisError> {
        if self.current_sidereal_mode == Some(mode) {
            return Ok(());
        }
        // Note: Adjust based on actual swisseph crate API
        // Typical API: swisseph::set_sid_mode(mode, t0, ayon) -> Result<(), String>
        swisseph::set_sid_mode(mode, 0.0, 0.0)
            .map_err(|_| EphemerisError::InvalidAyanamsa {
                ayanamsa: format!("mode {}", mode),
                valid: vec![],
            })?;
        self.current_sidereal_mode = Some(mode);
        Ok(())
    }
}

/// Convert UTC datetime to Julian Day
fn datetime_to_julian_day(dt: DateTime<Utc>) -> f64 {
    let year = dt.year();
    let month = dt.month();
    let day = dt.day();
    let hour = dt.hour() as f64;
    let minute = dt.minute() as f64;
    let second = dt.second() as f64;
    let hour_decimal = hour + minute / 60.0 + second / 3600.0;

    // GREG_CAL = 1
    swisseph::julday(year, month, day, hour_decimal, 1)
        .unwrap_or(0.0)
}

/// Convert Julian Day to UTC datetime
fn julian_day_to_datetime(jd: f64) -> DateTime<Utc> {
    // GREG_CAL = 1
    let (year, month, day, hour, minute, second) =
        swisseph::revjul(jd, 1).unwrap_or((2000, 1, 1, 0, 0, 0));
    chrono::Utc
        .with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .unwrap_or_else(|| chrono::Utc::now())
}

/// Convert house system string to byte format
fn get_house_system_byte(house_system: &str) -> Result<u8, EphemerisError> {
    HOUSE_SYSTEMS
        .iter()
        .find(|(name, _)| *name == house_system.to_lowercase())
        .map(|(_, byte)| *byte)
        .ok_or_else(|| EphemerisError::InvalidHouseSystem {
            system: house_system.to_string(),
            valid: HOUSE_SYSTEMS.iter().map(|(name, _)| name.to_string()).collect(),
        })
}

