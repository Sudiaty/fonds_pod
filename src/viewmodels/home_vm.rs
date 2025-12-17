/// Home View Model - MVVM architecture
/// Manages the state and business logic for the home page (fonds management)
use crate::services::SettingsService;
use crate::{AppWindow, CrudListItem, DialogField, DialogFieldType};
use crate::persistence::{
    FondsRepository, SeriesRepository, FilesRepository, ItemsRepository,
    FondSchemasRepository, FondClassificationsRepository, SchemaRepository, 
    schema_item_repository::SchemaItemRepository, SequencesRepository,
    establish_connection,
};
use crate::models::fond::Fond;
use crate::models::series::Series;
use crate::models::file::File;
use crate::models::item::Item;
use crate::models::fond_schema::FondSchema;
use crate::core::GenericRepository;
use slint::{ComponentHandle, ModelRc, VecModel, SharedString, Model};
use crate::slint_generatedAppWindow;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::path::PathBuf;
use diesel::SqliteConnection;
use open;

/// Home ViewModel - handles state and business logic for fonds management
pub struct HomeViewModel {
    pub library_names: Vec<String>,
    pub selected_archive_index: i32,
    pub selected_file_index: i32,
    pub selected_item_index: i32,
    pub last_opened_library: String,
    
    // Fonds data
    pub fonds_list: Vec<Fond>,
    pub selected_fonds_index: i32,
    
    // Series data
    pub series_list: Vec<Series>,
    pub selected_series_index: i32,
    pub selected_series_no: String,
    
    // Files data
    pub files_list: Vec<File>,
    pub selected_file: i32,
    
    // Items data
    pub items_list: Vec<Item>,
    pub selected_item: i32,
    
    // Dialog states
    pub show_add_file_dialog: bool,
    pub new_file_name: String,
    pub new_file_path: String,
    pub show_add_item_dialog: bool,
    pub new_item_name: String,
    pub new_item_path: String,
    pub show_rename_file_dialog: bool,
    pub rename_file_index: i32,
    pub show_rename_item_dialog: bool,
    pub rename_item_index: i32,
    
    settings_service: Rc<SettingsService>,
    db_connection: Option<Rc<RefCell<SqliteConnection>>>,
    current_db_path: Option<PathBuf>,
    sequences_repo: Option<SequencesRepository>,
}

impl Default for HomeViewModel {
    fn default() -> Self {
        Self {
            library_names: Vec::new(),
            selected_archive_index: -1,
            selected_file_index: -1,
            selected_item_index: -1,
            last_opened_library: String::new(),
            fonds_list: Vec::new(),
            selected_fonds_index: 0,
            series_list: Vec::new(),
            selected_series_index: -1,
            selected_series_no: String::new(),
            files_list: Vec::new(),
            selected_file: 0,
            items_list: Vec::new(),
            selected_item: 0,
            show_add_file_dialog: false,
            new_file_name: String::new(),
            new_file_path: String::new(),
            show_add_item_dialog: false,
            new_item_name: String::new(),
            new_item_path: String::new(),
            show_rename_file_dialog: false,
            rename_file_index: -1,
            show_rename_item_dialog: false,
            rename_item_index: -1,
            settings_service: Rc::new(SettingsService::new()),
            db_connection: None,
            current_db_path: None,
            sequences_repo: None,
        }
    }
}

impl HomeViewModel {
    /// Create a new HomeViewModel with the given settings service
    pub fn new(settings_service: Rc<SettingsService>) -> Self {
        Self {
            library_names: Vec::new(),
            selected_archive_index: -1,
            selected_file_index: -1,
            selected_item_index: -1,
            last_opened_library: String::new(),
            fonds_list: Vec::new(),
            selected_fonds_index: 0,
            series_list: Vec::new(),
            selected_series_index: -1,
            selected_series_no: String::new(),
            files_list: Vec::new(),
            selected_file: 0,
            items_list: Vec::new(),
            selected_item: 0,
            show_add_file_dialog: false,
            new_file_name: String::new(),
            new_file_path: String::new(),
            show_add_item_dialog: false,
            new_item_name: String::new(),
            new_item_path: String::new(),
            show_rename_file_dialog: false,
            rename_file_index: -1,
            show_rename_item_dialog: false,
            rename_item_index: -1,
            settings_service,
            db_connection: None,
            current_db_path: None,
            sequences_repo: None,
        }
    }

    /// Browse folder and return selected path
    pub fn browse_folder(&self) -> Option<String> {
        if let Some(folder_path) = rfd::FileDialog::new()
            .set_directory("/")
            .pick_folder() {
            Some(folder_path.to_string_lossy().to_string())
        } else {
            None
        }
    }

    /// Browse file or folder and return selected path
    pub fn browse_file_or_folder(&self) -> Option<String> {
        // Get the current file's path as default directory
        let default_dir = if self.selected_file >= 0 && (self.selected_file as usize) < self.files_list.len() {
            self.files_list[self.selected_file as usize].path.clone()
                .unwrap_or_else(|| self.last_opened_library.clone())
        } else {
            self.last_opened_library.clone()
        };

        // Use pick_file() which on many systems allows selecting folders too
        if let Some(path) = rfd::FileDialog::new()
            .set_directory(&default_dir)
            .pick_file() {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    }

    /// Update database connection for the selected archive
    fn update_connection(&mut self, library_path: &str) -> Result<(), Box<dyn Error>> {
        let db_path = PathBuf::from(library_path).join(".fondspod.db");
        let conn = establish_connection(&db_path)?;
        self.db_connection = Some(Rc::clone(&conn));
        self.current_db_path = Some(db_path);
        self.sequences_repo = Some(SequencesRepository::new(conn));
        Ok(())
    }

    /// Get a repository for fonds
    fn get_fonds_repo(&self) -> Option<FondsRepository> {
        self.db_connection.as_ref().map(|conn| FondsRepository::new(Rc::clone(conn)))
    }

    /// Get a repository for series
    fn get_series_repo(&self) -> Option<SeriesRepository> {
        self.db_connection.as_ref().map(|conn| SeriesRepository::new(Rc::clone(conn)))
    }

    /// Get a repository for files
    fn get_files_repo(&self) -> Option<FilesRepository> {
        self.db_connection.as_ref().map(|conn| FilesRepository::new(Rc::clone(conn)))
    }

    /// Get a repository for items
    fn get_items_repo(&self) -> Option<ItemsRepository> {
        self.db_connection.as_ref().map(|conn| ItemsRepository::new(Rc::clone(conn)))
    }

    /// Get a repository for fond schemas
    fn get_fond_schemas_repo(&self) -> Option<FondSchemasRepository> {
        self.db_connection.as_ref().map(|conn| FondSchemasRepository::new(Rc::clone(conn)))
    }

    /// Get a repository for classifications
    fn get_classifications_repo(&self) -> Option<FondClassificationsRepository> {
        self.db_connection.as_ref().map(|conn| FondClassificationsRepository::new(Rc::clone(conn)))
    }

    /// Get a repository for schemas
    fn get_schema_repo(&self) -> Option<SchemaRepository> {
        self.db_connection.as_ref().map(|conn| SchemaRepository::new(Rc::clone(conn)))
    }

    /// Get a repository for schema items
    fn get_schema_items_repo(&self) -> Option<SchemaItemRepository> {
        self.db_connection.as_ref().map(|conn| SchemaItemRepository::new(Rc::clone(conn)))
    }

    /// Get fond_no for a given series by looking up the fond
    fn get_fond_no_for_series(&self, fond_id: i32) -> Result<String, Box<dyn Error>> {
        for fond in &self.fonds_list {
            if fond.id == fond_id {
                return Ok(fond.fond_no.clone());
            }
        }
        Err(format!("Fond with id {} not found", fond_id).into())
    }

    /// Load archive libraries and set up initial state
    pub fn load_libraries(&mut self) -> Result<(), Box<dyn Error>> {
        let libraries = self.settings_service.list_archive_libraries()?;
        self.library_names = libraries.iter().map(|lib| lib.name.clone()).collect();

        // Load last opened library
        if let Ok(Some(last_lib)) = self.settings_service.get_last_opened_library() {
            self.last_opened_library = last_lib.clone();
            // Find the index of the last opened library
            if let Some(index) = libraries.iter().position(|lib| lib.path == last_lib) {
                self.selected_archive_index = index as i32;
                log::debug!("HomeViewModel: Setting selected_archive_index to {} (found matching library: {})", self.selected_archive_index, last_lib);
            } else {
                // Last opened library not found in current libraries, do not reset setting
                log::warn!("HomeViewModel: Last opened library '{}' not found in current libraries, keeping setting unchanged", last_lib);
                self.selected_archive_index = -1;
                log::debug!("HomeViewModel: Setting selected_archive_index to -1 (library not found)");
            }
        } else {
            // If no last opened library in settings, do not auto-set it
            // Only show first library as default in UI, but don't modify settings
            if !self.library_names.is_empty() {
                self.selected_archive_index = 0;
                log::debug!("HomeViewModel: Setting selected_archive_index to 0 (no last_opened_library, showing first library)");
                if let Some(first_lib) = libraries.first() {
                    self.last_opened_library = first_lib.path.clone();
                }
            } else {
                // No libraries available
                self.selected_archive_index = -1;
                log::debug!("HomeViewModel: Setting selected_archive_index to -1 (no libraries available)");
                self.last_opened_library = String::new();
            }
        }

        Ok(())
    }

    /// Set selected archive and update last opened library
    pub fn set_selected_archive(&mut self, index: i32) -> Result<(), Box<dyn Error>> {
        if index >= 0 && (index as usize) < self.library_names.len() {
            self.selected_archive_index = index;
            log::debug!("HomeViewModel: Setting selected_archive_index to {} (user selection)", index);
            // Get the library path and update settings
            let libraries = self.settings_service.list_archive_libraries()?;
            if let Some(lib) = libraries.get(index as usize) {
                self.last_opened_library = lib.path.clone();
                log::info!("HomeViewModel: Setting last_opened_library to: {}", lib.path);
                self.settings_service.set_last_opened_library(Some(lib.path.clone()))?;
                log::info!("HomeViewModel: Successfully saved last_opened_library");
                
                // Update database connection
                self.update_connection(&lib.path)?;
                
                // Load fonds for this library
                self.load_fonds()?;
            }
        } else {
            log::debug!("HomeViewModel: Invalid index {} for selected_archive, not updating", index);
        }
        Ok(())
    }

    /// Load fonds for the current archive
    pub fn load_fonds(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(mut repo) = self.get_fonds_repo() {
            self.fonds_list = repo.find_all()?;
            log::info!("HomeViewModel: Loaded {} fonds", self.fonds_list.len());
            
            // Reset fonds selection and dependent data
            if !self.fonds_list.is_empty() {
                self.selected_fonds_index = 0;
                let fond_id = self.fonds_list[0].id;
                self.load_series(fond_id)?;
            } else {
                self.selected_fonds_index = 0;
                self.series_list.clear();
                self.selected_series_index = -1;
                self.selected_series_no.clear();
                self.files_list.clear();
                self.selected_file = 0;
                self.items_list.clear();
                self.selected_item = 0;
            }
        }
        Ok(())
    }

    /// Load series for a specific fond
    pub fn load_series(&mut self, fond_id: i32) -> Result<(), Box<dyn Error>> {
        if let Some(mut repo) = self.get_series_repo() {
            // Find series for this fond by fond_id
            let all_series = repo.find_all()?;
            self.series_list = all_series.into_iter()
                .filter(|s| s.fond_id == fond_id)
                .collect();
            log::info!("HomeViewModel: Loaded {} series for fond_id {}", self.series_list.len(), fond_id);
            
            // If no series found, try to generate them
            if self.series_list.is_empty() {
                log::info!("No series found for fond_id {}, attempting to generate series", fond_id);
                self.generate_series(fond_id)?;
                // Reload series after generation
                let all_series = repo.find_all()?;
                self.series_list = all_series.into_iter()
                    .filter(|s| s.fond_id == fond_id)
                    .collect();
                log::info!("After generation: Loaded {} series for fond_id {}", self.series_list.len(), fond_id);
            }
            
            // Reset selection and load files for first series
            if !self.series_list.is_empty() {
                self.selected_series_index = 0;
                let series_id = self.series_list[0].id;
                // Generate series_no for display purposes (no longer stored in DB)
                self.selected_series_no = format!("S{:05}", series_id);
                self.load_files(series_id)?;
            } else {
                self.selected_series_index = -1;
                self.selected_series_no.clear();
                self.files_list.clear();
                self.items_list.clear();
            }
        }
        Ok(())
    }

    /// Load files for a specific series
    pub fn load_files(&mut self, series_id: i32) -> Result<(), Box<dyn Error>> {
        if let Some(mut repo) = self.get_files_repo() {
            let all_files = repo.find_all()?;
            self.files_list = all_files.into_iter()
                .filter(|f| f.series_id == series_id)
                .collect();
            log::info!("HomeViewModel: Loaded {} files for series_id {}", self.files_list.len(), series_id);
            
            // Reset selection and load items for first file
            if !self.files_list.is_empty() {
                self.selected_file = 0;
                let file_id = self.files_list[0].id;
                self.load_items(file_id)?;
            } else {
                self.selected_file = 0;
                self.items_list.clear();
            }
        }
        Ok(())
    }

    /// Load items for a specific file
    pub fn load_items(&mut self, file_id: i32) -> Result<(), Box<dyn Error>> {
        if let Some(mut repo) = self.get_items_repo() {
            let all_items = repo.find_all()?;
            self.items_list = all_items.into_iter()
                .filter(|i| i.file_id == file_id)
                .collect();
            log::info!("HomeViewModel: Loaded {} items for file_id {}", self.items_list.len(), file_id);
            self.selected_item = 0;
        }
        Ok(())
    }

    /// Generate series for a fond based on fond_schemas (cartesian product of schema items)
    pub fn generate_series(&mut self, fond_id: i32) -> Result<(), Box<dyn Error>> {
        // Get fond_no from fonds_list for logging and fond_schemas filtering
        let fond_no = if let Some(fond) = self.fonds_list.iter().find(|f| f.id == fond_id) {
            fond.fond_no.clone()
        } else {
            return Err(format!("Fond with id {} not found", fond_id).into());
        };

        // Get fond_schemas for this fond
        let fond_schemas = if let Some(mut repo) = self.get_fond_schemas_repo() {
            let all_schemas = repo.find_all()?;
            let mut schemas: Vec<_> = all_schemas.into_iter()
                .filter(|fs| fs.fond_id == fond_id)
                .collect();
            schemas.sort_by_key(|s| s.sort_order);
            log::info!("Found {} fond_schemas for fond {}", schemas.len(), fond_no);
            schemas
        } else {
            return Err("No database connection".into());
        };

        if fond_schemas.is_empty() {
            log::warn!("No fond_schemas found for fond {} - cannot generate series", fond_no);
            return Ok(());
        }

        // Get the fond to extract created_at year for Year schema special handling
        let created_year = if let Some(mut fond_repo) = self.get_fonds_repo() {
            let all_fonds = fond_repo.find_all()?;
            if let Some(fond) = all_fonds.iter().find(|f| f.fond_no == fond_no) {
                fond.created_at.format("%Y").to_string().parse::<i32>().unwrap_or(2025)
            } else {
                chrono::Utc::now().format("%Y").to_string().parse::<i32>().unwrap_or(2025)
            }
        } else {
            chrono::Utc::now().format("%Y").to_string().parse::<i32>().unwrap_or(2025)
        };
        let current_year = chrono::Utc::now().format("%Y").to_string().parse::<i32>().unwrap_or(2025);

        // Get schema items for each fond_schema
        let mut dimension_items: Vec<Vec<crate::models::schema_item::SchemaItem>> = Vec::new();
        
        if let Some(mut schema_repo) = self.get_schema_repo() {
            if let Some(mut items_repo) = self.get_schema_items_repo() {
                let all_schemas = schema_repo.find_all()?;
                let all_items = items_repo.find_all()?;
                log::info!("Available schemas: {}", all_schemas.iter().map(|s| s.schema_no.as_str()).collect::<Vec<_>>().join(", "));
                
                for fond_schema in &fond_schemas {
                    log::debug!("Processing fond_schema: schema_item_id={:?}", fond_schema.schema_item_id);
                    
                    if let Some(schema) = all_schemas.iter().find(|s| s.id == fond_schema.schema_id) {
                        if schema.schema_no == "Year" {
                            // Special handling for Year schema: generate years from created_year to current_year
                            // Year schema has no pre-defined items, generate them dynamically
                            let mut year_items = Vec::new();
                            for year in created_year..=current_year {
                                year_items.push(crate::models::schema_item::SchemaItem {
                                    id: 0, // dummy id for cartesian product
                                    schema_id: schema.id,
                                    item_no: year.to_string(),
                                    item_name: year.to_string(),
                                    created_by: "system".into(),
                                    created_machine: "system".into(),
                                    created_at: chrono::Utc::now().naive_utc(),
                                });
                            }
                            dimension_items.push(year_items);
                        } else if let Some(item_id) = fond_schema.schema_item_id {
                            // Normal schema with specific item
                            if let Some(item) = all_items.iter().find(|i| i.id == item_id) {
                                dimension_items.push(vec![item.clone()]);
                            } else {
                                log::warn!("Schema item not found for id {}", item_id);
                            }
                        } else {
                            // Normal schema: collect all existing items for this schema
                            let schema_items: Vec<_> = all_items.iter()
                                .filter(|i| i.schema_id == schema.id)
                                .cloned()
                                .collect();
                            dimension_items.push(schema_items);
                        }
                    }
                }
            }
        }

        if dimension_items.is_empty() {
            log::warn!("No schema items found for fond {} - cannot generate series", fond_no);
            return Ok(());
        }

        // Generate cartesian product
        let mut series_combinations: Vec<Vec<&crate::models::schema_item::SchemaItem>> = vec![vec![]];
        for dimension in &dimension_items {
            let mut new_combinations = Vec::new();
            for combo in &series_combinations {
                for item in dimension {
                    let mut new_combo = combo.clone();
                    new_combo.push(item);
                    new_combinations.push(new_combo);
                }
            }
            series_combinations = new_combinations;
        }

        log::info!("Generated {} series combinations for fond {}", series_combinations.len(), fond_no);

        // Delete existing series for this fond and recreate them
        if let Some(mut series_repo) = self.get_series_repo() {
            // First delete existing series
            let existing_series = series_repo.find_all()?;
            for series in existing_series.iter().filter(|s| s.fond_id == fond_id) {
                series_repo.delete(series.id)?;
            }
            log::debug!("Deleted existing series for fond {}", fond_no);

            // Create new series from combinations
            for (index, combo) in series_combinations.iter().enumerate() {
                let name = combo.iter()
                    .map(|item| item.item_name.as_str())
                    .collect::<Vec<_>>()
                    .join("-");
                
                let series_no_part = combo.iter()
                    .map(|item| item.item_no.as_str())
                    .collect::<Vec<_>>()
                    .join("-");
                
                let series_no = format!("{}-{}", fond_no, series_no_part);
                
                let series = Series {
                    id: 0,
                    fond_id,
                    series_no,
                    name,
                    created_by: String::new(),
                    created_machine: String::new(),
                    created_at: chrono::Utc::now().naive_utc(),
                };
                series_repo.create(series)?;
            }
            
            log::info!("Generated and created {} series for fond {}", series_combinations.len(), fond_no);
        }

        // Reload series
        self.load_series(fond_id)?;
        Ok(())
    }

    /// Add a new fond with the given data
    pub fn add_fond(&mut self, name: &str, classification_code: &str, selected_schema_nos: Vec<String>) -> Result<(), Box<dyn Error>> {
        // Generate fond_no using sequence
        let fond_no = self.generate_next_fond_no(classification_code)?;
        
        // Create the fond
        let fond_id = if let Some(mut repo) = self.get_fonds_repo() {
            let fond = Fond {
                id: 0,
                fond_no: fond_no.clone(),
                fond_classification_code: classification_code.to_string(),
                name: name.to_string(),
                created_by: String::new(),
                created_machine: String::new(),
                created_at: chrono::Utc::now().naive_utc(),
            };
            let id = repo.create(fond)?;
            log::info!("Created fond: {} - {}", fond_no, name);
            id
        } else {
            return Err("No database connection".into());
        };

        // Create fond_schemas for selected schemas
        if let Some(mut fs_repo) = self.get_fond_schemas_repo() {
            if let Some(mut schema_repo) = self.get_schema_repo() {
                if let Some(mut items_repo) = self.get_schema_items_repo() {
                    let all_schemas = schema_repo.find_all()?;
                    let all_items = items_repo.find_all()?;
                    for (order, schema_no) in selected_schema_nos.iter().enumerate() {
                        if let Some(schema) = all_schemas.iter().find(|s| s.schema_no == *schema_no) {
                            // For all schemas, set schema_item_id to None to allow cartesian product of all items
                            let fond_schema = FondSchema {
                                id: 0,
                                fond_id,
                                schema_id: schema.id,
                                schema_item_id: None,
                                sort_order: order as i32,
                                created_by: String::new(),
                                created_machine: String::new(),
                                created_at: chrono::Utc::now().naive_utc(),
                            };
                            fs_repo.create(fond_schema)?;
                        }
                    }
                    log::info!("Created {} fond_schemas for fond {}", selected_schema_nos.len(), fond_no);
                }
            }
        }

        // Reload fonds immediately so generate_series can find the newly created fond
        self.load_fonds()?;

        // Generate series based on the schemas
        self.generate_series(fond_id)?;
        
        // Create fond directory
        let fond_dir = std::path::Path::new(&self.last_opened_library).join(&fond_no);
        if let Err(e) = std::fs::create_dir_all(&fond_dir) {
            log::error!("Failed to create fond directory {:?}: {}", fond_dir, e);
        } else {
            log::info!("Created fond directory: {:?}", fond_dir);
        }
        
        // Select the newly created fond
        self.selected_fonds_index = self.fonds_list.len() as i32 - 1;

        Ok(())
    }

    /// Add a new file to the selected series
    pub fn add_file(&mut self, name: &str, file_no: &str, path: Option<String>) -> Result<(), Box<dyn Error>> {
        if self.selected_series_index < 0 || self.selected_series_index >= self.series_list.len() as i32 {
            return Err("No series selected".into());
        }
        
        let series_id = self.series_list[self.selected_series_index as usize].id;
        
        // Auto-generate path if not provided
        let file_path = match path {
            Some(p) => Some(p),
            None => {
                // Generate path as [fond_directory]/[file_no]
                if self.selected_fonds_index >= 0 && (self.selected_fonds_index as usize) < self.fonds_list.len() {
                    let fond = &self.fonds_list[self.selected_fonds_index as usize];
                    let fond_dir = std::path::Path::new(&self.last_opened_library).join(&fond.fond_no);
                    let file_dir = fond_dir.join(file_no);
                    
                    // Create the directory
                    if let Err(e) = std::fs::create_dir_all(&file_dir) {
                        log::error!("Failed to create file directory {:?}: {}", file_dir, e);
                        return Err(format!("Failed to create file directory: {}", e).into());
                    } else {
                        log::info!("Created file directory: {:?}", file_dir);
                    }
                    
                    Some(file_dir.to_string_lossy().to_string())
                } else {
                    return Err("No fond selected".into());
                }
            }
        };
        
        if let Some(mut repo) = self.get_files_repo() {
            let file = File {
                id: 0,
                series_id,
                name: name.to_string(),
                file_no: file_no.to_string(),
                path: file_path.clone(),
                created_by: String::new(),
                created_machine: String::new(),
                created_at: chrono::Utc::now().naive_utc(),
            };
            repo.create(file)?;
            log::info!("Created file: {} - {} (path: {:?})", file_no, name, file_path);
        }

        self.load_files(series_id)?;
        Ok(())
    }

    /// Generate next file number for current series
    fn generate_next_file_no(&mut self) -> Result<String, Box<dyn Error>> {
        if self.selected_series_index < 0 || self.selected_series_index >= self.series_list.len() as i32 {
            return Err("No series selected".into());
        }

        let series = &self.series_list[self.selected_series_index as usize];

        if let Some(mut repo) = self.sequences_repo.as_mut() {
            let seq_num = repo.get_next_number(&series.series_no, Some(2))?;
            Ok(format!("{}-{}", series.series_no, seq_num))
        } else {
            Err("No database connection".into())
        }
    }
    
    /// Generate next fond number
    fn generate_next_fond_no(&mut self, classification_code: &str) -> Result<String, Box<dyn Error>> {
        if let Some(mut repo) = self.sequences_repo.as_mut() {
            let seq_num = repo.get_next_number(classification_code, Some(2))?;
            Ok(format!("{}{}", classification_code, seq_num))
        } else {
            Err("No database connection".into())
        }
    }
    pub fn add_item(&mut self, name: &str, path: Option<String>) -> Result<(), Box<dyn Error>> {
        if self.files_list.is_empty() || self.selected_file < 0 {
            return Err("No file selected".into());
        }
        
        let file_id = self.files_list[self.selected_file as usize].id;
        let file_no = self.files_list[self.selected_file as usize].file_no.clone();
        // Generate item_no
        let item_no = self.generate_next_item_no(&file_no)?;
        
        if let Some(mut repo) = self.get_items_repo() {
            let item = Item {
                id: 0,
                file_id,
                item_no: item_no.clone(),
                name: name.to_string(),
                path,
                created_by: String::new(),
                created_machine: String::new(),
                created_at: chrono::Utc::now().naive_utc(),
            };
            repo.create(item)?;
            log::info!("Created item: {} - {}", item_no, name);
        }

        self.load_items(file_id)?;
        Ok(())
    }

    /// Generate next item number
    fn generate_next_item_no(&mut self, file_no: &str) -> Result<String, Box<dyn Error>> {
        if let Some(mut repo) = self.sequences_repo.as_mut() {
            let seq_num = repo.get_next_number(file_no, Some(3))?;
            Ok(format!("{}-{}", file_no, seq_num))
        } else {
            Err("No database connection".into())
        }
    }
    
    /// Delete the selected file
    pub fn delete_file(&mut self) -> Result<(), Box<dyn Error>> {
        if self.files_list.is_empty() || self.selected_file < 0 {
            return Err("No file selected".into());
        }
        
        let file_id = self.files_list[self.selected_file as usize].id;
        let series_id = self.files_list[self.selected_file as usize].series_id;
        
        if let Some(mut repo) = self.get_files_repo() {
            repo.delete(file_id)?;
            log::info!("Deleted file with id {}", file_id);
        }

        self.load_files(series_id)?;
        Ok(())
    }

    /// Rename the selected file
    pub fn rename_file(&mut self, index: i32, new_name: &str) -> Result<(), Box<dyn Error>> {
        if self.files_list.is_empty() || index < 0 || (index as usize) >= self.files_list.len() {
            return Err("Invalid file index".into());
        }
        
        let file_id = self.files_list[index as usize].id;
        let series_id = self.files_list[index as usize].series_id;
        
        if let Some(mut repo) = self.get_files_repo() {
            if let Some(mut file) = repo.find_by_id(file_id)? {
                file.name = new_name.to_string();
                repo.update(&file)?;
                log::info!("Renamed file with id {} to '{}'", file_id, new_name);
            }
        }

        self.load_files(series_id)?;
        Ok(())
    }

    /// Delete the selected item
    pub fn delete_item(&mut self) -> Result<(), Box<dyn Error>> {
        if self.items_list.is_empty() || self.selected_item < 0 {
            return Err("No item selected".into());
        }
        
        let item_id = self.items_list[self.selected_item as usize].id;
        let file_id = self.items_list[self.selected_item as usize].file_id;
        
        if let Some(mut repo) = self.get_items_repo() {
            repo.delete(item_id)?;
            log::info!("Deleted item with id {}", item_id);
        }

        self.load_items(file_id)?;
        Ok(())
    }

    /// Rename the selected item
    pub fn rename_item(&mut self, index: i32, new_name: &str) -> Result<(), Box<dyn Error>> {
        if self.items_list.is_empty() || index < 0 || (index as usize) >= self.items_list.len() {
            return Err("Invalid item index".into());
        }
        
        let item_id = self.items_list[index as usize].id;
        let file_id = self.items_list[index as usize].file_id;
        
        if let Some(mut repo) = self.get_items_repo() {
            if let Some(mut item) = repo.find_by_id(item_id)? {
                item.name = new_name.to_string();
                repo.update(&item)?;
                log::info!("Renamed item with id {} to '{}'", item_id, new_name);
            }
        }

        self.load_items(file_id)?;
        Ok(())
    }

    /// Get classification options for the add fonds dialog
    pub fn get_classification_options(&self) -> Vec<SharedString> {
        if let Some(mut repo) = self.get_classifications_repo() {
            if let Ok(classifications) = repo.find_by_parent_id(None) {
                return classifications.into_iter()
                    .filter(|c| c.active)
                    .map(|c| SharedString::from(format!("{} - {}", c.code, c.name)))
                    .collect();
            }
        }
        Vec::new()
    }

    /// Get primary classifications for the add fonds dialog
    pub fn get_primary_classifications(&self) -> Vec<SharedString> {
        if let Some(mut repo) = self.get_classifications_repo() {
            if let Ok(classifications) = repo.find_by_parent_id(None) {
                return classifications.into_iter()
                    .filter(|c| c.active)
                    .map(|c| SharedString::from(format!("{} - {}", c.code, c.name)))
                    .collect();
            }
        }
        Vec::new()
    }

    /// Get primary codes for the add fonds dialog
    pub fn get_primary_codes(&self) -> Vec<SharedString> {
        if let Some(mut repo) = self.get_classifications_repo() {
            if let Ok(classifications) = repo.find_by_parent_id(None) {
                return classifications.into_iter()
                    .filter(|c| c.active)
                    .map(|c| SharedString::from(c.code))
                    .collect();
            }
        }
        Vec::new()
    }

    /// Get secondary classifications for the add fonds dialog
    pub fn get_secondary_classifications(&self) -> Vec<Vec<SharedString>> {
        if let Some(mut repo) = self.get_classifications_repo() {
            if let Ok(primary_classifications) = repo.find_by_parent_id(None) {
                let mut secondary_lists = Vec::new();
                for primary in primary_classifications.into_iter().filter(|c| c.active) {
                    if let Ok(secondary_classifications) = repo.find_by_parent_id(Some(primary.id)) {
                        let secondary_list = secondary_classifications.into_iter()
                            .filter(|c| c.active)
                            .map(|c| SharedString::from(format!("{} - {}", c.code, c.name)))
                            .collect();
                        secondary_lists.push(secondary_list);
                    } else {
                        secondary_lists.push(Vec::new());
                    }
                }
                return secondary_lists;
            }
        }
        Vec::new()
    }

    /// Get secondary codes for the add fonds dialog
    pub fn get_secondary_codes(&self) -> Vec<Vec<SharedString>> {
        if let Some(mut repo) = self.get_classifications_repo() {
            if let Ok(primary_classifications) = repo.find_by_parent_id(None) {
                let mut secondary_lists = Vec::new();
                for primary in primary_classifications.into_iter().filter(|c| c.active) {
                    if let Ok(secondary_classifications) = repo.find_by_parent_id(Some(primary.id)) {
                        let secondary_list = secondary_classifications.into_iter()
                            .filter(|c| c.active)
                            .map(|c| SharedString::from(c.code))
                            .collect();
                        secondary_lists.push(secondary_list);
                    } else {
                        secondary_lists.push(Vec::new());
                    }
                }
                return secondary_lists;
            }
        }
        Vec::new()
    }

    /// Get schema options for the add fonds dialog
    pub fn get_schema_options(&self) -> Vec<slint_generatedAppWindow::FondsSchemaOption> {
        if let Some(mut repo) = self.get_schema_repo() {
            if let Ok(schemas) = repo.find_all() {
                return schemas.into_iter()
                    .map(|s| slint_generatedAppWindow::FondsSchemaOption {
                        id: s.id,
                        schema_no: s.schema_no.into(),
                        name: s.name.into(),
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    /// Initialize UI with current data
    pub fn init_ui(&self, ui_handle: &AppWindow) {
        // Set library names
        let names: Vec<SharedString> = self.library_names.iter().map(|s| s.as_str().into()).collect();
        let names_model = ModelRc::new(VecModel::from(names));
        ui_handle.set_library_names(names_model);

        // Set selected archive index
        log::debug!("HomeViewModel: Setting UI selected_archive to {}", self.selected_archive_index);
        ui_handle.set_selected_archive(self.selected_archive_index);

        // Set last opened library
        ui_handle.set_last_opened_library(self.last_opened_library.clone().into());

        // Set fonds names
        let fonds_names: Vec<SharedString> = self.fonds_list.iter()
            .map(|f| format!("{} - {}", f.fond_no, f.name).into())
            .collect();
        let fonds_model = ModelRc::new(VecModel::from(fonds_names));
        ui_handle.set_fonds_names(fonds_model);
        ui_handle.set_selected_fonds(self.selected_fonds_index);

        // Set series list items
        let series_items: Vec<CrudListItem> = self.series_list.iter()
            .map(|s| CrudListItem {
                id: s.id,
                title: s.name.clone().into(),
                subtitle: s.series_no.clone().into(),
                active: true,
            })
            .collect();
        let series_model = ModelRc::new(VecModel::from(series_items));
        ui_handle.set_series_list_items(series_model);
        ui_handle.set_selected_series_index(self.selected_series_index);
        ui_handle.set_selected_series_no(self.selected_series_no.clone().into());

        // Set files list items
        let files_items: Vec<CrudListItem> = self.files_list.iter()
            .map(|f| CrudListItem {
                id: f.id,
                title: f.name.clone().into(),
                subtitle: f.file_no.clone().into(),
                active: true,
            })
            .collect();
        let files_model = ModelRc::new(VecModel::from(files_items));
        ui_handle.set_files_list_items(files_model);
        ui_handle.set_selected_file(self.selected_file);

        // Set items list items
        let items_items: Vec<CrudListItem> = self.items_list.iter()
            .map(|i| CrudListItem {
                id: i.id,
                title: i.name.clone().into(),
                subtitle: i.item_no.clone().into(),
                active: true,
            })
            .collect();
        let items_model = ModelRc::new(VecModel::from(items_items));
        ui_handle.set_items_list_items(items_model);
        ui_handle.set_selected_item(self.selected_item);

        // Set dialog states
        ui_handle.set_show_add_file_dialog(self.show_add_file_dialog);
        
        // Initialize add file dialog fields
        let add_file_fields = vec![
            DialogField {
                label: "label_file_name".into(),
                field_type: DialogFieldType::Text,
                value: "".into(),
                placeholder: "placeholder_file_name".into(),
            },
            DialogField {
                label: "label_file_no".into(),
                field_type: DialogFieldType::Text,
                value: "".into(),
                placeholder: "placeholder_file_no".into(),
            },
            DialogField {
                label: "label_file_path".into(),
                field_type: DialogFieldType::Text,
                value: "".into(),
                placeholder: "placeholder_file_path".into(),
            },
        ];
        let fields_model = ModelRc::new(VecModel::from(add_file_fields));
        ui_handle.set_add_file_fields(fields_model);

        // Initialize add item dialog fields
        let add_item_fields = vec![
            DialogField {
                label: "label_item_name".into(),
                field_type: DialogFieldType::Text,
                value: "".into(),
                placeholder: "placeholder_item_name".into(),
            },
            DialogField {
                label: "label_item_path".into(),
                field_type: DialogFieldType::Text,
                value: "".into(),
                placeholder: "placeholder_item_path".into(),
            },
        ];
        let item_fields_model = ModelRc::new(VecModel::from(add_item_fields));
        ui_handle.set_add_item_fields(item_fields_model);

        // Set rename dialog states
        ui_handle.set_show_rename_file_dialog(self.show_rename_file_dialog);
        ui_handle.set_show_rename_item_dialog(self.show_rename_item_dialog);

        // Initialize rename file dialog fields
        let rename_file_fields = vec![
            DialogField {
                label: "label_name".into(),
                field_type: DialogFieldType::Text,
                value: "".into(),
                placeholder: "placeholder_file_name".into(),
            },
        ];
        let rename_file_fields_model = ModelRc::new(VecModel::from(rename_file_fields));
        ui_handle.set_rename_file_fields(rename_file_fields_model);

        // Initialize rename item dialog fields
        let rename_item_fields = vec![
            DialogField {
                label: "label_name".into(),
                field_type: DialogFieldType::Text,
                value: "".into(),
                placeholder: "placeholder_item_name".into(),
            },
        ];
        let rename_item_fields_model = ModelRc::new(VecModel::from(rename_item_fields));
        ui_handle.set_rename_item_fields(rename_item_fields_model);

        // Set classification and schema options for add fonds dialog - these are no longer used in UI
        // as the dialog is now independent
    }

    /// Setup UI callbacks for the home page
    pub fn setup_ui_callbacks(&self, ui_handle: &AppWindow, vm: Rc<RefCell<Self>>) {
        let ui_weak = ui_handle.as_weak();
        
        // Archive selected callback
        ui_handle.on_archive_selected({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    if let Err(e) = vm.set_selected_archive(index) {
                        log::error!("Failed to set selected archive: {}", e);
                    } else {
                        log::info!("Archive selected: index={}", index);
                        if let Some(ui) = ui_weak.upgrade() {
                            vm.init_ui(&ui);
                        }
                    }
                }
            }
        });
        
        // Fonds selected callback
        ui_handle.on_fonds_selected({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.selected_fonds_index = index;
                    if let Some(fond) = vm.fonds_list.get(index as usize) {
                        let fond_id = fond.id;
                        if let Err(e) = vm.load_series(fond_id) {
                            log::error!("Failed to load series: {}", e);
                        }
                        if let Some(ui) = ui_weak.upgrade() {
                            vm.init_ui(&ui);
                        }
                    }
                }
            }
        });

        // Series selected callback
        ui_handle.on_series_selected({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.selected_series_index = index;
                    if let Some(series) = vm.series_list.get(index as usize).cloned() {
                        // Generate series_no for display purposes
                        vm.selected_series_no = format!("S{:05}", series.id);
                        let series_id = series.id;
                        if let Err(e) = vm.load_files(series_id) {
                            log::error!("Failed to load files: {}", e);
                        } else {
                            log::info!("Successfully loaded files for series {}, refreshing UI", series_id);
                            if let Some(ui) = ui_weak.upgrade() {
                                vm.init_ui(&ui);
                            }
                        }
                    }
                }
            }
        });

        // Rebuild series callback
        ui_handle.on_rebuild_series({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move || {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    if let Some(fond) = vm.fonds_list.get(vm.selected_fonds_index as usize).cloned() {
                        if let Err(e) = vm.generate_series(fond.id) {
                            log::error!("Failed to regenerate series: {}", e);
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.invoke_show_toast(format!("重新生成系列失败: {}", e).into());
                            }
                        } else if let Some(ui) = ui_weak.upgrade() {
                            vm.init_ui(&ui);
                            ui.invoke_show_toast("系列已重新生成".into());
                        }
                    }
                }
            }
        });

        // Request add fonds dialog callback
        ui_handle.on_request_add_fonds_dialog({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move || {
                if let Ok(vm_ref) = vm.try_borrow() {
                    if let Some(ui) = ui_weak.upgrade() {
                        // Prepare dialog data via ui properties
                        let schemas = vm_ref.get_schema_options();
                        let available: Vec<_> = schemas.into_iter().map(|s| slint_generatedAppWindow::FondsSchemaOption {
                            id: s.id,
                            schema_no: s.schema_no,
                            name: s.name,
                        }).collect();
                        
                        let primary_classifications = vm_ref.get_primary_classifications();
                        let primary_codes = vm_ref.get_primary_codes();
                        let secondary_classifications = vm_ref.get_secondary_classifications();
                        let secondary_codes = vm_ref.get_secondary_codes();
                        
                        // Set dialog data via app-window properties
                        ui.set_available_schemas(ModelRc::new(VecModel::from(available)));
                        ui.set_selected_schemas(ModelRc::new(VecModel::from(Vec::new())));
                        ui.set_primary_classifications(ModelRc::new(VecModel::from(primary_classifications.clone())));
                        ui.set_primary_codes(ModelRc::new(VecModel::from(primary_codes.clone())));
                        
                        let secondary_models: Vec<ModelRc<SharedString>> = secondary_classifications.into_iter()
                            .map(|vec| ModelRc::new(VecModel::from(vec)))
                            .collect();
                        ui.set_secondary_classifications(ModelRc::new(VecModel::from(secondary_models)));
                        
                        let secondary_code_models: Vec<ModelRc<SharedString>> = secondary_codes.into_iter()
                            .map(|vec| ModelRc::new(VecModel::from(vec)))
                            .collect();
                        ui.set_secondary_codes(ModelRc::new(VecModel::from(secondary_code_models)));
                        
                        // Clear input
                        ui.set_add_fonds_name("".into());
                        ui.set_selected_primary_classification(0);
                        ui.set_selected_secondary_classification(0);
                        ui.set_highlighted_available_schema(-1);
                        ui.set_highlighted_chosen_schema(-1);
                        
                        // Show the dialog
                        ui.set_show_add_fonds_dialog(true);
                    }
                }
            }
        });

        // Confirm add fonds callback
        ui_handle.on_confirm_add_fonds({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |fonds_name, selected_primary, selected_secondary, selected_schemas| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    // Calculate actual classification_code from selections
                    let actual_code = if let Some(ui) = ui_weak.upgrade() {
                        let primary_codes = ui.get_primary_codes();
                        let secondary_codes = ui.get_secondary_codes();
                        let selected_primary = selected_primary as usize;
                        let selected_secondary = selected_secondary as usize;
                        
                        if selected_primary < primary_codes.row_count() {
                            if let Some(secondary_for_primary) = secondary_codes.row_data(selected_primary) {
                                if secondary_for_primary.row_count() > 0 && selected_secondary < secondary_for_primary.row_count() {
                                    if let Some(code) = secondary_for_primary.row_data(selected_secondary) {
                                        code.to_string()
                                    } else {
                                        "".to_string()
                                    }
                                } else {
                                    if let Some(code) = primary_codes.row_data(selected_primary) {
                                        code.to_string()
                                    } else {
                                        "".to_string()
                                    }
                                }
                            } else {
                                if let Some(code) = primary_codes.row_data(selected_primary) {
                                    code.to_string()
                                } else {
                                    "".to_string()
                                }
                            }
                        } else {
                            "".to_string()
                        }
                    } else {
                        "".to_string()
                    };
                    
                    // Convert selected_schemas to Vec<String> (extract schema_no)
                    let schema_nos: Vec<String> = selected_schemas.iter().map(|s| s.schema_no.to_string()).collect();
                    if let Err(e) = vm.add_fond(&fonds_name, &actual_code, schema_nos) {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("添加全宗失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.set_show_add_fonds_dialog(false);
                        ui.invoke_show_toast("全宗添加成功".into());
                    }
                }
            }
        });

        // Cancel add fonds callback
        ui_handle.on_cancel_add_fonds({
            let ui_weak = ui_weak.clone();
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_show_add_fonds_dialog(false);
                }
            }
        });

        // Move schema to selected callback
        ui_handle.on_move_schema_to_selected({
            let ui_weak = ui_weak.clone();
            move |index| {
                if let Some(ui) = ui_weak.upgrade() {
                    let available = ui.get_available_schemas();
                    let chosen = ui.get_selected_schemas();
                    
                    let mut available_vec = Vec::new();
                    for i in 0..available.row_count() {
                        if let Some(item) = available.row_data(i) {
                            available_vec.push(item);
                        }
                    }
                    
                    let mut chosen_vec = Vec::new();
                    for i in 0..chosen.row_count() {
                        if let Some(item) = chosen.row_data(i) {
                            chosen_vec.push(item);
                        }
                    }
                    
                    if index >= 0 && (index as usize) < available_vec.len() {
                        let item = available_vec.remove(index as usize);
                        chosen_vec.push(item);
                        ui.set_available_schemas((&available_vec[..]).into());
                        ui.set_selected_schemas((&chosen_vec[..]).into());
                        ui.set_highlighted_available_schema(-1);
                    }
                }
            }
        });

        // Move schema back callback
        ui_handle.on_move_schema_back({
            let ui_weak = ui_weak.clone();
            move |index| {
                if let Some(ui) = ui_weak.upgrade() {
                    let available = ui.get_available_schemas();
                    let chosen = ui.get_selected_schemas();
                    
                    let mut available_vec = Vec::new();
                    for i in 0..available.row_count() {
                        if let Some(item) = available.row_data(i) {
                            available_vec.push(item);
                        }
                    }
                    
                    let mut chosen_vec = Vec::new();
                    for i in 0..chosen.row_count() {
                        if let Some(item) = chosen.row_data(i) {
                            chosen_vec.push(item);
                        }
                    }
                    
                    if index >= 0 && (index as usize) < chosen_vec.len() {
                        let item = chosen_vec.remove(index as usize);
                        available_vec.push(item);
                        ui.set_available_schemas((&available_vec[..]).into());
                        ui.set_selected_schemas((&chosen_vec[..]).into());
                        ui.set_highlighted_chosen_schema(-1);
                    }
                }
            }
        });

        // File clicked callback
        ui_handle.on_file_clicked({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.selected_file = index;
                    vm.selected_file_index = index;
                    if let Some(file) = vm.files_list.get(index as usize) {
                        let file_id = file.id;
                        if let Err(e) = vm.load_items(file_id) {
                            log::error!("Failed to load items: {}", e);
                        }
                        if let Some(ui) = ui_weak.upgrade() {
                            vm.init_ui(&ui);
                        }
                    }
                }
            }
        });
        
        // Item clicked callback
        ui_handle.on_item_clicked({
            let vm = Rc::clone(&vm);
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.selected_item = index;
                    vm.selected_item_index = index;
                }
            }
        });

        // Add file callback
        ui_handle.on_add_file({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move || {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.show_add_file_dialog = true;
                    vm.new_file_name.clear();
                    vm.new_file_path.clear();
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_show_add_file_dialog(true);
                    }
                }
            }
        });

        // Delete file callback
        ui_handle.on_delete_file({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move || {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    if let Err(e) = vm.delete_file() {
                        log::error!("Failed to delete file: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("删除案卷失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.invoke_show_toast("案卷已删除".into());
                    }
                }
            }
        });

        // Rename file callback
        ui_handle.on_rename_file({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |index, new_name| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    if let Err(e) = vm.rename_file(index, &new_name) {
                        log::error!("Failed to rename file: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("重命名案卷失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.invoke_show_toast("案卷已重命名".into());
                    }
                }
            }
        });

        // Add item callback
        ui_handle.on_add_item({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move || {
                // Show folder picker dialog
                use rfd::FileDialog;
                if let Some(folder_path) = FileDialog::new()
                    .set_directory("/")
                    .pick_folder() {
                    let folder_name = folder_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("新文件")
                        .to_string();
                    let path_str = folder_path.to_string_lossy().to_string();
                    
                    if let Ok(mut vm) = vm.try_borrow_mut() {
                        if let Err(e) = vm.add_item(&folder_name, Some(path_str)) {
                            log::error!("Failed to add item: {}", e);
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.invoke_show_toast(format!("添加文件失败: {}", e).into());
                            }
                        } else if let Some(ui) = ui_weak.upgrade() {
                            vm.init_ui(&ui);
                            ui.invoke_show_toast("文件添加成功".into());
                        }
                    }
                }
            }
        });

        // Add folder item callback
        ui_handle.on_add_folder_item({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move || {
                // Show folder picker dialog
                use rfd::FileDialog;
                if let Some(folder_path) = FileDialog::new()
                    .set_directory("/")
                    .pick_folder() {
                    let folder_name = folder_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("新文件夹")
                        .to_string();
                    let path_str = folder_path.to_string_lossy().to_string();
                    
                    if let Ok(mut vm) = vm.try_borrow_mut() {
                        if let Err(e) = vm.add_item(&folder_name, Some(path_str)) {
                            log::error!("Failed to add folder item: {}", e);
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.invoke_show_toast(format!("添加文件夹失败: {}", e).into());
                            }
                        } else if let Some(ui) = ui_weak.upgrade() {
                            vm.init_ui(&ui);
                            ui.invoke_show_toast("文件夹添加成功".into());
                        }
                    }
                }
            }
        });

        // Delete item callback
        ui_handle.on_delete_item({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move || {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    if let Err(e) = vm.delete_item() {
                        log::error!("Failed to delete item: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("删除文件失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.invoke_show_toast("文件已删除".into());
                    }
                }
            }
        });

        // Rename item callback
        ui_handle.on_rename_item({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |index, new_name| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    if let Err(e) = vm.rename_item(index, &new_name) {
                        log::error!("Failed to rename item: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("重命名文件失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.invoke_show_toast("文件已重命名".into());
                    }
                }
            }
        });

        // Confirm add file callback
        ui_handle.on_confirm_add_file({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |fields| {
                let file_name = if fields.row_count() >= 1 {
                    fields.row_data(0).unwrap().value.to_string()
                } else {
                    String::new()
                };
                
                // File number is always auto-generated
                let file_no = match vm.borrow_mut().generate_next_file_no() {
                    Ok(no) => no,
                    Err(e) => {
                        log::error!("Failed to generate file number: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast("生成文件编号失败".into());
                        }
                        return;
                    }
                };
                
                log::info!("Adding file: name='{}', file_no='{}'", file_name, file_no);
                
                // Validate input
                if file_name.trim().is_empty() {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.invoke_show_toast("文件名不能为空".into());
                    }
                    return;
                }
                
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    // Add file with auto-generated path
                    if let Err(e) = vm.add_file(&file_name, &file_no, None) {
                        log::error!("Failed to add file: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("添加案卷失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.invoke_show_toast("案卷添加成功".into());
                    }
                    
                    vm.show_add_file_dialog = false;
                    vm.new_file_name.clear();
                    vm.new_file_path.clear();
                }
            }
        });
        
        // Cancel add file callback
        ui_handle.on_cancel_add_file({
            let vm = Rc::clone(&vm);
            move || {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.show_add_file_dialog = false;
                    vm.new_file_name.clear();
                    vm.new_file_path.clear();
                }
            }
        });
        
        // Browse file folder callback
        ui_handle.on_browse_file_folder({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |field_index| {
                log::info!("Browse file folder requested for field {}", field_index);
                
                let vm = vm.borrow();
                if let Some(path_str) = vm.browse_folder() {
                    if let Some(ui) = ui_weak.upgrade() {
                        log::info!("Selected folder: {}", path_str);
                        
                        // Update the path field
                        let fields = ui.get_add_file_fields();
                        if field_index == 1 && fields.row_count() > 1 {
                            // Path field (index 1)
                            let mut field = fields.row_data(1).unwrap();
                            field.value = path_str.clone().into();
                            fields.set_row_data(1, field);
                            ui.set_add_file_fields(fields);
                            
                            // Auto-fill name field with folder name if it's empty
                            let fields = ui.get_add_file_fields();
                            if fields.row_count() > 0 {
                                let name_field = fields.row_data(0).unwrap();
                                if name_field.value.trim().is_empty() {
                                    if let Some(folder_name) = std::path::Path::new(&path_str)
                                        .file_name()
                                        .and_then(|n| n.to_str()) {
                                        let mut name_field = fields.row_data(0).unwrap();
                                        name_field.value = folder_name.into();
                                        fields.set_row_data(0, name_field);
                                        ui.set_add_file_fields(fields);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    log::info!("No folder selected");
                }
            }
        });
        
        // Confirm add item callback
        ui_handle.on_confirm_add_item({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |fields| {
                let item_name = if fields.row_count() >= 1 {
                    fields.row_data(0).unwrap().value.to_string()
                } else {
                    String::new()
                };
                
                let item_path = if fields.row_count() >= 2 {
                    fields.row_data(1).unwrap().value.to_string()
                } else {
                    String::new()
                };
                
                log::info!("Adding item: name='{}', path='{}'", item_name, item_path);
                
                // Validate input
                if item_name.trim().is_empty() {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.invoke_show_toast("文件名不能为空".into());
                    }
                    return;
                }
                
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    // Add item
                    if let Err(e) = vm.add_item(&item_name, Some(item_path)) {
                        log::error!("Failed to add item: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("添加文件失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.invoke_show_toast("文件添加成功".into());
                    }
                    
                    vm.show_add_item_dialog = false;
                    vm.new_item_name.clear();
                    vm.new_item_path.clear();
                }
            }
        });
        
        // Cancel add item callback
        ui_handle.on_cancel_add_item({
            let vm = Rc::clone(&vm);
            move || {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.show_add_item_dialog = false;
                    vm.new_item_name.clear();
                    vm.new_item_path.clear();
                }
            }
        });

        // Confirm rename file callback
        ui_handle.on_confirm_rename_file({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |fields| {
                let new_name = if fields.row_count() >= 1 {
                    fields.row_data(0).unwrap().value.to_string()
                } else {
                    String::new()
                };

                // Validate input
                if new_name.trim().is_empty() {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.invoke_show_toast("文件名不能为空".into());
                    }
                    return;
                }

                if let Ok(mut vm) = vm.try_borrow_mut() {
                    let index = vm.rename_file_index;
                    if let Err(e) = vm.rename_file(index, &new_name) {
                        log::error!("Failed to rename file: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("重命名案卷失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.invoke_show_toast("案卷已重命名".into());
                    }

                    vm.show_rename_file_dialog = false;
                    vm.rename_file_index = -1;
                }
            }
        });

        // Cancel rename file callback
        ui_handle.on_cancel_rename_file({
            let vm = Rc::clone(&vm);
            move || {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.show_rename_file_dialog = false;
                    vm.rename_file_index = -1;
                }
            }
        });

        // Confirm rename item callback
        ui_handle.on_confirm_rename_item({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |fields| {
                let new_name = if fields.row_count() >= 1 {
                    fields.row_data(0).unwrap().value.to_string()
                } else {
                    String::new()
                };

                // Validate input
                if new_name.trim().is_empty() {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.invoke_show_toast("文件名不能为空".into());
                    }
                    return;
                }

                if let Ok(mut vm) = vm.try_borrow_mut() {
                    let index = vm.rename_item_index;
                    if let Err(e) = vm.rename_item(index, &new_name) {
                        log::error!("Failed to rename item: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("重命名文件失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.invoke_show_toast("文件已重命名".into());
                    }

                    vm.show_rename_item_dialog = false;
                    vm.rename_item_index = -1;
                }
            }
        });

        // Cancel rename item callback
        ui_handle.on_cancel_rename_item({
            let vm = Rc::clone(&vm);
            move || {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.show_rename_item_dialog = false;
                    vm.rename_item_index = -1;
                }
            }
        });

        // Set rename file index callback
        ui_handle.on_set_rename_file_index({
            let vm = Rc::clone(&vm);
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.rename_file_index = index;
                }
            }
        });

        // Set rename item index callback
        ui_handle.on_set_rename_item_index({
            let vm = Rc::clone(&vm);
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.rename_item_index = index;
                }
            }
        });
        
        // Browse item folder callback
        ui_handle.on_browse_item_folder({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move |field_index| {
                log::info!("Browse item file/folder requested for field {}", field_index);
                
                let vm = vm.borrow();
                if let Some(path_str) = vm.browse_file_or_folder() {
                    if let Some(ui) = ui_weak.upgrade() {
                        log::info!("Selected path: {}", path_str);
                        
                        // Update the path field
                        let fields = ui.get_add_item_fields();
                        if field_index == 1 && fields.row_count() > 1 {
                            // Path field
                            let mut field = fields.row_data(1).unwrap();
                            field.value = path_str.clone().into();
                            fields.set_row_data(1, field);
                            ui.set_add_item_fields(fields);
                            
                            // Auto-fill name field with file/folder name if it's empty
                            let fields = ui.get_add_item_fields();
                            if fields.row_count() > 0 {
                                let name_field = fields.row_data(0).unwrap();
                                if name_field.value.trim().is_empty() {
                                    if let Some(name) = std::path::Path::new(&path_str)
                                        .file_name()
                                        .and_then(|n| n.to_str()) {
                                        let mut name_field = fields.row_data(0).unwrap();
                                        name_field.value = name.into();
                                        fields.set_row_data(0, name_field);
                                        ui.set_add_item_fields(fields);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    log::info!("No path selected");
                }
            }
        });
        
        // Home page initialization callback
        ui_handle.on_initialize_home_page({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    if let Ok(mut vm) = vm.try_borrow_mut() {
                        if let Err(e) = vm.load_libraries() {
                            ui.invoke_show_toast(format!("加载档案库失败: {}", e).into());
                            return;
                        }
                        vm.init_ui(&ui);
                    }
                }
            }
        });

        // Open file at index callback
        ui_handle.on_open_file_at({
            let vm = Rc::clone(&vm);
            move |index| {
                if let Ok(vm_ref) = vm.try_borrow() {
                    if let Some(file) = vm_ref.files_list.get(index as usize) {
                        if let Some(ref path) = file.path {
                            if let Err(e) = open::that(path) {
                                log::error!("Failed to open file at path {}: {}", path, e);
                            } else {
                                log::info!("Opened file: {}", path);
                            }
                        } else {
                            log::warn!("Cannot open file '{}' - no path available", file.name);
                        }
                    }
                }
            }
        });

        // Open item at index callback
        ui_handle.on_open_item_at({
            let vm = Rc::clone(&vm);
            move |index| {
                if let Ok(vm_ref) = vm.try_borrow() {
                    if let Some(item) = vm_ref.items_list.get(index as usize) {
                        if let Some(ref path) = item.path {
                            if let Err(e) = open::that(path) {
                                log::error!("Failed to open item at path {}: {}", path, e);
                            } else {
                                log::info!("Opened item: {}", path);
                            }
                        } else {
                            log::warn!("Cannot open item '{}' - no path available", item.name);
                        }
                    }
                }
            }
        });
    }
    
    /// Static method to setup callbacks (called from App)
    pub fn setup_callbacks(vm: Rc<RefCell<Self>>, ui_handle: &AppWindow) {
        // First load libraries data
        {
            let mut vm_borrowed = vm.borrow_mut();
            if let Err(e) = vm_borrowed.load_libraries() {
                log::error!("HomeViewModel: Failed to load libraries: {}", e);
            }
        }
        // Then initialize UI
        {
            let vm_borrowed = vm.borrow();
            vm_borrowed.init_ui(ui_handle);
        }
        // Finally setup callbacks
        {
            let vm_borrowed = vm.borrow();
            vm_borrowed.setup_ui_callbacks(ui_handle, Rc::clone(&vm));
        }
    }
}

impl Clone for HomeViewModel {
    fn clone(&self) -> Self {
        Self {
            library_names: self.library_names.clone(),
            selected_archive_index: self.selected_archive_index,
            selected_file_index: self.selected_file_index,
            selected_item_index: self.selected_item_index,
            last_opened_library: self.last_opened_library.clone(),
            fonds_list: self.fonds_list.clone(),
            selected_fonds_index: self.selected_fonds_index,
            series_list: self.series_list.clone(),
            selected_series_index: self.selected_series_index,
            selected_series_no: self.selected_series_no.clone(),
            files_list: self.files_list.clone(),
            selected_file: self.selected_file,
            items_list: self.items_list.clone(),
            selected_item: self.selected_item,
            show_add_file_dialog: self.show_add_file_dialog,
            new_file_name: self.new_file_name.clone(),
            new_file_path: self.new_file_path.clone(),
            show_add_item_dialog: self.show_add_item_dialog,
            new_item_name: self.new_item_name.clone(),
            new_item_path: self.new_item_path.clone(),
            show_rename_file_dialog: self.show_rename_file_dialog,
            rename_file_index: self.rename_file_index,
            show_rename_item_dialog: self.show_rename_item_dialog,
            rename_item_index: self.rename_item_index,
            settings_service: Rc::clone(&self.settings_service),
            db_connection: self.db_connection.as_ref().map(Rc::clone),
            current_db_path: self.current_db_path.clone(),
            sequences_repo: self.sequences_repo.as_ref().map(|repo| repo.clone_with_conn(Rc::clone(self.db_connection.as_ref().unwrap()))),
        }
    }
}
