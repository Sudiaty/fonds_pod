/// Settings View Model - MVVM architecture
/// Manages the state and business logic for the settings page
use crate::models::app_settings::ArchiveLibrary;
use crate::services::SettingsService;
use crate::{AppWindow, CrudListItem};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

/// Archive library item for UI display
#[derive(Clone, Debug)]
pub struct ArchiveLibraryUIItem {
    pub name: String,
    pub path: String,
}

impl From<&ArchiveLibrary> for ArchiveLibraryUIItem {
    fn from(lib: &ArchiveLibrary) -> Self {
        Self {
            name: lib.name.clone(),
            path: lib.path.clone(),
        }
    }
}

/// Settings ViewModel - handles state and business logic
pub struct SettingsViewModel {
    pub selected_language: i32,
    pub archive_libraries: Vec<ArchiveLibraryUIItem>,
    pub selected_archive_index: i32,
    pub new_archive_name: String,
    pub new_archive_path: String,
    pub show_add_archive_dialog: bool,
    pub show_rename_dialog: bool,
    pub rename_input: String,
    settings_service: Rc<SettingsService>,
}

impl Default for SettingsViewModel {
    fn default() -> Self {
        Self {
            selected_language: 0,
            archive_libraries: Vec::new(),
            selected_archive_index: -1,
            new_archive_name: String::new(),
            new_archive_path: String::new(),
            show_add_archive_dialog: false,
            show_rename_dialog: false,
            rename_input: String::new(),
            settings_service: Rc::new(SettingsService::new()),
        }
    }
}

impl SettingsViewModel {
    /// Create a new SettingsViewModel with the given settings service
    pub fn new(settings_service: Rc<SettingsService>) -> Self {
        Self {
            selected_language: 0,
            archive_libraries: Vec::new(),
            selected_archive_index: -1,
            new_archive_name: String::new(),
            new_archive_path: String::new(),
            show_add_archive_dialog: false,
            show_rename_dialog: false,
            rename_input: String::new(),
            settings_service,
        }
    }

    /// Load settings from service
    pub fn load_from_service(
        &mut self,
        language: String,
        libraries: Vec<ArchiveLibrary>,
    ) {
        // Set language (0 for Chinese, 1 for English)
        self.selected_language = if language.contains("en") { 1 } else { 0 };

        // Convert archive libraries to UI items
        self.archive_libraries = libraries.iter().map(ArchiveLibraryUIItem::from).collect();

        // Reset selected index
        self.selected_archive_index = if !self.archive_libraries.is_empty() {
            0
        } else {
            -1
        };
    }

    /// Validate add archive form
    pub fn validate_add_archive(&self) -> Result<(), Box<dyn Error>> {
        if self.new_archive_name.trim().is_empty() {
            return Err("Archive name cannot be empty".into());
        }

        if self.new_archive_path.trim().is_empty() {
            return Err("Archive path cannot be empty".into());
        }

        // Check if name already exists
        if self
            .archive_libraries
            .iter()
            .any(|lib| lib.name == self.new_archive_name)
        {
            return Err("Archive name already exists".into());
        }

        Ok(())
    }

    /// Validate rename form
    pub fn validate_rename(&self) -> Result<(), Box<dyn Error>> {
        if self.rename_input.trim().is_empty() {
            return Err("Archive name cannot be empty".into());
        }

        if self.selected_archive_index < 0
            || self.selected_archive_index as usize >= self.archive_libraries.len()
        {
            return Err("No archive selected".into());
        }

        // Check if new name already exists (excluding current)
        let current_name = self.archive_libraries[self.selected_archive_index as usize].name.clone();
        if self.rename_input != current_name
            && self
                .archive_libraries
                .iter()
                .any(|lib| lib.name == self.rename_input)
        {
            return Err("Archive name already exists".into());
        }

        Ok(())
    }

    /// Add a new archive library
    pub fn add_archive(&mut self, name: String, path: String) {
        self.archive_libraries.push(ArchiveLibraryUIItem { name, path });
    }

    /// Remove an archive library by index
    pub fn remove_archive(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        if index < self.archive_libraries.len() {
            self.archive_libraries.remove(index);
            if self.selected_archive_index as usize >= self.archive_libraries.len() {
                self.selected_archive_index =
                    if self.archive_libraries.is_empty() { -1 } else { 0 };
            }
            Ok(())
        } else {
            Err("Archive index out of bounds".into())
        }
    }

    /// Rename an archive library
    pub fn rename_archive(&mut self, index: usize, new_name: String) -> Result<(), Box<dyn Error>> {
        if index < self.archive_libraries.len() {
            self.archive_libraries[index].name = new_name;
            Ok(())
        } else {
            Err("Archive index out of bounds".into())
        }
    }

    /// Get language setting for service
    pub fn get_language_for_service(&self) -> String {
        if self.selected_language == 1 {
            "en_US".to_string()
        } else {
            "zh_CN".to_string()
        }
    }

    /// Get archive libraries as service format
    pub fn get_archive_libraries_for_service(&self) -> Vec<ArchiveLibrary> {
        self.archive_libraries
            .iter()
            .map(|ui_item| ArchiveLibrary {
                name: ui_item.name.clone(),
                path: ui_item.path.clone(),
            })
            .collect()
    }

    /// Add archive library
    pub fn add_archive_library(&mut self, name: String, path: String) -> Result<(), Box<dyn Error>> {
        // Validate input
        if name.is_empty() || path.is_empty() {
            return Err("Name and path cannot be empty".into());
        }

        // Check for duplicate names
        if self.archive_libraries.iter().any(|lib| lib.name == name) {
            return Err("Archive name already exists".into());
        }

        // Add to service
        self.settings_service.add_archive_library(name.clone(), path.clone())?;

        // Add to VM
        self.archive_libraries.push(ArchiveLibraryUIItem { name, path });
        self.selected_archive_index = (self.archive_libraries.len() - 1) as i32;

        Ok(())
    }

    /// Remove archive library
    pub fn remove_archive_library(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        if index >= self.archive_libraries.len() {
            return Err("Archive index out of bounds".into());
        }

        // Remove from service
        self.settings_service.remove_archive_library(index)?;

        // Remove from VM
        self.archive_libraries.remove(index);

        // Update selection
        if self.archive_libraries.is_empty() {
            self.selected_archive_index = -1;
        } else if self.selected_archive_index as usize >= self.archive_libraries.len() {
            self.selected_archive_index = (self.archive_libraries.len() - 1) as i32;
        }

        Ok(())
    }

    /// Rename archive library
    pub fn rename_archive_library(&mut self, index: usize, new_name: String) -> Result<(), Box<dyn Error>> {
        if index >= self.archive_libraries.len() {
            return Err("Archive index out of bounds".into());
        }

        if new_name.is_empty() {
            return Err("Name cannot be empty".into());
        }

        // Rename in service
        self.settings_service.rename_archive_library(index, new_name.clone())?;

        // Rename in VM
        self.archive_libraries[index].name = new_name;

        Ok(())
    }

    /// Apply settings
    pub fn apply_settings(&mut self) -> Result<(), Box<dyn Error>> {
        let language = self.get_language_for_service();
        let libraries = self.get_archive_libraries_for_service();

        self.settings_service.apply_settings(language.clone(), libraries)?;

        // Apply language change immediately
        if !language.is_empty() {
            match slint::select_bundled_translation(&language) {
                Ok(_) => log::info!("Successfully selected bundled translation for: {}", language),
                Err(e) => log::warn!("Failed to select bundled translation for {}: {}", language, e),
            }
        }

        Ok(())
    }

    /// Cancel settings - reload from service
    pub fn cancel_settings(&mut self) -> Result<(), Box<dyn Error>> {
        let libraries = self.settings_service.list_archive_libraries()?;
        let language = self.settings_service.get_language()?;

        self.load_from_service(language, libraries);
        Ok(())
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

    /// Helper function to convert archive libraries to UI format
    fn to_ui_items(libraries: &[ArchiveLibraryUIItem]) -> ModelRc<CrudListItem> {
        let ui_items: Vec<CrudListItem> = libraries.iter().enumerate().map(|(idx, lib)| {
            CrudListItem {
                id: idx as i32,
                title: lib.name.clone().into(),
                subtitle: lib.path.clone().into(),
                active: true,
            }
        }).collect();
        ModelRc::new(VecModel::from(ui_items))
    }

    /// Initialize UI with current ViewModel state
    pub fn init_ui(&self, ui_handle: &AppWindow) {
        // 设置语言
        ui_handle.set_selected_language(self.selected_language);
        
        // 设置档案库列表
        ui_handle.set_archive_libraries(Self::to_ui_items(&self.archive_libraries));
        
        // 设置选中的档案库
        ui_handle.set_selected_archive(self.selected_archive_index);
        
        log::info!("Initialized UI with {} archive libraries", self.archive_libraries.len());
    }

    /// Setup all UI callbacks for settings page
    pub fn setup_callbacks(vm: Rc<RefCell<Self>>, ui_handle: &AppWindow) {
        // 添加档案库
        ui_handle.on_add_archive_library({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_handle.as_weak();
            
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut vm = vm.borrow_mut();
                    
                    // 从UI同步数据到ViewModel
                    let fields = ui.get_add_archive_fields();
                    if fields.row_count() >= 2 {
                        vm.new_archive_name = fields.row_data(0).unwrap().value.to_string();
                        vm.new_archive_path = fields.row_data(1).unwrap().value.to_string();
                    }
                    
                    log::info!("Adding archive: name='{}', path='{}'", vm.new_archive_name, vm.new_archive_path);
                    
                    // 验证输入
                    if let Err(e) = vm.validate_add_archive() {
                        ui.invoke_show_toast(format!("Error: {}", e).into());
                        return;
                    }
                    
                    let new_name = vm.new_archive_name.clone();
                    let new_path = vm.new_archive_path.clone();
                    
                    // 添加档案库
                    if let Err(e) = vm.add_archive_library(new_name, new_path) {
                        ui.invoke_show_toast(format!("Failed to add archive: {}", e).into());
                        return;
                    }
                    
                    vm.new_archive_name.clear();
                    vm.new_archive_path.clear();
                    
                    // 更新UI
                    ui.set_archive_libraries(Self::to_ui_items(&vm.archive_libraries));
                    ui.invoke_show_toast("Archive library added successfully".into());
                }
            }
        });
        
        // 删除档案库
        ui_handle.on_remove_archive_library({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_handle.as_weak();
            
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut vm = vm.borrow_mut();
                    
                    if vm.selected_archive_index < 0 {
                        ui.invoke_show_toast("Please select an archive to remove".into());
                        return;
                    }
                    
                    let index = vm.selected_archive_index as usize;
                    
                    if let Err(e) = vm.remove_archive_library(index) {
                        ui.invoke_show_toast(format!("Failed to remove archive: {}", e).into());
                        return;
                    }
                    
                    ui.set_archive_libraries(Self::to_ui_items(&vm.archive_libraries));
                    ui.set_selected_archive(vm.selected_archive_index);
                    ui.invoke_show_toast("Archive library removed successfully".into());
                }
            }
        });
        
        // 重命名档案库
        ui_handle.on_rename_archive_library({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_handle.as_weak();
            
            move |index: i32, new_name: slint::SharedString| {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut vm = vm.borrow_mut();
                    
                    if new_name.is_empty() {
                        ui.invoke_show_toast("Archive name cannot be empty".into());
                        return;
                    }
                    
                    if let Err(e) = vm.rename_archive_library(index as usize, new_name.to_string()) {
                        ui.invoke_show_toast(format!("Failed to rename archive: {}", e).into());
                        return;
                    }
                    
                    ui.set_archive_libraries(Self::to_ui_items(&vm.archive_libraries));
                    ui.invoke_show_toast("Archive library renamed successfully".into());
                }
            }
        });
        
        // 选择档案库
        ui_handle.on_archive_selected({
            let vm = Rc::clone(&vm);
            move |index: i32| {
                let mut vm = vm.borrow_mut();
                vm.selected_archive_index = index;
                log::debug!("Archive selected: index={}", index);
            }
        });
        
        // 应用设置
        ui_handle.on_apply_settings({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_handle.as_weak();
            
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut vm = vm.borrow_mut();
                    
                    vm.selected_language = ui.get_selected_language();
                    
                    if let Err(e) = vm.apply_settings() {
                        ui.invoke_show_toast(format!("Failed to apply settings: {}", e).into());
                        return;
                    }
                    
                    ui.invoke_show_toast("Settings applied successfully".into());
                }
            }
        });
        
        // 取消设置
        ui_handle.on_cancel_settings({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_handle.as_weak();
            
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut vm = vm.borrow_mut();
                    
                    if let Err(e) = vm.cancel_settings() {
                        ui.invoke_show_toast(format!("Failed to cancel settings: {}", e).into());
                        return;
                    }
                    
                    ui.set_selected_language(vm.selected_language);
                    ui.set_archive_libraries(Self::to_ui_items(&vm.archive_libraries));
                    ui.set_selected_archive(vm.selected_archive_index);
                    ui.invoke_show_toast("Settings cancelled".into());
                }
            }
        });
        
        // 设置选择的路径
        ui_handle.on_set_selected_path({
            let ui_weak = ui_handle.as_weak();
            
            move |path: slint::SharedString| {
                if let Some(ui) = ui_weak.upgrade() {
                    let fields = ui.get_add_archive_fields();
                    if fields.row_count() >= 2 {
                        if let Some(mut path_field) = fields.row_data(1) {
                            path_field.value = path.clone();
                            fields.set_row_data(1, path_field);
                            ui.set_add_archive_fields(fields.clone());
                            
                            // 如果当前名称为空，从路径中提取文件夹名作为默认名称
                            if let Some(name_field) = fields.row_data(0) {
                                if name_field.value.is_empty() && !path.is_empty() {
                                    let path_str = path.as_str();
                                    let folder_name = std::path::Path::new(path_str)
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or("")
                                        .to_string();
                                    
                                    if !folder_name.is_empty() {
                                        let mut name_field = fields.row_data(0).unwrap();
                                        name_field.value = folder_name.into();
                                        fields.set_row_data(0, name_field);
                                        ui.set_add_archive_fields(fields);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        
        // 浏览文件夹
        ui_handle.on_browse_folder({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_handle.as_weak();
            
            move || {
                log::info!("Browse folder requested");
                
                let vm = vm.borrow();
                if let Some(path_str) = vm.browse_folder() {
                    if let Some(ui) = ui_weak.upgrade() {
                        log::info!("Selected folder: {}", path_str);
                        ui.invoke_set_selected_path(path_str.into());
                    }
                } else {
                    log::info!("No folder selected");
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_add_archive_empty_name() {
        let vm = SettingsViewModel::new(Rc::new(SettingsService::new()));
        assert!(vm.validate_add_archive().is_err());
    }

    #[test]
    fn test_language_conversion() {
        let mut vm = SettingsViewModel::new(Rc::new(SettingsService::new()));
        vm.selected_language = 0;
        assert_eq!(vm.get_language_for_service(), "zh_CN");

        vm.selected_language = 1;
        assert_eq!(vm.get_language_for_service(), "en_US");
    }
}
