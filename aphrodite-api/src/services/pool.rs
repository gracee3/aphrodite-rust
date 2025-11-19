use crate::error::ApiError;
use crate::services::ChartService;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;

/// Pool of ChartService instances for concurrent request handling
pub struct ChartServicePool {
    services: Vec<Arc<Mutex<ChartService>>>,
    counter: AtomicUsize,
}

impl ChartServicePool {
    /// Create a new service pool with the specified number of instances
    pub fn new(pool_size: usize, ephemeris_path: Option<PathBuf>, cache_size: usize, default_wheel_json_path: Option<String>) -> Result<Self, ApiError> {
        let mut services = Vec::with_capacity(pool_size);
        
        for _ in 0..pool_size {
            let service = ChartService::new(ephemeris_path.clone(), cache_size, default_wheel_json_path.clone())
                .map_err(|e| ApiError::InternalError(format!("Failed to create service in pool: {}", e)))?;
            services.push(Arc::new(Mutex::new(service)));
        }

        Ok(Self {
            services,
            counter: AtomicUsize::new(0),
        })
    }

    /// Get a service from the pool using round-robin selection
    pub fn get_service(&self) -> Arc<Mutex<ChartService>> {
        let index = self.counter.fetch_add(1, Ordering::Relaxed) % self.services.len();
        self.services[index].clone()
    }
}

