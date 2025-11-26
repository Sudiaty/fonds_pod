/// Configuration repository implementation
use crate::domain::{AppSettings, ArchiveLibrary as DomainArchiveLibrary, ConfigRepository};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArchiveLibrary {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedSettings {
    pub language: String,
    pub archive_libraries: Vec<ArchiveLibrary>,
    pub last_opened_library: Option<String>,
}

/// File-based configuration repository
#[derive(Clone)]
pub struct FileConfigRepository {
    config_path: PathBuf,
}

impl FileConfigRepository {
    pub fn new() -> Self {
        let config_dir = dirs::config_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("FondsPod");
        
        fs::create_dir_all(&config_dir).ok();
        
        let config_path = config_dir.join("settings.json");
        
        Self { config_path }
    }
    
    fn to_serialized(&self, settings: &AppSettings) -> SerializedSettings {
        SerializedSettings {
            language: settings.language.clone(),
            archive_libraries: settings.archive_libraries.iter().map(|lib| {
                ArchiveLibrary {
                    name: lib.name.clone(),
                    path: lib.path.clone(),
                }
            }).collect(),
            last_opened_library: settings.last_opened_library.clone(),
        }
    }
    
    fn from_serialized(&self, serialized: SerializedSettings) -> AppSettings {
        AppSettings {
            language: serialized.language,
            archive_libraries: serialized.archive_libraries.into_iter().map(|lib| {
                DomainArchiveLibrary {
                    name: lib.name,
                    path: lib.path,
                }
            }).collect(),
            last_opened_library: serialized.last_opened_library,
        }
    }
}

impl ConfigRepository for FileConfigRepository {
    fn load(&self) -> Result<AppSettings, Box<dyn Error>> {
        if self.config_path.exists() {
            let mut file = File::open(&self.config_path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let serialized: SerializedSettings = serde_json::from_str(&content)?;
            Ok(self.from_serialized(serialized))
        } else {
            Ok(AppSettings::default())
        }
    }
    
    fn save(&self, settings: &AppSettings) -> Result<(), Box<dyn Error>> {
        let serialized = self.to_serialized(settings);
        let content = serde_json::to_string_pretty(&serialized)?;
        let mut file = File::create(&self.config_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
