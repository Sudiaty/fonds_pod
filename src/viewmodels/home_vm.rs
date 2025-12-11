/// Home View Model - MVVM architecture
/// Manages the state and business logic for the home page (fonds management)
use crate::services::SettingsService;
use crate::{AppWindow};
use slint::{ComponentHandle, ModelRc, VecModel, SharedString};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

/// Home ViewModel - handles state and business logic for fonds management
pub struct HomeViewModel {
    pub library_names: Vec<String>,
    pub selected_archive_index: i32,
    pub selected_file_index: i32,
    pub selected_item_index: i32,
    pub last_opened_library: String,
    settings_service: Rc<SettingsService>,
}

impl Default for HomeViewModel {
    fn default() -> Self {
        Self {
            library_names: Vec::new(),
            selected_archive_index: -1,
            selected_file_index: -1,
            selected_item_index: -1,
            last_opened_library: String::new(),
            settings_service: Rc::new(SettingsService::new()),
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
            settings_service,
        }
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
            }
        } else if !self.library_names.is_empty() {
            // If no last opened library, select the first one
            self.selected_archive_index = 0;
            if let Some(first_lib) = libraries.first() {
                self.last_opened_library = first_lib.path.clone();
                let _ = self.settings_service.set_last_opened_library(Some(first_lib.path.clone()));
            }
        }

        Ok(())
    }

    /// Set selected archive and update last opened library
    pub fn set_selected_archive(&mut self, index: i32) -> Result<(), Box<dyn Error>> {
        if index >= 0 && (index as usize) < self.library_names.len() {
            self.selected_archive_index = index;
            // Get the library path and update settings
            let libraries = self.settings_service.list_archive_libraries()?;
            if let Some(lib) = libraries.get(index as usize) {
                self.last_opened_library = lib.path.clone();
                self.settings_service.set_last_opened_library(Some(lib.path.clone()))?;
            }
        }
        Ok(())
    }

    /// Initialize UI with current data
    pub fn init_ui(&self, ui_handle: &AppWindow) {
        // Set library names
        let names: Vec<SharedString> = self.library_names.iter().map(|s| s.as_str().into()).collect();
        let names_model = ModelRc::new(VecModel::from(names));
        ui_handle.set_library_names(names_model);

        // Set selected archive index
        ui_handle.set_selected_archive(self.selected_archive_index);

        // Set last opened library
        ui_handle.set_last_opened_library(self.last_opened_library.clone().into());
    }

    /// Setup UI callbacks for the home page
    pub fn setup_ui_callbacks(&self, ui_handle: &AppWindow) {
        let vm = Rc::new(RefCell::new(self.clone()));

        // Archive selected callback
        ui_handle.on_archive_selected({
            let vm = Rc::clone(&vm);
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    if let Err(e) = vm.set_selected_archive(index) {
                        log::error!("Failed to set selected archive: {}", e);
                    }
                }
            }
        });
        
        // File clicked callback
        ui_handle.on_file_clicked({
            let vm = Rc::clone(&vm);
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.selected_file_index = index;
                    // TODO: Implement file selection logic
                }
            }
        });
        
        // Item clicked callback
        ui_handle.on_item_clicked({
            let vm = Rc::clone(&vm);
            move |index| {
                if let Ok(mut vm) = vm.try_borrow_mut() {
                    vm.selected_item_index = index;
                    // TODO: Implement item selection logic
                }
            }
        });
        
        // Home page initialization callback
        ui_handle.on_initialize_home_page({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_handle.as_weak();
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
        let vm_borrowed = vm.borrow();
        vm_borrowed.init_ui(ui_handle);
        vm_borrowed.setup_ui_callbacks(ui_handle);
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
            settings_service: Rc::clone(&self.settings_service),
        }
    }
}