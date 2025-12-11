/// Configuration repository for JSON-based application settings
use crate::models::app_settings::AppSettings;

use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub struct FileConfigRepository {
    config_path: PathBuf,
}

impl FileConfigRepository {
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        Self { config_path }
    }

    /// Get the configuration file path
    fn get_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_local_dir() {
            config_dir.join("FondsPod").join("settings.json")
        } else {
            PathBuf::from("settings.json")
        }
    }

    /// Load settings from configuration file
    pub fn load(&self) -> Result<AppSettings, Box<dyn Error>> {
        if !self.config_path.exists() {
            // Create config directory if it doesn't exist
            if let Some(parent) = self.config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Return default settings if config file doesn't exist
            let default_settings = AppSettings::default();
            self.save(&default_settings)?;
            return Ok(default_settings);
        }

        let content = fs::read_to_string(&self.config_path)?;
        let settings: AppSettings = serde_json::from_str(&content)
            .unwrap_or_else(|_| AppSettings::default());
        
        Ok(settings)
    }

    /// Save settings to configuration file
    pub fn save(&self, settings: &AppSettings) -> Result<(), Box<dyn Error>> {
        // Create config directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(settings)?;
        fs::write(&self.config_path, content)?;
        
        Ok(())
    }
}