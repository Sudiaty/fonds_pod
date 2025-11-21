/// Configuration repository interface - kept as trait for file/memory implementations
use super::models::AppSettings;
use std::error::Error;

/// Configuration repository interface
pub trait ConfigRepository {
    fn load(&self) -> Result<AppSettings, Box<dyn Error>>;
    fn save(&self, settings: &AppSettings) -> Result<(), Box<dyn Error>>;
}
