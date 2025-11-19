use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Location DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: Option<String>,
    pub lat: f64,
    pub lon: f64,
}

/// Subject DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: String,
    #[serde(alias = "name")]
    pub label: String,
    #[serde(rename = "birthDateTime")]
    pub birth_date_time: Option<String>,
    #[serde(rename = "birthTimezone")]
    pub birth_timezone: Option<String>,
    pub location: Option<Location>,
}

/// Orb settings DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbSettings {
    #[serde(default = "default_conjunction")]
    pub conjunction: f64,
    #[serde(default = "default_opposition")]
    pub opposition: f64,
    #[serde(default = "default_trine")]
    pub trine: f64,
    #[serde(default = "default_square")]
    pub square: f64,
    #[serde(default = "default_sextile")]
    pub sextile: f64,
}

fn default_conjunction() -> f64 {
    8.0
}
fn default_opposition() -> f64 {
    8.0
}
fn default_trine() -> f64 {
    7.0
}
fn default_square() -> f64 {
    6.0
}
fn default_sextile() -> f64 {
    4.0
}

impl Default for OrbSettings {
    fn default() -> Self {
        Self {
            conjunction: 8.0,
            opposition: 8.0,
            trine: 7.0,
            square: 6.0,
            sextile: 4.0,
        }
    }
}

/// Chart settings DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSettings {
    #[serde(rename = "zodiacType", default = "default_zodiac_type")]
    pub zodiac_type: String,
    pub ayanamsa: Option<String>,
    #[serde(rename = "houseSystem", default = "default_house_system")]
    pub house_system: String,
    #[serde(rename = "orbSettings", default)]
    pub orb_settings: OrbSettings,
    #[serde(rename = "includeObjects", default)]
    pub include_objects: Vec<String>,
    #[serde(rename = "vedicConfig", skip_serializing_if = "Option::is_none")]
    pub vedic_config: Option<VedicConfig>,
}

fn default_zodiac_type() -> String {
    "tropical".to_string()
}
fn default_house_system() -> String {
    "placidus".to_string()
}

impl Default for ChartSettings {
    fn default() -> Self {
        Self {
            zodiac_type: "tropical".to_string(),
            ayanamsa: None,
            house_system: "placidus".to_string(),
            orb_settings: OrbSettings::default(),
            include_objects: vec![],
            vedic_config: None,
        }
    }
}

/// Vedic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VedicConfig {
    #[serde(default)]
    pub include_nakshatras: bool,
    #[serde(default = "default_true")]
    pub include_angles_in_nakshatra: bool,
    pub nakshatra_objects: Option<Vec<String>>,
    #[serde(default)]
    pub vargas: Vec<String>,
    #[serde(default)]
    pub include_dashas: bool,
    #[serde(default = "default_vimshottari")]
    pub dasha_systems: Vec<String>,
    #[serde(default = "default_dashas_depth")]
    pub dashas_depth: String,
    #[serde(default)]
    pub include_yogas: bool,
}

fn default_true() -> bool {
    true
}
fn default_vimshottari() -> Vec<String> {
    vec!["vimshottari".to_string()]
}
fn default_dashas_depth() -> String {
    "pratyantardasha".to_string()
}

/// Layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    pub kind: String, // "natal", "transit", "progressed"
    #[serde(rename = "subjectId", skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
    #[serde(rename = "explicitDateTime", skip_serializing_if = "Option::is_none")]
    pub explicit_date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

/// Render request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    pub subjects: Vec<Subject>,
    pub settings: ChartSettings,
    #[serde(rename = "layer_config")]
    pub layer_config: HashMap<String, LayerConfig>,
    #[serde(rename = "settings_override", default, skip_serializing_if = "HashMap::is_empty")]
    pub settings_override: HashMap<String, serde_json::Value>,
}

