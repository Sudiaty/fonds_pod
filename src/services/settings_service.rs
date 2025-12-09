/// Settings Service - Business logic for settings management
/// Handles archive libraries, language preferences, and configuration
use crate::models::app_settings::ArchiveLibrary;
use crate::persistence::config_repository::FileConfigRepository;
use std::error::Error;
use std::path::PathBuf;
use std::fs;

/// Settings service for managing application preferences
pub struct SettingsService {
    config_repo: FileConfigRepository,
}

impl SettingsService {
    /// Create a new settings service
    pub fn new() -> Self {
        Self {
            config_repo: FileConfigRepository::new(),
        }
    }

    /// Get current language setting
    pub fn get_language(&self) -> Result<String, Box<dyn Error>> {
        let settings = self.config_repo.load()?;
        Ok(settings.language)
    }

    /// Set language preference
    pub fn set_language(&self, language: String) -> Result<(), Box<dyn Error>> {
        let mut settings = self.config_repo.load()?;
        settings.set_language(language);
        self.config_repo.save(&settings)?;
        Ok(())
    }

    /// Get all archive libraries
    pub fn list_archive_libraries(&self) -> Result<Vec<ArchiveLibrary>, Box<dyn Error>> {
        let settings = self.config_repo.load()?;
        Ok(settings.archive_libraries)
    }

    /// Add a new archive library
    pub fn add_archive_library(
        &self,
        name: String,
        path: String,
    ) -> Result<(), Box<dyn Error>> {
        // Validate inputs
        if name.trim().is_empty() {
            return Err("Archive name cannot be empty".into());
        }

        if path.trim().is_empty() {
            return Err("Archive path cannot be empty".into());
        }

        // Check if path is valid
        let path_buf = PathBuf::from(&path);
        if !path_buf.parent().is_some() {
            return Err("Invalid archive path".into());
        }

        // Check if name already exists
        let libraries = self.list_archive_libraries()?;
        if libraries.iter().any(|lib| lib.name == name) {
            return Err("Archive name already exists".into());
        }

        // Ensure directory exists
        fs::create_dir_all(&path)?;
        
        // Initialize database if needed
        let db_path = std::path::PathBuf::from(&path).join(".fondspod.db");
        if !db_path.exists() {
            crate::persistence::establish_connection(&db_path)?;
        }
        
        // Save to configuration
        let mut settings = self.config_repo.load()?;
        settings.add_library(name, path);
        self.config_repo.save(&settings)?;
        
        Ok(())
    }

    /// Remove an archive library
    pub fn remove_archive_library(&self, index: usize) -> Result<(), Box<dyn Error>> {
        let libraries = self.list_archive_libraries()?;
        if index < libraries.len() {
            let lib = &libraries[index];
            let mut settings = self.config_repo.load()?;
            settings.remove_library(&lib.path);
            self.config_repo.save(&settings)?;
            Ok(())
        } else {
            Err("Archive index out of bounds".into())
        }
    }

    /// Rename an archive library
    pub fn rename_archive_library(&self, index: usize, new_name: String) -> Result<(), Box<dyn Error>> {
        if new_name.trim().is_empty() {
            return Err("Archive name cannot be empty".into());
        }

        let libraries = self.list_archive_libraries()?;
        if index < libraries.len() {
            // Check if new name already exists
            if libraries.iter().enumerate().any(|(i, lib)| i != index && lib.name == new_name) {
                return Err("Archive name already exists".into());
            }

            let mut settings = self.config_repo.load()?;
            settings.rename_library(index, new_name)?;
            self.config_repo.save(&settings)?;
            Ok(())
        } else {
            Err("Archive index out of bounds".into())
        }
    }

    /// Apply all settings changes
    pub fn apply_settings(
        &self,
        language: String,
        libraries: Vec<ArchiveLibrary>,
    ) -> Result<(), Box<dyn Error>> {
        // Update language
        self.set_language(language)?;

        // Update archive libraries
        let current_libraries = self.list_archive_libraries()?;

        // Remove libraries that are no longer in the new list
        // Collect paths to remove first to avoid index issues during iteration
        let paths_to_remove: Vec<String> = current_libraries
            .iter()
            .filter(|lib| !libraries.iter().any(|l| l.path == lib.path))
            .map(|lib| lib.path.clone())
            .collect();

        for path in paths_to_remove {
            let mut settings = self.config_repo.load()?;
            settings.remove_library(&path);
            let _ = self.config_repo.save(&settings);
        }

        // Add new libraries that don't exist in current list
        for new_lib in libraries {
            if !current_libraries.iter().any(|l| l.path == new_lib.path) {
                // Use the existing add_archive_library method for validation and addition
                let _ = self.add_archive_library(new_lib.name, new_lib.path);
            }
        }

        Ok(())
    }
    pub fn get_last_opened_library(&self) -> Result<Option<String>, Box<dyn Error>> {
        let settings = self.config_repo.load()?;
        Ok(settings.last_opened_library)
    }

    /// Set the last opened library
    pub fn set_last_opened_library(&self, path: Option<String>) -> Result<(), Box<dyn Error>> {
        let mut settings = self.config_repo.load()?;
        settings.set_last_opened_library(path);
        self.config_repo.save(&settings)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_archive_name() {
        let service = SettingsService::new();
        let result = service.add_archive_library("".to_string(), "/tmp".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_archive_path() {
        let service = SettingsService::new();
        let result = service.add_archive_library("Test".to_string(), "".to_string());
        assert!(result.is_err());
    }
}
