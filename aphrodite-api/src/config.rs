use std::env;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub swiss_ephemeris_path: Option<String>,
    pub log_level: String,
    pub service_pool_size: usize,
    pub cache_size: usize,
    pub default_wheel_json_path: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .unwrap_or(8000),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            swiss_ephemeris_path: env::var("SWISS_EPHEMERIS_PATH").ok(),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            service_pool_size: env::var("SERVICE_POOL_SIZE")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
            cache_size: env::var("CACHE_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            default_wheel_json_path: env::var("DEFAULT_WHEEL_JSON_PATH")
                .ok()
                .or_else(|| {
                    // Default to wheels/default.json relative to the executable or current directory
                    Some("wheels/default.json".to_string())
                }),
        }
    }
}

