/// Application settings model
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveLibrary {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub theme: String,
    pub archive_libraries: Vec<ArchiveLibrary>,
    pub last_opened_library: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            language: "zh_CN".to_string(),
            theme: "light".to_string(),
            archive_libraries: Vec::new(),
            last_opened_library: None,
        }
    }
}

impl AppSettings {
    /// Set language preference
    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }

    /// Add an archive library
    pub fn add_library(&mut self, name: String, path: String) {
        self.archive_libraries.push(ArchiveLibrary { name, path });
    }

    /// Remove an archive library by path
    pub fn remove_library(&mut self, path: &str) {
        self.archive_libraries.retain(|lib| lib.path != path);
    }

    /// Rename an archive library
    pub fn rename_library(&mut self, index: usize, new_name: String) -> Result<(), &'static str> {
        if let Some(lib) = self.archive_libraries.get_mut(index) {
            lib.name = new_name;
            Ok(())
        } else {
            Err("Library index out of bounds")
        }
    }

    /// Set the last opened library
    pub fn set_last_opened_library(&mut self, path: Option<String>) {
        self.last_opened_library = path;
    }
}