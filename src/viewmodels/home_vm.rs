/// Home View Model - MVVM architecture
/// Manages the state and business logic for the home page (fonds management)
use crate::services::SettingsService;
use crate::{AppWindow, CrudListItem};
use crate::persistence::{
    FondsRepository, SeriesRepository, FilesRepository, ItemsRepository,
    FondSchemasRepository, FondClassificationsRepository, SchemaRepository, SchemaItemRepository,
    establish_connection,
};
use crate::models::fond::Fond;
use crate::models::series::Series;
use crate::models::file::File;
use crate::models::item::Item;
use crate::models::fond_schema::FondSchema;
use crate::core::GenericRepository;
use slint::{ComponentHandle, ModelRc, VecModel, SharedString};
use crate::slint_generatedAppWindow;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::path::PathBuf;
use diesel::SqliteConnection;

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
    
    settings_service: Rc<SettingsService>,
    db_connection: Option<Rc<RefCell<SqliteConnection>>>,
    current_db_path: Option<PathBuf>,
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
            settings_service: Rc::new(SettingsService::new()),
            db_connection: None,
            current_db_path: None,
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
            settings_service,
            db_connection: None,
            current_db_path: None,
        }
    }

    /// Update database connection for the selected archive
    fn update_connection(&mut self, library_path: &str) -> Result<(), Box<dyn Error>> {
        let db_path = PathBuf::from(library_path).join(".fondspod.db");
        let conn = establish_connection(&db_path)?;
        self.db_connection = Some(conn);
        self.current_db_path = Some(db_path);
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
            
            // If we have fonds and one is selected, load its series
            if !self.fonds_list.is_empty() && self.selected_fonds_index >= 0 {
                let fond_no = self.fonds_list.get(self.selected_fonds_index as usize)
                    .map(|f| f.fond_no.clone());
                if let Some(fond_no) = fond_no {
                    self.load_series(&fond_no)?;
                }
            }
        }
        Ok(())
    }

    /// Load series for a specific fond
    pub fn load_series(&mut self, fond_no: &str) -> Result<(), Box<dyn Error>> {
        if let Some(mut repo) = self.get_series_repo() {
            // Find series for this fond
            let all_series = repo.find_all()?;
            self.series_list = all_series.into_iter()
                .filter(|s| s.fond_no == fond_no)
                .collect();
            log::info!("HomeViewModel: Loaded {} series for fond {}", self.series_list.len(), fond_no);
            
            // Reset selection and load files for first series
            if !self.series_list.is_empty() {
                self.selected_series_index = 0;
                self.selected_series_no = self.series_list[0].series_no.clone();
                self.load_files(&self.selected_series_no.clone())?;
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
    pub fn load_files(&mut self, series_no: &str) -> Result<(), Box<dyn Error>> {
        if let Some(mut repo) = self.get_files_repo() {
            let all_files = repo.find_all()?;
            self.files_list = all_files.into_iter()
                .filter(|f| f.series_no == series_no)
                .collect();
            log::info!("HomeViewModel: Loaded {} files for series {}", self.files_list.len(), series_no);
            
            // Reset selection and load items for first file
            if !self.files_list.is_empty() {
                self.selected_file = 0;
                self.load_items(&self.files_list[0].file_no.clone())?;
            } else {
                self.selected_file = 0;
                self.items_list.clear();
            }
        }
        Ok(())
    }

    /// Load items for a specific file
    pub fn load_items(&mut self, file_no: &str) -> Result<(), Box<dyn Error>> {
        if let Some(mut repo) = self.get_items_repo() {
            let all_items = repo.find_all()?;
            self.items_list = all_items.into_iter()
                .filter(|i| i.file_no == file_no)
                .collect();
            log::info!("HomeViewModel: Loaded {} items for file {}", self.items_list.len(), file_no);
            self.selected_item = 0;
        }
        Ok(())
    }

    /// Generate series for a fond based on fond_schemas (cartesian product of schema items)
    pub fn generate_series(&mut self, fond_no: &str) -> Result<(), Box<dyn Error>> {
        // Get fond_schemas for this fond
        let fond_schemas = if let Some(mut repo) = self.get_fond_schemas_repo() {
            let all_schemas = repo.find_all()?;
            let mut schemas: Vec<_> = all_schemas.into_iter()
                .filter(|fs| fs.fond_no == fond_no)
                .collect();
            schemas.sort_by_key(|s| s.order_no);
            schemas
        } else {
            return Err("No database connection".into());
        };

        if fond_schemas.is_empty() {
            log::warn!("No fond_schemas found for fond {}", fond_no);
            return Ok(());
        }

        // Get schema items for each fond_schema
        let mut dimension_items: Vec<Vec<crate::models::schema_item::SchemaItem>> = Vec::new();
        
        if let Some(mut schema_repo) = self.get_schema_repo() {
            if let Some(mut items_repo) = self.get_schema_items_repo() {
                for fond_schema in &fond_schemas {
                    // Find the schema by schema_no
                    let all_schemas = schema_repo.find_all()?;
                    if let Some(schema) = all_schemas.iter().find(|s| s.schema_no == fond_schema.schema_no) {
                        let items = items_repo.find_by_schema_id(schema.id)?;
                        if !items.is_empty() {
                            dimension_items.push(items);
                        }
                    }
                }
            }
        }

        if dimension_items.is_empty() {
            log::warn!("No schema items found for fond {}", fond_no);
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

        // Delete existing series for this fond
        if let Some(mut series_repo) = self.get_series_repo() {
            let existing_series = series_repo.find_all()?;
            for series in existing_series.iter().filter(|s| s.fond_no == fond_no) {
                series_repo.delete(series.id)?;
            }

            // Create new series
            for (idx, combo) in series_combinations.iter().enumerate() {
                let series_no = format!("{}-{:03}", fond_no, idx + 1);
                let name = combo.iter()
                    .map(|item| item.item_name.as_str())
                    .collect::<Vec<_>>()
                    .join("-");
                
                let series = Series {
                    id: 0,
                    series_no: series_no.clone(),
                    fond_no: fond_no.to_string(),
                    name,
                    created_by: String::new(),
                    created_machine: String::new(),
                    created_at: chrono::Utc::now().naive_utc(),
                };
                series_repo.create(series)?;
            }
            
            log::info!("Generated {} series for fond {}", series_combinations.len(), fond_no);
        }

        // Reload series
        self.load_series(fond_no)?;
        Ok(())
    }

    /// Add a new fond with the given data
    pub fn add_fond(&mut self, name: &str, classification_code: &str, selected_schema_nos: Vec<String>) -> Result<(), Box<dyn Error>> {
        // Generate fond_no
        let fond_no = format!("F{:04}", self.fonds_list.len() + 1);
        
        // Create the fond
        if let Some(mut repo) = self.get_fonds_repo() {
            let fond = Fond {
                id: 0,
                fond_no: fond_no.clone(),
                fond_classification_code: classification_code.to_string(),
                name: name.to_string(),
                created_by: String::new(),
                created_machine: String::new(),
                created_at: chrono::Utc::now().naive_utc(),
            };
            repo.create(fond)?;
            log::info!("Created fond: {} - {}", fond_no, name);
        }

        // Create fond_schemas for selected schemas
        if let Some(mut fs_repo) = self.get_fond_schemas_repo() {
            for (order, schema_no) in selected_schema_nos.iter().enumerate() {
                let fond_schema = FondSchema {
                    id: 0,
                    fond_no: fond_no.clone(),
                    schema_no: schema_no.clone(),
                    order_no: order as i32,
                    created_by: String::new(),
                    created_machine: String::new(),
                    created_at: chrono::Utc::now().naive_utc(),
                };
                fs_repo.create(fond_schema)?;
            }
            log::info!("Created {} fond_schemas for fond {}", selected_schema_nos.len(), fond_no);
        }

        // Generate series based on the schemas
        self.generate_series(&fond_no)?;

        // Reload fonds
        self.load_fonds()?;
        
        // Select the newly created fond
        self.selected_fonds_index = self.fonds_list.len() as i32 - 1;

        Ok(())
    }

    /// Add a new file to the selected series
    pub fn add_file(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        if self.selected_series_no.is_empty() {
            return Err("No series selected".into());
        }
        
        let file_no = format!("{}-W{:03}", self.selected_series_no, self.files_list.len() + 1);
        
        if let Some(mut repo) = self.get_files_repo() {
            let file = File {
                id: 0,
                file_no: file_no.clone(),
                series_no: self.selected_series_no.clone(),
                name: name.to_string(),
                created_by: String::new(),
                created_machine: String::new(),
                created_at: chrono::Utc::now().naive_utc(),
            };
            repo.create(file)?;
            log::info!("Created file: {} - {}", file_no, name);
        }

        self.load_files(&self.selected_series_no.clone())?;
        Ok(())
    }

    /// Add a new item to the selected file
    pub fn add_item(&mut self, name: &str, path: Option<String>) -> Result<(), Box<dyn Error>> {
        if self.files_list.is_empty() || self.selected_file < 0 {
            return Err("No file selected".into());
        }
        
        let file_no = self.files_list[self.selected_file as usize].file_no.clone();
        let item_no = format!("{}-D{:03}", file_no, self.items_list.len() + 1);
        
        if let Some(mut repo) = self.get_items_repo() {
            let item = Item {
                id: 0,
                item_no: item_no.clone(),
                file_no: file_no.clone(),
                name: name.to_string(),
                path,
                created_by: String::new(),
                created_machine: String::new(),
                created_at: chrono::Utc::now().naive_utc(),
            };
            repo.create(item)?;
            log::info!("Created item: {} - {}", item_no, name);
        }

        self.load_items(&file_no)?;
        Ok(())
    }

    /// Delete the selected file
    pub fn delete_file(&mut self) -> Result<(), Box<dyn Error>> {
        if self.files_list.is_empty() || self.selected_file < 0 {
            return Err("No file selected".into());
        }
        
        let file_id = self.files_list[self.selected_file as usize].id;
        
        if let Some(mut repo) = self.get_files_repo() {
            repo.delete(file_id)?;
            log::info!("Deleted file with id {}", file_id);
        }

        self.load_files(&self.selected_series_no.clone())?;
        Ok(())
    }

    /// Delete the selected item
    pub fn delete_item(&mut self) -> Result<(), Box<dyn Error>> {
        if self.items_list.is_empty() || self.selected_item < 0 {
            return Err("No item selected".into());
        }
        
        let item_id = self.items_list[self.selected_item as usize].id;
        
        if let Some(mut repo) = self.get_items_repo() {
            repo.delete(item_id)?;
            log::info!("Deleted item with id {}", item_id);
        }

        if !self.files_list.is_empty() && self.selected_file >= 0 {
            self.load_items(&self.files_list[self.selected_file as usize].file_no.clone())?;
        }
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
                        let fond_no = fond.fond_no.clone();
                        if let Err(e) = vm.load_series(&fond_no) {
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
                        vm.selected_series_no = series.series_no.clone();
                        if let Err(e) = vm.load_files(&series.series_no) {
                            log::error!("Failed to load files: {}", e);
                        }
                        if let Some(ui) = ui_weak.upgrade() {
                            vm.init_ui(&ui);
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
                        if let Err(e) = vm.generate_series(&fond.fond_no) {
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
                        let secondary_classifications = vm_ref.get_secondary_classifications();
                        
                        // Set dialog data via app-window properties
                        ui.set_available_schemas(ModelRc::new(VecModel::from(available)));
                        ui.set_selected_schemas(ModelRc::new(VecModel::from(Vec::new())));
                        ui.set_primary_classifications(ModelRc::new(VecModel::from(primary_classifications.clone())));
                        
                        let secondary_models: Vec<ModelRc<SharedString>> = secondary_classifications.into_iter()
                            .map(|vec| ModelRc::new(VecModel::from(vec)))
                            .collect();
                        ui.set_secondary_classifications(ModelRc::new(VecModel::from(secondary_models)));
                        
                        // Clear input
                        ui.set_add_fonds_name("".into());
                        ui.set_selected_primary_classification(0);
                        ui.set_selected_secondary_classification(0);
                        
                        // Show the dialog
                        ui.set_show_add_fonds_dialog(true);
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
                        let file_no = file.file_no.clone();
                        if let Err(e) = vm.load_items(&file_no) {
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
                    // Generate a default name
                    let name = format!("新案卷 {}", vm.files_list.len() + 1);
                    if let Err(e) = vm.add_file(&name) {
                        log::error!("Failed to add file: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.invoke_show_toast(format!("添加案卷失败: {}", e).into());
                        }
                    } else if let Some(ui) = ui_weak.upgrade() {
                        vm.init_ui(&ui);
                        ui.invoke_show_toast("案卷添加成功".into());
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

        // Add item callback
        ui_handle.on_add_item({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_weak.clone();
            move || {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    // Generate a default name
                    let name = format!("新文件 {}", vm.items_list.len() + 1);
                    if let Err(e) = vm.add_item(&name, None) {
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
            settings_service: Rc::clone(&self.settings_service),
            db_connection: self.db_connection.as_ref().map(Rc::clone),
            current_db_path: self.current_db_path.clone(),
        }
    }
}