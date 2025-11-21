/// Archive service - manages archive libraries and configuration
use crate::domain::{AppSettings, ArchiveLibrary, ConfigRepository};
use crate::infrastructure::persistence::establish_connection;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs;

pub struct ArchiveService<CR: ConfigRepository> {
    config_repo: CR,
}

impl<CR: ConfigRepository> ArchiveService<CR> {
    pub fn new(config_repo: CR) -> Self {
        Self { config_repo }
    }
    
    /// Get all archive libraries
    pub fn list_libraries(&self) -> Result<Vec<ArchiveLibrary>, Box<dyn Error>> {
        let settings = self.config_repo.load()?;
        Ok(settings.archive_libraries)
    }
    
    /// Get current settings
    pub fn get_settings(&self) -> Result<AppSettings, Box<dyn Error>> {
        self.config_repo.load()
    }
    
    /// Add a new archive library
    pub fn add_library(&self, name: String, path: String) -> Result<(), Box<dyn Error>> {
        // Ensure directory exists
        fs::create_dir_all(&path)?;
        
        // Initialize database
        let db_path = PathBuf::from(&path).join(".fondspod.db");
        establish_connection(&db_path)?;
        
        // Save to configuration
        let mut settings = self.config_repo.load()?;
        settings.archive_libraries.push(ArchiveLibrary { name, path });
        self.config_repo.save(&settings)?;
        
        Ok(())
    }
    
    /// Remove an archive library
    pub fn remove_library(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut settings = self.config_repo.load()?;
        settings.archive_libraries.retain(|lib| lib.path != path);
        self.config_repo.save(&settings)?;
        Ok(())
    }
    
    /// Rename an archive library
    pub fn rename_library(&self, index: usize, new_name: String) -> Result<(), Box<dyn Error>> {
        let mut settings = self.config_repo.load()?;
        if let Some(lib) = settings.archive_libraries.get_mut(index) {
            lib.name = new_name;
            self.config_repo.save(&settings)?;
            Ok(())
        } else {
            Err("Library index out of bounds".into())
        }
    }
    
    /// Get database path for an archive
    pub fn get_database_path(&self, archive_path: &str) -> PathBuf {
        Path::new(archive_path).join(".fondspod.db")
    }
    
    /// Get database path by index
    pub fn get_database_path_by_index(&self, index: i32) -> Result<Option<PathBuf>, Box<dyn Error>> {
        if index < 0 {
            return Ok(None);
        }
        
        let settings = self.config_repo.load()?;
        if let Some(lib) = settings.archive_libraries.get(index as usize) {
            Ok(Some(self.get_database_path(&lib.path)))
        } else {
            Ok(None)
        }
    }
    
    /// Set language preference
    pub fn set_language(&self, language: String) -> Result<(), Box<dyn Error>> {
        let mut settings = self.config_repo.load()?;
        settings.language = language;
        self.config_repo.save(&settings)?;
        Ok(())
    }
    
    /// Get the index of last opened library
    pub fn get_last_opened_index(&self) -> Result<Option<usize>, Box<dyn Error>> {
        let settings = self.config_repo.load()?;
        if let Some(last_opened) = &settings.last_opened_library {
            Ok(settings.archive_libraries.iter().position(|lib| &lib.path == last_opened))
        } else {
            Ok(None)
        }
    }
}
