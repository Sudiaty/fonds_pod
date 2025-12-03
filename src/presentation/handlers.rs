/// UI event handlers - connects UI callbacks to application services
use crate::services::{SchemaService, ArchiveService, ClassificationService, FondsService, FileService, CreateFileInput, CreateFondsInput, CreateItemInput};
use crate::domain::ConfigRepository;
use std::rc::Rc;
use std::fs;
use std::path::Path;

// Import AppWindow and ComponentHandle trait from parent module (main.rs)
use crate::AppWindow;
use crate::FileInputDialog;
use slint::{ComponentHandle, Model};

// ============================================================================
// Helper functions for setting CrudListItem data alongside original data
// ============================================================================

/// Set series list with both SeriesItem and CrudListItem formats
fn set_series_with_crud_items(series_items: Vec<crate::SeriesItem>, ui: &AppWindow) {
    // Create CrudListItem format
    let crud_items: Vec<crate::CrudListItem> = series_items.iter()
        .map(|s| crate::CrudListItem {
            title: s.name.clone(),
            subtitle: s.series_no.clone(),
        })
        .collect();
    
    let series_model = slint::ModelRc::new(slint::VecModel::from(series_items));
    ui.set_series_list(series_model);
    
    let crud_model = slint::ModelRc::new(slint::VecModel::from(crud_items));
    ui.set_series_list_items(crud_model);
}

/// Set files list with both FileItem and CrudListItem formats
fn set_files_with_crud_items(file_items: Vec<crate::FileItem>, ui: &AppWindow) {
    // Create CrudListItem format
    let crud_items: Vec<crate::CrudListItem> = file_items.iter()
        .map(|f| crate::CrudListItem {
            title: f.name.clone(),
            subtitle: f.file_no.clone(),
        })
        .collect();
    
    let files_model = slint::ModelRc::new(slint::VecModel::from(file_items));
    ui.set_files(files_model);
    
    let crud_model = slint::ModelRc::new(slint::VecModel::from(crud_items));
    ui.set_files_list_items(crud_model);
}

/// Set items list with both ItemItem and CrudListItem formats
fn set_items_with_crud_items(item_items: Vec<crate::ItemItem>, ui: &AppWindow) {
    // Create CrudListItem format
    let crud_items: Vec<crate::CrudListItem> = item_items.iter()
        .map(|i| crate::CrudListItem {
            title: i.name.clone(),
            subtitle: i.item_no.clone(),
        })
        .collect();
    
    let items_model = slint::ModelRc::new(slint::VecModel::from(item_items));
    ui.set_file_item_list(items_model);
    
    let crud_model = slint::ModelRc::new(slint::VecModel::from(crud_items));
    ui.set_items_list_items(crud_model);
}

/// Clear series list (both formats)
fn clear_series(ui: &AppWindow) {
    let empty_series: Vec<crate::SeriesItem> = Vec::new();
    let empty_crud: Vec<crate::CrudListItem> = Vec::new();
    ui.set_series_list(slint::ModelRc::new(slint::VecModel::from(empty_series)));
    ui.set_series_list_items(slint::ModelRc::new(slint::VecModel::from(empty_crud)));
}

/// Clear files list (both formats)
fn clear_files(ui: &AppWindow) {
    let empty_files: Vec<crate::FileItem> = Vec::new();
    let empty_crud: Vec<crate::CrudListItem> = Vec::new();
    ui.set_files(slint::ModelRc::new(slint::VecModel::from(empty_files)));
    ui.set_files_list_items(slint::ModelRc::new(slint::VecModel::from(empty_crud)));
}

/// Clear items list (both formats)
fn clear_items(ui: &AppWindow) {
    let empty_items: Vec<crate::ItemItem> = Vec::new();
    let empty_crud: Vec<crate::CrudListItem> = Vec::new();
    ui.set_file_item_list(slint::ModelRc::new(slint::VecModel::from(empty_items)));
    ui.set_items_list_items(slint::ModelRc::new(slint::VecModel::from(empty_crud)));
}

/// Schema handler - manages schema-related UI events
pub struct SchemaHandler<CR: ConfigRepository + 'static> {
    archive_service: Rc<ArchiveService<CR>>,
}

impl<CR: ConfigRepository + 'static> SchemaHandler<CR> {
    pub fn new(archive_service: Rc<ArchiveService<CR>>) -> Self {
        Self { archive_service }
    }
    
    /// Setup schema callbacks for the UI
    pub fn setup_callbacks(&self, ui: &AppWindow) {
        self.setup_toast(ui);
        self.setup_add_schema(ui);
        self.setup_delete_schema(ui);
        self.setup_delete_selected_schemas(ui);
        self.setup_schema_selected(ui);
        self.setup_schema_activated(ui);
        self.setup_schema_item_clicked(ui);
        self.setup_schema_item_activated(ui);
        self.setup_add_schema_item(ui);
        self.setup_delete_schema_item(ui);
        self.setup_delete_selected_items(ui);
        self.setup_schema_item_item_clicked(ui);
    }
    
    /// Setup toast notification handler
    fn setup_toast(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_show_toast(move |message: slint::SharedString| {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_toast_message(message.clone());
                ui.set_toast_visible(true);
                
                // Auto-hide after 3 seconds
                let ui_weak2 = ui.as_weak();
                slint::Timer::single_shot(std::time::Duration::from_secs(3), move || {
                    if let Some(ui) = ui_weak2.upgrade() {
                        ui.set_toast_visible(false);
                    }
                });
            }
        });
    }
    
    fn setup_add_schema(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_add_schema(move || {
            // Create and show the schema input dialog as a separate window
            match crate::SchemaInputDialog::new() {
                    Ok(dialog) => {
                        // Set the dialog language to match the main window
                        dialog.set_current_language(0);
                        
                        let archive_service = archive_service.clone();
                        let ui_weak2 = ui_weak.clone();
                        let dialog_weak = dialog.as_weak();
                        
                        dialog.on_confirm_input(move |schema_code: slint::SharedString, schema_name: slint::SharedString| {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let schema_code_str = schema_code.to_string();
                                let schema_name_str = schema_name.to_string();
                                
                                if schema_code_str.is_empty() || schema_name_str.is_empty() {
                                    eprintln!("Schema code and name cannot be empty");
                                    return;
                                }
                                
                                let selected_index = ui.get_selected_archive();
                                
                                match archive_service.get_database_path_by_index(selected_index) {
                                    Ok(Some(db_path)) => {
                                        if !db_path.exists() {
                                            eprintln!("Database not found for selected archive");
                                            return;
                                        }
                                        
                                        match SchemaService::create_schema(&db_path, schema_code_str.clone(), schema_name_str) {
                                            Ok(_) => {
                                                println!("Schema created: {}", schema_code_str);
                                                Self::reload_schemas(&db_path, &ui);
                                                // Hide the dialog
                                                if let Some(d) = dialog_weak.upgrade() {
                                                    let _ = d.hide();
                                                }
                                            }
                                            Err(e) => {
                                                let error_msg = if e.to_string().contains("UNIQUE constraint failed") {
                                                    ui.get_code_exists().replace("{}", &schema_code_str)
                                                } else {
                                                    ui.get_create_failed().replace("{}", &e.to_string())
                                                };
                                                ui.invoke_show_toast(error_msg.into());
                                            }
                                        }
                                    }
                                    Ok(None) => eprintln!("No archive selected"),
                                    Err(e) => eprintln!("Failed to get database path: {}", e),
                                }
                            }
                        });
                        
                        let dialog_weak2 = dialog.as_weak();
                        dialog.on_cancel_input(move || {
                            println!("Schema dialog cancelled");
                            if let Some(d) = dialog_weak2.upgrade() {
                                let _ = d.hide();
                            }
                        });
                        
                        // Show the dialog as a separate window (non-blocking)
                        if let Err(e) = dialog.show() {
                            eprintln!("Failed to show schema dialog: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to create schema dialog: {}", e),
                }
        });
    }
    
    fn setup_delete_schema(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_delete_schema(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_index = ui.get_selected_archive();
                let schema_display = ui.invoke_get_current_schema_name().to_string();
                
                if schema_display.is_empty() {
                    eprintln!("No schema selected");
                    return;
                }
                
                // Extract schema_no from display format "{code} - {name}"
                let schema_no = schema_display
                    .split(" - ")
                    .next()
                    .unwrap_or(&schema_display)
                    .to_string();
                
                let confirmed = rfd::MessageDialog::new()
                    .set_title(ui.get_confirm_delete())
                    .set_description(ui.get_confirm_delete_classification().replace("{}", &schema_no))
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show();
                
                if confirmed != rfd::MessageDialogResult::Yes {
                    return;
                }
                
                match archive_service.get_database_path_by_index(selected_index) {
                    Ok(Some(db_path)) => {
                        if !db_path.exists() {
                            eprintln!("Database not found");
                            return;
                        }
                        
                        match SchemaService::delete_schema(&db_path, schema_no.clone()) {
                            Ok(true) => {
                                println!("Schema deleted: {}", schema_no);
                                Self::reload_schemas(&db_path, &ui);
                            }
                            Ok(false) => {
                                ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &schema_no).into());
                            }
                            Err(e) => {
                                ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &e.to_string()).into());
                            }
                        }
                    }
                    Ok(None) => eprintln!("No archive selected"),
                    Err(e) => eprintln!("Failed to get database path: {}", e),
                }
            }
        });
    }
    
    fn setup_delete_selected_schemas(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_delete_selected_schemas(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_index = ui.get_selected_archive();
                let selected_schemas = ui.get_selected_schemas();
                
                if selected_schemas.row_count() == 0 {
                    eprintln!("No schemas selected");
                    return;
                }
                
                // Count selected items
                let mut count = 0;
                for i in 0..selected_schemas.row_count() {
                    if selected_schemas.row_data(i) == Some(1) {
                        count += 1;
                    }
                }
                
                // Show confirmation dialog
                let confirmed = rfd::MessageDialog::new()
                    .set_title(ui.get_confirm_delete())
                    .set_description(ui.get_confirm_delete_selected_classifications().replace("{}", &count.to_string()))
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show();
                
                if confirmed != rfd::MessageDialogResult::Yes {
                    return;
                }
                
                match archive_service.get_database_path_by_index(selected_index) {
                    Ok(Some(db_path)) => {
                        if !db_path.exists() {
                            eprintln!("Database not found");
                            return;
                        }
                        
                        let schema_items = ui.get_schema_items();
                        let mut deleted_count = 0;
                        
                        // Iterate through selected indices
                        for i in 0..selected_schemas.row_count() {
                            if selected_schemas.row_data(i) == Some(1) {
                                if let Some(schema_item) = schema_items.row_data(i) {
                                    let schema_no = schema_item.code.to_string();
                                    match SchemaService::delete_schema(&db_path, schema_no.clone()) {
                                        Ok(true) => {
                                            println!("Schema deleted: {}", schema_no);
                                            deleted_count += 1;
                                        }
                                        Ok(false) => {
                                            ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &schema_no).into());
                                        }
                                        Err(e) => {
                                            ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &e.to_string()).into());
                                        }
                                    }
                                }
                            }
                        }
                        
                        if deleted_count > 0 {
                            Self::reload_schemas(&db_path, &ui);
                            ui.set_selected_schemas(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        }
                    }
                    Ok(None) => eprintln!("No archive selected"),
                    Err(e) => eprintln!("Failed to get database path: {}", e),
                }
            }
        });
    }
    
    fn setup_schema_selected(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_schema_selected(move |_index: i32| {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_index = ui.get_selected_archive();
                let schema_display = ui.invoke_get_current_schema_name().to_string();
                
                if schema_display.is_empty() {
                    return;
                }
                
                // Extract schema_no from display format "{code} - {name}"
                let schema_no = schema_display
                    .split(" - ")
                    .next()
                    .unwrap_or(&schema_display)
                    .to_string();
                
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_index) {
                    if db_path.exists() {
                        Self::reload_schema_items(&db_path, &schema_no, &ui);
                    }
                }
            }
        });
    }
    
    /// Setup schema activated callback - triggered when a schema is activated via CrudList
    fn setup_schema_activated(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_schema_activated(move |index: i32| {
            if let Some(ui) = ui_weak.upgrade() {
                let schema_items = ui.get_schema_items();
                
                if index < 0 || index >= schema_items.row_count() as i32 {
                    return;
                }
                
                if let Some(schema_item) = schema_items.row_data(index as usize) {
                    let schema_no = schema_item.code.to_string();
                    let selected_index = ui.get_selected_archive();
                    
                    // Clear multi-selection when activating
                    ui.set_selected_schemas(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                    
                    if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_index) {
                        if db_path.exists() {
                            Self::reload_schema_items(&db_path, &schema_no, &ui);
                        }
                    }
                }
            }
        });
    }
    
    /// Setup schema item activated callback - triggered when a schema item is activated via CrudList
    fn setup_schema_item_activated(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_schema_item_activated(move |_index: i32| {
            if let Some(_ui) = ui_weak.upgrade() {
                // Clear multi-selection when activating a single item
                _ui.set_selected_items(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                // Currently no additional action needed when a schema item is activated
                // This is a placeholder for future functionality
            }
        });
    }
    
    fn setup_schema_item_clicked(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_schema_item_clicked(move |index: i32, ctrl: bool, shift: bool| {
            if let Some(ui) = ui_weak.upgrade() {
                let schema_items = ui.get_schema_items();
                let total_count = schema_items.row_count() as i32;
                
                if index < 0 || index >= total_count {
                    return;
                }
                
                if ctrl {
                    // Ctrl+Click: Toggle selection
                    let mut selections = vec![0; total_count as usize];
                    let current_selections = ui.get_selected_schemas();
                    
                    // Copy current selections
                    for i in 0..current_selections.row_count() {
                        if i < selections.len() {
                            selections[i] = current_selections.row_data(i).unwrap_or(0);
                        }
                    }
                    
                    // Toggle current item
                    selections[index as usize] = if selections[index as usize] == 1 { 0 } else { 1 };
                    
                    let model = slint::ModelRc::new(slint::VecModel::from(selections));
                    ui.set_selected_schemas(model);
                    ui.set_selected_schema(index);
                } else if shift {
                    // Shift+Click: Range selection
                    let last_index = ui.get_selected_schema();
                    let start = last_index.min(index);
                    let end = last_index.max(index);
                    
                    let mut selections = vec![0; total_count as usize];
                    for i in start..=end {
                        selections[i as usize] = 1;
                    }
                    
                    let model = slint::ModelRc::new(slint::VecModel::from(selections));
                    ui.set_selected_schemas(model);
                    ui.set_selected_schema(index);
                } else {
                    // Normal click: Clear multi-selection and select single item
                    ui.set_selected_schema(index);
                    ui.set_selected_schemas(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                    
                    // Load schema items for the selected schema
                    if let Some(schema_item) = schema_items.row_data(index as usize) {
                        let schema_no = schema_item.code.to_string();
                        let selected_archive = ui.get_selected_archive();
                        
                        if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_archive) {
                            if db_path.exists() {
                                Self::reload_schema_items(&db_path, &schema_no, &ui);
                            }
                        }
                    }
                }
            }
        });
    }
    
    fn setup_add_schema_item(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_add_schema_item(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let schema_display = ui.invoke_get_current_schema_name().to_string();
                
                if schema_display.is_empty() {
                    eprintln!("No schema selected");
                    return;
                }
                
                // Extract schema_no from display format "{code} - {name}"
                let schema_no = schema_display
                    .split(" - ")
                    .next()
                    .unwrap_or(&schema_display)
                    .to_string();
                
                // Create and show the schema item input dialog
                match crate::SchemaItemInputDialog::new() {
                    Ok(dialog) => {
                        // Set the dialog language to match the main window
                        dialog.set_current_language(0);
                        
                        let archive_service = archive_service.clone();
                        let ui_weak2 = ui_weak.clone();
                        let schema_no_clone = schema_no.clone();
                        let dialog_weak = dialog.as_weak();
                        
                        dialog.on_confirm_input(move |item_code: slint::SharedString, item_name: slint::SharedString| {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let item_code_str = item_code.to_string();
                                let item_name_str = item_name.to_string();
                                
                                if item_code_str.is_empty() || item_name_str.is_empty() {
                                    eprintln!("Item code and name cannot be empty");
                                    return;
                                }
                                
                                let selected_index = ui.get_selected_archive();
                                
                                match archive_service.get_database_path_by_index(selected_index) {
                                    Ok(Some(db_path)) => {
                                        if !db_path.exists() {
                                            eprintln!("Database not found");
                                            return;
                                        }
                                        
                                        match SchemaService::add_schema_item(
                                            &db_path,
                                            schema_no_clone.clone(),
                                            item_code_str.clone(),
                                            item_name_str,
                                        ) {
                                            Ok(_) => {
                                                println!("Schema item added: {}", item_code_str);
                                                Self::reload_schema_items(&db_path, &schema_no_clone, &ui);
                                                // Hide the dialog
                                                if let Some(d) = dialog_weak.upgrade() {
                                                    let _ = d.hide();
                                                }
                                            }
                                            Err(e) => {
                                                let error_msg = if e.to_string().contains("UNIQUE constraint failed") {
                                                    ui.get_code_exists().replace("{}", &item_code_str)
                                                } else {
                                                    ui.get_create_failed().replace("{}", &e.to_string())
                                                };
                                                ui.invoke_show_toast(error_msg.into());
                                            }
                                        }
                                    }
                                    Ok(None) => eprintln!("No archive selected"),
                                    Err(e) => eprintln!("Failed to get database path: {}", e),
                                }
                            }
                        });
                        
                        let dialog_weak2 = dialog.as_weak();
                        dialog.on_cancel_input(move || {
                            println!("Schema item dialog cancelled");
                            if let Some(d) = dialog_weak2.upgrade() {
                                let _ = d.hide();
                            }
                        });
                        
                        // Show the dialog as a separate window (non-blocking)
                        if let Err(e) = dialog.show() {
                            eprintln!("Failed to show schema item dialog: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to create schema item dialog: {}", e),
                }
            }
        });
    }
    
    fn setup_delete_schema_item(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_delete_schema_item(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_index = ui.get_selected_archive();
                let schema_display = ui.invoke_get_current_schema_name().to_string();
                let item_display = ui.invoke_get_current_item_name().to_string();
                
                if schema_display.is_empty() {
                    eprintln!("No schema selected");
                    return;
                }
                
                if item_display.is_empty() {
                    eprintln!("No schema item selected");
                    return;
                }
                
                // Extract schema_no from display format "{code} - {name}"
                let schema_no = schema_display
                    .split(" - ")
                    .next()
                    .unwrap_or(&schema_display)
                    .to_string();
                
                // Extract item_no from display format "{code}: {name}"
                let item_no = item_display
                    .split(": ")
                    .next()
                    .unwrap_or(&item_display)
                    .to_string();
                
                // Show confirmation dialog
                let confirmed = rfd::MessageDialog::new()
                    .set_title(ui.get_confirm_delete())
                    .set_description(ui.get_confirm_delete_classification().replace("{}", &item_no))
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show();
                
                if confirmed != rfd::MessageDialogResult::Yes {
                    return;
                }
                
                match archive_service.get_database_path_by_index(selected_index) {
                    Ok(Some(db_path)) => {
                        if !db_path.exists() {
                            eprintln!("Database not found");
                            return;
                        }
                        
                        match SchemaService::delete_schema_item(&db_path, schema_no.clone(), item_no.clone()) {
                            Ok(_) => {
                                println!("Schema item deleted: {}", item_no);
                                Self::reload_schema_items(&db_path, &schema_no, &ui);
                            }
                            Err(e) => eprintln!("Failed to delete schema item: {}", e),
                        }
                    }
                    Ok(None) => eprintln!("No archive selected"),
                    Err(e) => eprintln!("Failed to get database path: {}", e),
                }
            }
        });
    }
    
    fn setup_delete_selected_items(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_delete_selected_items(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_archive = ui.get_selected_archive();
                let schema_display = ui.invoke_get_current_schema_name().to_string();
                
                if schema_display.is_empty() {
                    eprintln!("No schema selected");
                    return;
                }
                
                // Extract schema_no from display format "{code} - {name}"
                let schema_no = schema_display
                    .split(" - ")
                    .next()
                    .unwrap_or(&schema_display)
                    .to_string();
                
                match archive_service.get_database_path_by_index(selected_archive) {
                    Ok(Some(db_path)) => {
                        if !db_path.exists() {
                            eprintln!("Database not found");
                            return;
                        }
                        
                        let selected_items = ui.get_selected_items();
                        let item_details = ui.get_item_details();
                        
                        // Count selected items
                        let mut count = 0;
                        for i in 0..selected_items.row_count() {
                            if selected_items.row_data(i).unwrap_or(0) == 1 {
                                count += 1;
                            }
                        }
                        
                        // Show confirmation dialog
                        let confirmed = rfd::MessageDialog::new()
                            .set_title(ui.get_confirm_delete())
                            .set_description(ui.get_confirm_delete_selected_classifications().replace("{}", &count.to_string()))
                            .set_buttons(rfd::MessageButtons::YesNo)
                            .show();
                        
                        if confirmed != rfd::MessageDialogResult::Yes {
                            return;
                        }
                    
                    let mut deleted_count = 0;                        for i in 0..selected_items.row_count() {
                            if selected_items.row_data(i).unwrap_or(0) == 1 {
                                // Get the item at this index
                                if let Some(item) = item_details.row_data(i) {
                                    let item_no = item.code.to_string();
                                    
                                    match SchemaService::delete_schema_item(&db_path, schema_no.clone(), item_no.clone()) {
                                        Ok(_) => {
                                            println!("Schema item deleted: {}", item_no);
                                            deleted_count += 1;
                                        }
                                        Err(e) => eprintln!("Failed to delete schema item {}: {}", item_no, e),
                                    }
                                }
                            }
                        }
                        
                        if deleted_count > 0 {
                            Self::reload_schema_items(&db_path, &schema_no, &ui);
                            ui.set_selected_items(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        }
                    }
                    Ok(None) => eprintln!("No archive selected"),
                    Err(e) => eprintln!("Failed to get database path: {}", e),
                }
            }
        });
    }
    
    fn setup_schema_item_item_clicked(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_schema_item_item_clicked(move |index: i32, ctrl: bool, shift: bool| {
            if let Some(ui) = ui_weak.upgrade() {
                let item_details = ui.get_item_details();
                let total_count = item_details.row_count() as i32;
                
                if ctrl {
                    // Ctrl+Click: Toggle selection
                    let selected_items = ui.get_selected_items();
                    let mut selections = vec![0; total_count as usize];
                    
                    // Copy current selections
                    for i in 0..selected_items.row_count() {
                        if let Some(val) = selected_items.row_data(i) {
                            if i < selections.len() {
                                selections[i] = val;
                            }
                        }
                    }
                    
                    // Toggle current item
                    selections[index as usize] = if selections[index as usize] == 1 { 0 } else { 1 };
                    
                    let model = slint::ModelRc::new(slint::VecModel::from(selections));
                    ui.set_selected_items(model);
                    ui.set_selected_item(index);
                } else if shift {
                    // Shift+Click: Range selection
                    let last_index = ui.get_selected_item();
                    let start = last_index.min(index);
                    let end = last_index.max(index);
                    
                    let mut selections = vec![0; total_count as usize];
                    for i in start..=end {
                        selections[i as usize] = 1;
                    }
                    
                    let model = slint::ModelRc::new(slint::VecModel::from(selections));
                    ui.set_selected_items(model);
                    ui.set_selected_item(index);
                } else {
                    // Normal click: Clear multi-selection and select single item
                    ui.set_selected_item(index);
                    ui.set_selected_items(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                }
            }
        });
    }
    
    /// Helper to reload schemas into UI
    fn reload_schemas(db_path: &std::path::PathBuf, ui: &AppWindow) {
        match SchemaService::list_schemas(db_path) {
            Ok(schemas) => {
                // Load names for backward compatibility
                let schema_names: Vec<slint::SharedString> = schemas
                    .iter()
                    .map(|s| format!("{} - {}", s.schema_no, s.name).into())
                    .collect();
                let model = std::rc::Rc::new(slint::VecModel::from(schema_names));
                ui.invoke_load_schemas(slint::ModelRc::from(model));
                
                // Load structured data for new UI
                let schema_items: Vec<crate::SchemaItem> = schemas
                    .iter()
                    .map(|s| crate::SchemaItem {
                        code: s.schema_no.as_str().into(),
                        name: s.name.as_str().into(),
                    })
                    .collect();
                let items_model = slint::ModelRc::new(slint::VecModel::from(schema_items));
                ui.set_schema_items(items_model);
                
                // Load CrudListItem format for CrudList component
                let schema_list_items: Vec<crate::CrudListItem> = schemas
                    .iter()
                    .map(|s| crate::CrudListItem {
                        title: s.schema_no.as_str().into(),
                        subtitle: s.name.as_str().into(),
                    })
                    .collect();
                let list_model = slint::ModelRc::new(slint::VecModel::from(schema_list_items));
                ui.set_schema_list_items(list_model);
                
                // Load items for first schema if available
                if let Some(first) = schemas.first() {
                    Self::reload_schema_items(db_path, &first.schema_no, ui);
                }
            }
            Err(e) => eprintln!("Failed to load schemas: {}", e),
        }
    }
    
    /// Helper to reload schema items into UI
    fn reload_schema_items(db_path: &std::path::PathBuf, schema_no: &str, ui: &AppWindow) {
        match SchemaService::list_schema_items(db_path, schema_no) {
            Ok(items) => {
                // Load names for backward compatibility
                let item_names: Vec<slint::SharedString> = items
                    .iter()
                    .map(|i| format!("{}: {}", i.item_no, i.item_name).into())
                    .collect();
                let model = std::rc::Rc::new(slint::VecModel::from(item_names));
                ui.invoke_load_schema_items(slint::ModelRc::from(model));
                
                // Load structured data for new UI
                let item_details: Vec<crate::SchemaItemDetail> = items
                    .iter()
                    .map(|i| crate::SchemaItemDetail {
                        code: i.item_no.as_str().into(),
                        name: i.item_name.as_str().into(),
                    })
                    .collect();
                let details_model = slint::ModelRc::new(slint::VecModel::from(item_details));
                ui.set_item_details(details_model);
                
                // Load CrudListItem format for CrudList component
                let detail_list_items: Vec<crate::CrudListItem> = items
                    .iter()
                    .map(|i| crate::CrudListItem {
                        title: i.item_no.as_str().into(),
                        subtitle: i.item_name.as_str().into(),
                    })
                    .collect();
                let list_model = slint::ModelRc::new(slint::VecModel::from(detail_list_items));
                ui.set_detail_list_items(list_model);
            }
            Err(e) => eprintln!("Failed to load schema items: {}", e),
        }
    }
    
    /// Load initial schemas for an archive
    pub fn load_initial_schemas(db_path: &std::path::PathBuf, ui: &AppWindow) {
        if db_path.exists() {
            Self::reload_schemas(db_path, ui);
            // Reset selection states when loading initial data
            ui.set_selected_schema(0);
            ui.set_selected_item(0);
            ui.set_selected_schemas(slint::ModelRc::new(slint::VecModel::from(Vec::<i32>::new())));
            ui.set_selected_items(slint::ModelRc::new(slint::VecModel::from(Vec::<i32>::new())));
        }
    }
}

/// Archive handler - manages archive-related UI events
pub struct ArchiveHandler<CR: ConfigRepository + 'static> {
    archive_service: Rc<ArchiveService<CR>>,
}

impl<CR: ConfigRepository + 'static> ArchiveHandler<CR> {
    pub fn new(archive_service: Rc<ArchiveService<CR>>) -> Self {
        Self { archive_service }
    }
    
    /// Setup archive callbacks for the UI
    pub fn setup_callbacks(&self, ui: &AppWindow) {
        self.setup_add_archive(ui);
        self.setup_remove_archive(ui);
        self.setup_rename_archive(ui);
    }
    
    fn setup_add_archive(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_add_archive_library(move || {
            if let Some(ui) = ui_weak.upgrade() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let path_str = path.to_string_lossy().to_string();
                    let name = path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    
                    match archive_service.add_library(name, path_str) {
                        Ok(_) => {
                            println!("Archive library added");
                            Self::reload_archives(&archive_service, &ui);
                            
                            // Load schemas for the new library
                            let db_path = archive_service.get_database_path(&path.to_string_lossy());
                            SchemaHandler::<CR>::load_initial_schemas(&db_path, &ui);
                        }
                        Err(e) => eprintln!("Failed to add archive library: {}", e),
                    }
                }
            }
        });
    }
    
    fn setup_remove_archive(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_remove_archive_library(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_index = ui.get_selected_archive() as usize;
                
                match archive_service.list_libraries() {
                    Ok(libraries) => {
                        if selected_index < libraries.len() {
                            let library_name = &libraries[selected_index].name;
                            
                            // Show confirmation dialog
                            let confirmed = rfd::MessageDialog::new()
                                .set_title(ui.get_confirm_delete())
                                .set_description(ui.get_confirm_delete_classification().replace("{}", library_name))
                                .set_buttons(rfd::MessageButtons::YesNo)
                                .show();
                            
                            if confirmed != rfd::MessageDialogResult::Yes {
                                return;
                            }
                            
                            let path = libraries[selected_index].path.clone();
                            
                            if let Err(e) = archive_service.remove_library(&path) {
                                eprintln!("Failed to remove library: {}", e);
                                return;
                            }
                            
                            Self::reload_archives(&archive_service, &ui);
                        }
                    }
                    Err(e) => eprintln!("Failed to list libraries: {}", e),
                }
            }
        });
    }
    
    fn setup_rename_archive(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_rename_archive_library(move |index: i32, current_name: slint::SharedString| {
            // Create and show the rename dialog
            match crate::RenameArchiveDialog::new() {
                    Ok(dialog) => {
                        // Set the dialog language to match the main window
                        dialog.set_current_language(0);
                        dialog.set_library_name_input(current_name.clone());
                        
                        let archive_service = archive_service.clone();
                        let ui_weak2 = ui_weak.clone();
                        let dialog_weak = dialog.as_weak();
                        
                        dialog.on_confirm_input(move |new_name: slint::SharedString| {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let new_name_str = new_name.to_string();
                                
                                if new_name_str.is_empty() {
                                    eprintln!("Library name cannot be empty");
                                    return;
                                }
                                
                                if new_name_str != current_name.as_str() {
                                    match archive_service.rename_library(index as usize, new_name_str.clone()) {
                                        Ok(_) => {
                                            println!("Archive library renamed to: {}", new_name_str);
                                            Self::reload_archives(&archive_service, &ui);
                                            // Hide the dialog
                                            if let Some(d) = dialog_weak.upgrade() {
                                                let _ = d.hide();
                                            }
                                        }
                                        Err(e) => eprintln!("Failed to rename library: {}", e),
                                    }
                                } else {
                                    // Name unchanged, just close dialog
                                    if let Some(d) = dialog_weak.upgrade() {
                                        let _ = d.hide();
                                    }
                                }
                            }
                        });
                        
                        let dialog_weak2 = dialog.as_weak();
                        dialog.on_cancel_input(move || {
                            println!("Rename dialog cancelled");
                            if let Some(d) = dialog_weak2.upgrade() {
                                let _ = d.hide();
                            }
                        });
                        
                        // Show the dialog as a separate window (non-blocking)
                        if let Err(e) = dialog.show() {
                            eprintln!("Failed to show rename dialog: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to create rename dialog: {}", e),
                }
        });
    }
    
    /// Helper to reload archive list
    fn reload_archives(archive_service: &ArchiveService<CR>, ui: &AppWindow) {
        match archive_service.list_libraries() {
            Ok(libraries) => {
                // Load names for ComboBox compatibility
                let names: Vec<slint::SharedString> = libraries
                    .iter()
                    .map(|lib| lib.name.as_str().into())
                    .collect();
                let new_len = names.len();
                let model = slint::ModelRc::new(slint::VecModel::from(names));
                ui.set_archive_libraries(model);
                
                // Load structured data for settings display
                let items: Vec<crate::ArchiveLibraryItem> = libraries
                    .iter()
                    .map(|lib| crate::ArchiveLibraryItem {
                        name: lib.name.as_str().into(),
                        path: lib.path.as_str().into(),
                    })
                    .collect();
                let items_model = slint::ModelRc::new(slint::VecModel::from(items));
                ui.set_archive_library_items(items_model);
                
                // Load CrudList format data for settings page
                let crud_items: Vec<crate::CrudListItem> = libraries
                    .iter()
                    .map(|lib| crate::CrudListItem {
                        title: lib.name.as_str().into(),
                        subtitle: lib.path.as_str().into(),
                    })
                    .collect();
                let crud_model = slint::ModelRc::new(slint::VecModel::from(crud_items));
                ui.set_archive_library_crud_items(crud_model);
                
                // Adjust selection
                let current = ui.get_selected_archive() as usize;
                if current >= new_len && new_len > 0 {
                    ui.set_selected_archive((new_len - 1) as i32);
                } else if new_len == 0 {
                    ui.set_selected_archive(-1);
                }
            }
            Err(e) => eprintln!("Failed to reload archives: {}", e),
        }
    }
    
    /// Initialize archives in UI
    pub fn initialize(&self, ui: &AppWindow) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.archive_service.get_settings()?;
        
        // Load archive names for ComboBox
        let archive_names: Vec<slint::SharedString> = settings
            .archive_libraries
            .iter()
            .map(|lib| lib.name.as_str().into())
            .collect();
        let model = slint::ModelRc::new(slint::VecModel::from(archive_names));
        ui.set_archive_libraries(model);
        
        // Load structured data for settings display
        let items: Vec<crate::ArchiveLibraryItem> = settings
            .archive_libraries
            .iter()
            .map(|lib| crate::ArchiveLibraryItem {
                name: lib.name.as_str().into(),
                path: lib.path.as_str().into(),
            })
            .collect();
        let items_model = slint::ModelRc::new(slint::VecModel::from(items));
        ui.set_archive_library_items(items_model);
        
        // Load CrudList format data for settings page
        let crud_items: Vec<crate::CrudListItem> = settings
            .archive_libraries
            .iter()
            .map(|lib| crate::CrudListItem {
                title: lib.name.as_str().into(),
                subtitle: lib.path.as_str().into(),
            })
            .collect();
        let crud_model = slint::ModelRc::new(slint::VecModel::from(crud_items));
        ui.set_archive_library_crud_items(crud_model);
        
        // Set selected archive from last opened
        if let Some(index) = self.archive_service.get_last_opened_index()? {
            ui.set_selected_archive(index as i32);
        }
        
        // Language is now fixed to Chinese (0)
        
        Ok(())
    }
}

/// Settings handler - manages settings-related UI events
pub struct SettingsHandler<CR: ConfigRepository + 'static> {
    archive_service: Rc<ArchiveService<CR>>,
}

impl<CR: ConfigRepository + 'static> SettingsHandler<CR> {
    pub fn new(archive_service: Rc<ArchiveService<CR>>) -> Self {
        Self { archive_service }
    }
    
    /// Setup settings callbacks
    pub fn setup_callbacks(&self, ui: &AppWindow) {
        self.setup_apply_settings(ui);
        self.setup_cancel_settings(ui);
    }
    
    fn setup_apply_settings(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_apply_settings(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let language_index = ui.get_selected_language();
                let language = if language_index == 0 { "zh_CN" } else { "en_US" };
                if let Err(e) = archive_service.set_language(language.to_string()) {
                    eprintln!("Failed to save language: {}", e);
                } else {
                    // Apply translation immediately
                    let _ = slint::select_bundled_translation(language);
                    ui.invoke_show_toast(ui.get_language_applied());
                }
            }
        });
    }
    
    fn setup_cancel_settings(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let _archive_service = self.archive_service.clone();
        
        ui.on_cancel_settings(move || {
            if let Some(_ui) = ui_weak.upgrade() {
                // Language is fixed to Chinese, no need to reset
            }
        });
    }
}

/// Classification handler - manages classification-related UI events
pub struct ClassificationHandler<CR: ConfigRepository + 'static> {
    archive_service: Rc<ArchiveService<CR>>,
}

impl<CR: ConfigRepository + 'static> ClassificationHandler<CR> {
    pub fn new(archive_service: Rc<ArchiveService<CR>>) -> Self { Self { archive_service } }

    pub fn setup_callbacks(&self, ui: &AppWindow) {
        self.setup_add_classification(ui);
        self.setup_delete_classification(ui);
        self.setup_delete_selected_classifications(ui);
        self.setup_classification_item_clicked(ui);
        self.setup_classification_selected(ui);
        self.setup_add_child_classification(ui);
        self.setup_delete_child_classification(ui);
        self.setup_delete_selected_children(ui);
        self.setup_child_classification_clicked(ui);
        self.setup_activate_classification(ui);
        self.setup_deactivate_classification(ui);
        self.setup_activate_child_classification(ui);
        self.setup_deactivate_child_classification(ui);
        self.setup_export_classifications(ui);
        self.setup_import_classifications(ui);
    }

    fn setup_add_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_add_classification(move || {
            match crate::ClassificationInputDialog::new() {
                    Ok(dialog) => {
                        dialog.set_current_language(0);
                        let archive_service = archive_service.clone(); let ui_weak2 = ui_weak.clone(); let dialog_weak = dialog.as_weak();
                        dialog.on_confirm_input(move |code: slint::SharedString, name: slint::SharedString| {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let code_s = code.to_string(); let name_s = name.to_string(); if code_s.is_empty() || name_s.is_empty() { return; }
                                let sel_index = ui.get_selected_archive();
                                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_index) {
                                    match ClassificationService::create_top(&db_path, code_s.clone(), name_s) {
                                        Ok(_) => { Self::reload_top(&db_path, &ui); if let Some(d) = dialog_weak.upgrade() { let _ = d.hide(); } }
                                        Err(e) => { let msg = if e.to_string().contains("UNIQUE constraint failed") { ui.get_code_exists().replace("{}", &code_s) } else { ui.get_create_failed().replace("{}", &e.to_string()) }; ui.invoke_show_toast(msg.into()); }
                                    }
                                }
                            }
                        });
                        let dialog_weak2 = dialog.as_weak(); dialog.on_cancel_input(move || { if let Some(d) = dialog_weak2.upgrade() { let _ = d.hide(); } }); let _ = dialog.show();
                    }
                    Err(e) => eprintln!("Failed to create classification dialog: {}", e),
                }
        });
    }

    fn setup_delete_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_delete_classification(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive();
                let current_index = ui.get_selected_classification();
                let list = ui.get_classification_items();
                if current_index < 0 || current_index as usize >= list.row_count() { return; }
                if let Some(item) = list.row_data(current_index as usize) {
                    let code = item.code.to_string();
                    let confirmed = rfd::MessageDialog::new().set_title(ui.get_confirm_delete()).set_description(ui.get_confirm_delete_classification().replace("{}", &code)).set_buttons(rfd::MessageButtons::YesNo).show();
                    if confirmed != rfd::MessageDialogResult::Yes { return; }
                    if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_index) {
                        match ClassificationService::delete(&db_path, &code) {
                            Ok(true) => { Self::reload_top(&db_path, &ui); }
                            Ok(false) => ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &code).into()),
                            Err(e) => ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &e.to_string()).into()),
                        }
                    }
                }
            }
        });
    }

    fn setup_delete_selected_classifications(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_delete_selected_classifications(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive();
                let selections = ui.get_selected_classifications();
                let items = ui.get_classification_items(); if selections.row_count() == 0 { return; }
                let mut count = 0; for i in 0..selections.row_count() { if selections.row_data(i) == Some(1) { count += 1; } }
                let confirmed = rfd::MessageDialog::new().set_title(ui.get_confirm_delete()).set_description(ui.get_confirm_delete_selected_classifications().replace("{}", &count.to_string())).set_buttons(rfd::MessageButtons::YesNo).show(); if confirmed != rfd::MessageDialogResult::Yes { return; }
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_index) {
                    let mut deleted = 0;
                    for i in 0..selections.row_count() { if selections.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); match ClassificationService::delete(&db_path, &code) { Ok(true) => deleted += 1, Ok(false) => ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &code).into()), Err(e) => ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &e.to_string()).into()) } } } }
                    if deleted > 0 { Self::reload_top(&db_path, &ui); ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::<i32>::default())); }
                }
            }
        });
    }

    fn setup_classification_item_clicked(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_classification_item_clicked(move |index: i32, ctrl: bool, shift: bool| {
            if let Some(ui) = ui_weak.upgrade() {
                let items_model = ui.get_classification_items(); let total = items_model.row_count() as i32; if index < 0 || index >= total { return; }
                if ctrl { let mut sel = vec![0; total as usize]; let current = ui.get_selected_classifications(); for i in 0..current.row_count() { sel[i] = current.row_data(i).unwrap_or(0); } sel[index as usize] = if sel[index as usize] == 1 {0} else {1}; ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::from(sel))); ui.set_selected_classification(index); }
                else if shift { let last = ui.get_selected_classification(); let start = last.min(index); let end = last.max(index); let mut sel = vec![0; total as usize]; for i in start..=end { sel[i as usize] = 1; } ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::from(sel))); ui.set_selected_classification(index); }
                else { ui.set_selected_classification(index); ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::<i32>::default())); }
                if let Some(item) = items_model.row_data(index as usize) { let code = item.code.to_string(); let sel_arch = ui.get_selected_archive(); if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_arch) { if db_path.exists() { Self::reload_children(&db_path, &code, &ui); } } }
            }
        });
    }

    fn setup_classification_selected(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_classification_selected(move |_i: i32| {
            if let Some(ui) = ui_weak.upgrade() { let idx = ui.get_selected_classification(); let items = ui.get_classification_items(); if idx >=0 && (idx as usize) < items.row_count() { if let Some(item) = items.row_data(idx as usize) { let code = item.code.to_string(); let sel_arch = ui.get_selected_archive(); if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_arch) { if db_path.exists() { Self::reload_children(&db_path, &code, &ui); } } } } }
        });
    }

    fn setup_add_child_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_add_child_classification(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let parent_idx = ui.get_selected_classification(); let parents = ui.get_classification_items(); if parent_idx < 0 || parent_idx as usize >= parents.row_count() { return; }
                if let Some(parent) = parents.row_data(parent_idx as usize) {
                    let parent_code = parent.code.to_string(); let parent_display = format!("{} - {}", parent.code, parent.name);
                    match crate::ClassificationChildInputDialog::new() {
                        Ok(dialog) => {
                            dialog.set_current_language(0); dialog.set_parent_display(parent_display.into());
                            let archive_service = archive_service.clone(); let ui_weak2 = ui_weak.clone(); let dialog_weak = dialog.as_weak(); let parent_code_clone = parent_code.clone();
                            dialog.on_confirm_input(move |code: slint::SharedString, name: slint::SharedString| {
                                if let Some(ui) = ui_weak2.upgrade() { let code_s = code.to_string(); let name_s = name.to_string(); if code_s.is_empty() || name_s.is_empty() { return; }
                                    let sel_index = ui.get_selected_archive(); if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_index) {
                                        match ClassificationService::create_child(&db_path, parent_code_clone.clone(), code_s.clone(), name_s) {
                                            Ok(_) => { Self::reload_children(&db_path, &parent_code_clone, &ui); if let Some(d) = dialog_weak.upgrade() { let _ = d.hide(); } }
                                            Err(e) => { let msg = if e.to_string().contains("UNIQUE constraint failed") { ui.get_code_exists().replace("{}", &code_s) } else { ui.get_create_failed().replace("{}", &e.to_string()) }; ui.invoke_show_toast(msg.into()); }
                                        }
                                    }
                                }
                            });
                            let dialog_weak2 = dialog.as_weak(); dialog.on_cancel_input(move || { if let Some(d) = dialog_weak2.upgrade() { let _ = d.hide(); } }); let _ = dialog.show();
                        }
                        Err(e) => eprintln!("Failed to create child classification dialog: {}", e),
                    }
                }
            }
        });
    }

    fn setup_delete_child_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_delete_child_classification(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let parent_idx = ui.get_selected_classification(); let parents = ui.get_classification_items(); if parent_idx < 0 || parent_idx as usize >= parents.row_count() { return; }
                let children_model = ui.get_classification_children(); let child_idx = ui.get_selected_child(); if child_idx < 0 || child_idx as usize >= children_model.row_count() { return; }
                if let Some(child) = children_model.row_data(child_idx as usize) {
                    let code = child.code.to_string(); let parent_code = parents.row_data(parent_idx as usize).unwrap().code.to_string();
                    let confirmed = rfd::MessageDialog::new().set_title(ui.get_confirm_delete()).set_description(ui.get_confirm_delete_child_classification().replace("{}", &code)).set_buttons(rfd::MessageButtons::YesNo).show();
                    if confirmed != rfd::MessageDialogResult::Yes { return; }
                    let sel_arch = ui.get_selected_archive(); if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_arch) {
                        match ClassificationService::delete(&db_path, &code) { Ok(true) => { Self::reload_children(&db_path, &parent_code, &ui); } Ok(false) => ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &code).into()), Err(e) => ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &e.to_string()).into()) }
                    }
                }
            }
        });
    }

    fn setup_delete_selected_children(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_delete_selected_children(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let parent_idx = ui.get_selected_classification(); let parents = ui.get_classification_items(); if parent_idx <0 || parent_idx as usize >= parents.row_count() { return; }
                let selections = ui.get_selected_children(); let children = ui.get_classification_children(); if selections.row_count() == 0 { return; }
                let mut count = 0; for i in 0..selections.row_count() { if selections.row_data(i)==Some(1){count+=1;} }
                let confirmed = rfd::MessageDialog::new().set_title(ui.get_confirm_delete()).set_description(ui.get_confirm_delete_selected_children().replace("{}", &count.to_string())).set_buttons(rfd::MessageButtons::YesNo).show(); if confirmed != rfd::MessageDialogResult::Yes { return; }
                let sel_arch = ui.get_selected_archive(); let parent_code = parents.row_data(parent_idx as usize).unwrap().code.to_string();
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_arch) {
                    let mut deleted = 0; for i in 0..selections.row_count() { if selections.row_data(i)==Some(1) { if let Some(child) = children.row_data(i) { let code = child.code.to_string(); match ClassificationService::delete(&db_path, &code) { Ok(true)=>deleted+=1, Ok(false)=>ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &code).into()), Err(e)=>ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &format!("'{}': {}", code, e)).into()) } } } }
                    if deleted>0 { Self::reload_children(&db_path, &parent_code, &ui); ui.set_selected_children(slint::ModelRc::new(slint::VecModel::<i32>::default())); }
                }
            }
        });
    }

    fn setup_child_classification_clicked(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        ui.on_child_classification_clicked(move |index: i32, ctrl: bool, shift: bool| {
            if let Some(ui) = ui_weak.upgrade() {
                let model = ui.get_classification_children(); let total = model.row_count() as i32; if index < 0 || index >= total { return; }
                if ctrl { let current = ui.get_selected_children(); let mut sel = vec![0; total as usize]; for i in 0..current.row_count(){ sel[i]=current.row_data(i).unwrap_or(0);} sel[index as usize] = if sel[index as usize]==1 {0} else {1}; ui.set_selected_children(slint::ModelRc::new(slint::VecModel::from(sel))); ui.set_selected_child(index); }
                else if shift { let last = ui.get_selected_child(); let start = last.min(index); let end = last.max(index); let mut sel = vec![0; total as usize]; for i in start..=end { sel[i as usize]=1; } ui.set_selected_children(slint::ModelRc::new(slint::VecModel::from(sel))); ui.set_selected_child(index); }
                else { ui.set_selected_child(index); ui.set_selected_children(slint::ModelRc::new(slint::VecModel::<i32>::default())); }
            }
        });
    }

    fn setup_activate_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_activate_classification(move || {
            println!(">>> on_activate_classification callback ENTERED <<<");
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive();
                let current_index = ui.get_selected_classification();
                let multi_select_count = ui.get_selected_classifications().row_count();
                println!("Activate classification: selected_archive={}, selected_classification={}, multi_select_count={}", sel_index, current_index, multi_select_count);
                
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        // Check if there are actual selections in the multi-select array (value == 1)
                        let selected = ui.get_selected_classifications();
                        let mut has_multi_selection = false;
                        for i in 0..selected.row_count() {
                            if selected.row_data(i) == Some(1) {
                                has_multi_selection = true;
                                break;
                            }
                        }
                        
                        if has_multi_selection {
                            let items = ui.get_classification_items();
                            for i in 0..selected.row_count() { 
                                if selected.row_data(i) == Some(1) { 
                                    if let Some(item) = items.row_data(i) { 
                                        let code = item.code.to_string(); 
                                        println!("Activating (multi): {}", code);
                                        if let Err(e) = ClassificationService::activate(&db_path, &code) { 
                                            ui.invoke_show_toast(ui.get_activate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); 
                                        } 
                                    } 
                                } 
                            }
                            Self::reload_top(&db_path, &ui); 
                            ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let list = ui.get_classification_items();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { 
                                if let Some(item) = list.row_data(current_index as usize) { 
                                    let code = item.code.to_string(); 
                                    println!("Activating (single): {}", code);
                                    if let Err(e) = ClassificationService::activate(&db_path, &code) { 
                                        ui.invoke_show_toast(ui.get_activate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); 
                                    } else { 
                                        Self::reload_top(&db_path, &ui); 
                                    } 
                                } 
                            }
                        }
                    }
                    _ => { println!("No database path found"); }
                }
            }
        });
    }

    fn setup_deactivate_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_deactivate_classification(move || {
            println!(">>> on_deactivate_classification callback ENTERED <<<");
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive();
                let current_index = ui.get_selected_classification();
                println!("Deactivate classification: selected_archive={}, selected_classification={}", sel_index, current_index);
                
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        // Check if there are actual selections in the multi-select array (value == 1)
                        let selected = ui.get_selected_classifications();
                        let mut has_multi_selection = false;
                        for i in 0..selected.row_count() {
                            if selected.row_data(i) == Some(1) {
                                has_multi_selection = true;
                                break;
                            }
                        }
                        
                        if has_multi_selection {
                            let items = ui.get_classification_items();
                            for i in 0..selected.row_count() { 
                                if selected.row_data(i) == Some(1) { 
                                    if let Some(item) = items.row_data(i) { 
                                        let code = item.code.to_string(); 
                                        println!("Deactivating (multi): {}", code);
                                        if let Err(e) = ClassificationService::deactivate(&db_path, &code) { 
                                            ui.invoke_show_toast(ui.get_deactivate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); 
                                        } 
                                    } 
                                } 
                            }
                            Self::reload_top(&db_path, &ui); 
                            ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let list = ui.get_classification_items();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { 
                                if let Some(item) = list.row_data(current_index as usize) { 
                                    let code = item.code.to_string(); 
                                    println!("Deactivating (single): {}", code);
                                    if let Err(e) = ClassificationService::deactivate(&db_path, &code) { 
                                        ui.invoke_show_toast(ui.get_deactivate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); 
                                    } else { 
                                        Self::reload_top(&db_path, &ui); 
                                    } 
                                } 
                            }
                        }
                    }
                    _ => { println!("No database path found"); }
                }
            }
        });
    }

    fn setup_activate_child_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_activate_child_classification(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive(); let parent_idx = ui.get_selected_classification(); let parents = ui.get_classification_items(); if parent_idx < 0 || parent_idx as usize >= parents.row_count() { return; }
                let parent_code = parents.row_data(parent_idx as usize).unwrap().code.to_string();
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_index) {
                    if ui.get_selected_children().row_count() > 0 {
                        let selected = ui.get_selected_children(); let items = ui.get_classification_children();
                        for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(ui.get_activate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } } } }
                        Self::reload_children(&db_path, &parent_code, &ui); ui.set_selected_children(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                    } else {
                        let current_index = ui.get_selected_child(); let list = ui.get_classification_children();
                        if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(ui.get_activate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } else { Self::reload_children(&db_path, &parent_code, &ui); } } }
                    }
                }
            }
        });
    }

    fn setup_deactivate_child_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_deactivate_child_classification(move || {
            println!(">>> on_deactivate_child_classification callback ENTERED <<<");
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive(); let parent_idx = ui.get_selected_classification(); let parents = ui.get_classification_items(); if parent_idx < 0 || parent_idx as usize >= parents.row_count() { return; }
                let parent_code = parents.row_data(parent_idx as usize).unwrap().code.to_string();
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(sel_index) {
                    if ui.get_selected_children().row_count() > 0 {
                        let selected = ui.get_selected_children(); let items = ui.get_classification_children();
                        for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(ui.get_deactivate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } } } }
                        Self::reload_children(&db_path, &parent_code, &ui); ui.set_selected_children(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                    } else {
                        let current_index = ui.get_selected_child(); let list = ui.get_classification_children();
                        if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(ui.get_deactivate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } else { Self::reload_children(&db_path, &parent_code, &ui); } } }
                    }
                }
            }
        });
    }

    fn setup_export_classifications(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_export_classifications(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive();
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        match ClassificationService::export_to_json(&db_path) {
                            Ok(json) => {
                                if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).save_file() {
                                    if let Err(e) = std::fs::write(&path, json) {
                                        ui.invoke_show_toast(ui.get_export_failed().replace("{}", &e.to_string()).into());
                                    } else {
                                        ui.invoke_show_toast(ui.get_export_success());
                                    }
                                }
                            }
                            Err(e) => ui.invoke_show_toast(ui.get_export_failed().replace("{}", &e.to_string()).into()),
                        }
                    }
                    _ => ui.invoke_show_toast(ui.get_no_archive_selected()),
                }
            }
        });
    }

    fn setup_import_classifications(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_import_classifications(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive();
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
                            match std::fs::read_to_string(&path) {
                                Ok(json) => {
                                    match ClassificationService::import_from_json(&db_path, &json) {
                                        Ok(_) => {
                                            Self::reload_top(&db_path, &ui);
                                            ui.invoke_show_toast(ui.get_import_success());
                                        }
                                        Err(e) => ui.invoke_show_toast(ui.get_import_failed().replace("{}", &e.to_string()).into()),
                                    }
                                }
                                Err(e) => ui.invoke_show_toast(ui.get_read_file_failed().replace("{}", &e.to_string()).into()),
                            }
                        }
                    }
                    _ => ui.invoke_show_toast(ui.get_no_archive_selected()),
                }
            }
        });
    }

    fn reload_top(db_path: &std::path::PathBuf, ui: &AppWindow) {
        match ClassificationService::list_top(db_path) {
            Ok(list) => {
                let items: Vec<crate::ClassificationItem> = list.iter().map(|c| crate::ClassificationItem { code: c.code.as_str().into(), name: c.name.as_str().into(), is_active: c.is_active }).collect();
                let model = slint::ModelRc::new(slint::VecModel::from(items)); ui.set_classification_items(model);

                // Convert to CrudListItem format for CrudList component
                let crud_items: Vec<crate::CrudListItem> = list.iter().map(|c| crate::CrudListItem {
                    title: c.name.as_str().into(),
                    subtitle: c.code.as_str().into(),
                }).collect();
                let crud_model = slint::ModelRc::new(slint::VecModel::from(crud_items)); ui.set_classification_crud_items(crud_model);

                // Generate row styles based on is_active
                let row_styles: Vec<crate::CrudListRowStyle> = list.iter().map(|c| {
                    if c.is_active {
                        crate::CrudListRowStyle {
                            title_color: slint::Color::from_rgb_u8(0x1a, 0x1a, 0x1a),  // Theme.text_primary
                            subtitle_color: slint::Color::from_rgb_u8(0x66, 0x66, 0x66),  // Theme.text_secondary
                            background_color: slint::Color::from_rgb_u8(0xff, 0xff, 0xff),  // Theme.list_background
                            opacity: 1.0,
                        }
                    } else {
                        crate::CrudListRowStyle {
                            title_color: slint::Color::from_rgb_u8(0x99, 0x99, 0x99),  // Dimmed text
                            subtitle_color: slint::Color::from_rgb_u8(0xaa, 0xaa, 0xaa),  // Dimmed subtitle
                            background_color: slint::Color::from_rgb_u8(0xf5, 0xf5, 0xf5),  // Slightly gray background
                            opacity: 0.6,
                        }
                    }
                }).collect();
                let row_styles_model = slint::ModelRc::new(slint::VecModel::from(row_styles)); ui.set_classification_row_styles(row_styles_model);

                if let Some(first) = list.first() {
                    Self::reload_children(db_path, &first.code, ui);
                } else {
                    // If no top classifications, clear children as well
                    let empty_children: Vec<crate::ClassificationItem> = Vec::new();
                    let children_model = slint::ModelRc::new(slint::VecModel::from(empty_children));
                    ui.set_classification_children(children_model);

                    let empty_crud_children: Vec<crate::CrudListItem> = Vec::new();
                    let crud_children_model = slint::ModelRc::new(slint::VecModel::from(empty_crud_children));
                    ui.set_child_crud_items(crud_children_model);
                    
                    let empty_child_styles: Vec<crate::CrudListRowStyle> = Vec::new();
                    let child_styles_model = slint::ModelRc::new(slint::VecModel::from(empty_child_styles));
                    ui.set_child_row_styles(child_styles_model);
                }
            }
            Err(e) => eprintln!("Failed to load classifications: {}", e),
        }
    }

    fn reload_children(db_path: &std::path::PathBuf, parent_code: &str, ui: &AppWindow) {
        match ClassificationService::list_children(db_path, parent_code) {
            Ok(children) => {
                let items: Vec<crate::ClassificationItem> = children.iter().map(|c| crate::ClassificationItem { code: c.code.as_str().into(), name: c.name.as_str().into(), is_active: c.is_active }).collect();
                let model = slint::ModelRc::new(slint::VecModel::from(items)); ui.set_classification_children(model);

                // Convert to CrudListItem format for CrudList component
                let crud_items: Vec<crate::CrudListItem> = children.iter().map(|c| crate::CrudListItem {
                    title: c.name.as_str().into(),
                    subtitle: c.code.as_str().into(),
                }).collect();
                let crud_model = slint::ModelRc::new(slint::VecModel::from(crud_items)); ui.set_child_crud_items(crud_model);

                // Generate row styles based on is_active
                let row_styles: Vec<crate::CrudListRowStyle> = children.iter().map(|c| {
                    if c.is_active {
                        crate::CrudListRowStyle {
                            title_color: slint::Color::from_rgb_u8(0x1a, 0x1a, 0x1a),  // Theme.text_primary
                            subtitle_color: slint::Color::from_rgb_u8(0x66, 0x66, 0x66),  // Theme.text_secondary
                            background_color: slint::Color::from_rgb_u8(0xff, 0xff, 0xff),  // Theme.list_background
                            opacity: 1.0,
                        }
                    } else {
                        crate::CrudListRowStyle {
                            title_color: slint::Color::from_rgb_u8(0x99, 0x99, 0x99),  // Dimmed text
                            subtitle_color: slint::Color::from_rgb_u8(0xaa, 0xaa, 0xaa),  // Dimmed subtitle
                            background_color: slint::Color::from_rgb_u8(0xf5, 0xf5, 0xf5),  // Slightly gray background
                            opacity: 0.6,
                        }
                    }
                }).collect();
                let row_styles_model = slint::ModelRc::new(slint::VecModel::from(row_styles)); ui.set_child_row_styles(row_styles_model);
            }
            Err(e) => eprintln!("Failed to load child classifications: {}", e),
        }
    }

    pub fn load_initial_classifications(db_path: &std::path::PathBuf, ui: &AppWindow) {
        if db_path.exists() {
            Self::reload_top(db_path, ui);
            // Reset selection states when loading initial data
            ui.set_selected_classification(0);
            ui.set_selected_child(0);
            ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::from(Vec::<i32>::new())));
            ui.set_selected_children(slint::ModelRc::new(slint::VecModel::from(Vec::<i32>::new())));
        }
    }
}

/// Fonds handler - manages fonds-related UI events
#[derive(Clone)]
pub struct FondsHandler<CR: ConfigRepository + 'static> {
    archive_service: Rc<ArchiveService<CR>>,
    file_service: Rc<FileService>,
}

impl<CR: ConfigRepository + 'static> FondsHandler<CR> {
    pub fn new(archive_service: Rc<ArchiveService<CR>>, file_service: Rc<FileService>) -> Self {
        Self { 
            archive_service,
            file_service,
        }
    }
    
    /// Setup fonds callbacks for the UI
    pub fn setup_callbacks(&self, ui: &AppWindow) {
        self.setup_add_fonds_dialog(ui);
        self.setup_add_file(ui);
        self.setup_delete_file(ui);
        self.setup_rename_file(ui);
        self.setup_delete_selected_files(ui);
        self.setup_file_clicked(ui);
        self.setup_select_series(ui);
        self.setup_file_selected(ui);
        self.setup_add_item(ui);
        self.setup_delete_item(ui);
        self.setup_rename_item(ui);
        self.setup_delete_selected_items(ui);
        self.setup_item_clicked(ui);
        self.setup_open_file(ui);
        self.setup_open_item(ui);
        self.setup_fonds_selected(ui);
        self.setup_archive_selected(ui);
        // New CrudList-related callbacks
        self.setup_file_activated(ui);
        self.setup_item_activated(ui);
        self.setup_series_activated(ui);
        self.setup_open_file_at(ui);
        self.setup_open_item_at(ui);
        self.setup_rebuild_series(ui);
        self.setup_delete_series(ui);
    }
    
    fn setup_add_file(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        let file_service = self.file_service.clone();
        
        ui.on_add_file(move || {
            if let Some(ui) = ui_weak.upgrade() {
                // Get the selected series
                let selected_series_no = ui.get_selected_series_no().to_string();
                if selected_series_no.is_empty() {
                    eprintln!("No series selected");
                    return;
                }
                
                // Create and show the file input dialog as a separate window
                match FileInputDialog::new() {
                    Ok(dialog) => {
                        // Set the dialog language to match the main window
                        dialog.set_current_language(0);
                        
                        let archive_service = archive_service.clone();
                        let _file_service = file_service.clone();
                        let ui_weak2 = ui_weak.clone();
                        let dialog_weak = dialog.as_weak();
                        let selected_series_no = selected_series_no.clone();
                        
                        dialog.on_confirm_input(move |file_name: slint::SharedString| {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let file_name_str = file_name.to_string();
                                
                                if file_name_str.is_empty() {
                                    eprintln!("File name cannot be empty");
                                    return;
                                }
                                
                                let selected_index = ui.get_selected_archive();
                                
                                match archive_service.get_database_path_by_index(selected_index) {
                                    Ok(Some(db_path)) => {
                                        if !db_path.exists() {
                                            eprintln!("Database not found for selected archive");
                                            return;
                                        }
                                        
                                        let input = CreateFileInput {
                                            series_no: selected_series_no.clone(),
                                            name: file_name_str.clone(),
                                            created_at: None,
                                        };
                                        
                                        match FileService::create_file(&db_path, input) {
                                            Ok(result) => {
                                                println!("File created: {}", result.file_no);
                                                
                                                // Create file folder: lib_path/fond_no/file_no
                                                // Extract fond_no from series_no (format: fond_no-series_no)
                                                let fond_no = selected_series_no.split('-').next().unwrap_or(&selected_series_no);
                                                
                                                if let Some(archive_item) = ui.get_archive_library_items().row_data(selected_index as usize) {
                                                    let file_path = format!("{}/{}/{}", archive_item.path, fond_no, result.file_no);
                                                    if let Err(e) = std::fs::create_dir_all(&file_path) {
                                                        eprintln!("Failed to create file folder {}: {}", file_path, e);
                                                    } else {
                                                        println!("File folder created: {}", file_path);
                                                    }
                                                }
                                                
                                                // Reload files for the series
                                                Self::reload_files_for_series(&db_path, &selected_series_no, &ui);
                                                // Hide the dialog
                                                if let Some(d) = dialog_weak.upgrade() {
                                                    let _ = d.hide();
                                                }
                                            }
                                            Err(e) => {
                                                ui.invoke_show_toast(format!("创建文件失败: {}", e).into());
                                            }
                                        }
                                    }
                                    Ok(None) => eprintln!("No archive selected"),
                                    Err(e) => eprintln!("Failed to get database path: {}", e),
                                }
                            }
                        });
                        
                        let dialog_weak2 = dialog.as_weak();
                        dialog.on_cancel_input(move || {
                            println!("File dialog cancelled");
                            if let Some(d) = dialog_weak2.upgrade() {
                                let _ = d.hide();
                            }
                        });
                        
                        // Show the dialog as a separate window (non-blocking)
                        if let Err(e) = dialog.show() {
                            eprintln!("Failed to show file dialog: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to create file dialog: {}", e),
                }
            }
        });
    }

    fn setup_delete_file(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_delete_file(move || {
            if let Some(ui) = ui_weak.upgrade() {
                // Get the selected file
                let selected_file_index = ui.get_selected_file();
                if selected_file_index < 0 {
                    eprintln!("No file selected");
                    return;
                }
                
                // Get file info from the model
                let files_model = ui.get_files();
                if let Some(file_item) = files_model.row_data(selected_file_index as usize) {
                    let _file_no_str = file_item.file_no.clone();
                    let file_name_str = file_item.name.clone();
                    
                    // Get the selected series for context
                    let selected_series_no = ui.get_selected_series_no().to_string();
                    
                    // Create and show confirmation dialog
                    match crate::ConfirmDialog::new() {
                        Ok(dialog) => {
                            dialog.set_current_language(0);
                            dialog.set_message("确定要删除这个文件吗？".into());
                            
                            let archive_service = archive_service.clone();
                            let ui_weak2 = ui_weak.clone();
                            let file_name_str = file_name_str.clone();
                            let selected_series_no = selected_series_no.clone();
                            let dialog_weak = dialog.as_weak();
                            
                            dialog.on_confirm(move || {
                                if let Some(ui) = ui_weak2.upgrade() {
                                    let selected_index = ui.get_selected_archive();
                                    
                                    match archive_service.get_database_path_by_index(selected_index) {
                                        Ok(Some(db_path)) => {
                                            if !db_path.exists() {
                                                eprintln!("Database not found for selected archive");
                                                return;
                                            }
                                            
                                            // Get lib_path from archive_library_items
                                            let archive_library_items = ui.get_archive_library_items();
                                            let lib_path = if let Some(archive_item) = archive_library_items.row_data(selected_index as usize) {
                                                archive_item.path.to_string()
                                            } else {
                                                String::new()
                                            };
                                            
                                            // Extract fond_no from series_no (format: fond_no-year-...)
                                            let fond_no = selected_series_no.split('-').next().unwrap_or("").to_string();
                                            
                                            // First get the file_no by name and series
                                            match FondsService::list_files_by_series(&db_path, &selected_series_no) {
                                                Ok(files) => {
                                                    // Find the file with matching name
                                                    if let Some(file) = files.into_iter().find(|f| f.name == *file_name_str) {
                                                        let file_no_for_deletion = file.file_no.clone();
                                                        match FileService::delete_file(&db_path, &file_no_for_deletion) {
                                                            Ok(deleted) => {
                                                                if deleted {
                                                                    println!("File deleted: {}", file_no_for_deletion);
                                                                    
                                                                    // Delete file folder if it exists
                                                                    let file_folder_path = format!("{}/{}/{}", lib_path, fond_no, file_no_for_deletion);
                                                                    if Path::new(&file_folder_path).exists() {
                                                                        match fs::remove_dir(&file_folder_path) {
                                                                            Ok(_) => println!("File folder deleted: {}", file_folder_path),
                                                                            Err(e) => {
                                                                                eprintln!("Failed to delete file folder {}: {}", file_folder_path, e);
                                                                                ui.invoke_show_toast(format!("删除文件夹失败（文件夹不为空）: {}", e).into());
                                                                                return;
                                                                            }
                                                                        }
                                                                    }
                                                                    
                                                                    // Reload files for the series
                                                                    Self::reload_files_for_series(&db_path, &selected_series_no, &ui);
                                                                    // Close the dialog
                                                                    if let Some(d) = dialog_weak.upgrade() {
                                                                        let _ = d.hide();
                                                                    }
                                                                } else {
                                                                    ui.invoke_show_toast("文件不存在".into());
                                                                }
                                                            }
                                                            Err(e) => {
                                                                ui.invoke_show_toast(format!("删除文件失败: {}", e).into());
                                                            }
                                                        }
                                                    } else {
                                                        ui.invoke_show_toast("文件不存在".into());
                                                    }
                                                }
                                                Err(e) => {
                                                    ui.invoke_show_toast(format!("获取文件列表失败: {}", e).into());
                                                }
                                            }
                                        }
                                        Ok(None) => eprintln!("No archive selected"),
                                        Err(e) => eprintln!("Failed to get database path: {}", e),
                                    }
                                }
                            });
                            
                            let dialog_weak = dialog.as_weak();
                            dialog.on_cancel(move || {
                                println!("Delete file cancelled");
                                if let Some(d) = dialog_weak.upgrade() {
                                    let _ = d.hide();
                                }
                            });
                            
                            // Show the dialog
                            if let Err(e) = dialog.show() {
                                eprintln!("Failed to show delete confirmation dialog: {}", e);
                            }
                        }
                        Err(e) => eprintln!("Failed to create confirmation dialog: {}", e),
                    }
                } else {
                    eprintln!("Invalid file selection");
                }
            }
        });
    }

    fn setup_rename_file(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_rename_file(move |index, old_name| {
            if let Some(ui) = ui_weak.upgrade() {
                // Create a rename dialog
                match crate::RenameArchiveDialog::new() {
                    Ok(dialog) => {
                        dialog.set_current_language(0);
                        dialog.set_library_name_input(old_name.clone());
                        
                        let archive_service = archive_service.clone();
                        let ui_weak2 = ui_weak.clone();
                        let selected_series_no = ui.get_selected_series_no().to_string();
                        let dialog_weak = dialog.as_weak();
                        
                        dialog.on_confirm_input(move |new_name| {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let selected_index = ui.get_selected_archive();
                                
                                match archive_service.get_database_path_by_index(selected_index) {
                                    Ok(Some(db_path)) => {
                                        if !db_path.exists() {
                                            eprintln!("Database not found for selected archive");
                                            return;
                                        }
                                        
                                        // Get files for the series to find the file number
                                        match FondsService::list_files_by_series(&db_path, &selected_series_no) {
                                            Ok(files) => {
                                                // Find the file with matching old name (at the selected index)
                                                if let Some(file) = files.get(index as usize) {
                                                    let file_no = file.file_no.clone();
                                                    match FileService::rename_file(&db_path, &file_no, &new_name) {
                                                        Ok(true) => {
                                                            println!("File renamed: {} -> {}", old_name, new_name);
                                                            
                                                            // Reload files for the series
                                                            Self::reload_files_for_series(&db_path, &selected_series_no, &ui);
                                                            
                                                            // Close the dialog
                                                            if let Some(d) = dialog_weak.upgrade() {
                                                                let _ = d.hide();
                                                            }
                                                        }
                                                        Ok(false) => {
                                                            ui.invoke_show_toast("文件不存在".into());
                                                        }
                                                        Err(e) => {
                                                            ui.invoke_show_toast(format!("重命名文件失败: {}", e).into());
                                                        }
                                                    }
                                                } else {
                                                    ui.invoke_show_toast("文件不存在".into());
                                                }
                                            }
                                            Err(e) => {
                                                ui.invoke_show_toast(format!("获取文件列表失败: {}", e).into());
                                            }
                                        }
                                    }
                                    Ok(None) => eprintln!("No archive selected"),
                                    Err(e) => eprintln!("Failed to get database path: {}", e),
                                }
                            }
                        });
                        
                        let dialog_weak = dialog.as_weak();
                        dialog.on_cancel_input(move || {
                            println!("Rename file cancelled");
                            if let Some(d) = dialog_weak.upgrade() {
                                let _ = d.hide();
                            }
                        });
                        
                        // Show the dialog
                        if let Err(e) = dialog.show() {
                            eprintln!("Failed to show rename dialog: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to create rename dialog: {}", e),
                }
            }
        });
    }

    fn setup_delete_selected_files(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_delete_selected_files(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_files = ui.get_selected_files();
                if selected_files.row_count() == 0 {
                    eprintln!("No files selected for deletion");
                    return;
                }
                
                // Create and show confirmation dialog
                match crate::ConfirmDialog::new() {
                    Ok(dialog) => {
                        dialog.set_current_language(0);
                        dialog.set_message(format!("确定要删除选中的 {} 个文件吗？", selected_files.row_count()).into());
                        
                        let archive_service = archive_service.clone();
                        let ui_weak2 = ui_weak.clone();
                        let selected_files = selected_files.clone();
                        let dialog_weak = dialog.as_weak();
                        
                        dialog.on_confirm(move || {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let selected_index = ui.get_selected_archive();
                                let selected_series_no = ui.get_selected_series_no().to_string();
                                
                                match archive_service.get_database_path_by_index(selected_index) {
                                    Ok(Some(db_path)) => {
                                        if !db_path.exists() {
                                            eprintln!("Database not found for selected archive");
                                            return;
                                        }
                                        
                                        // First get all files for the series
                                        match FondsService::list_files_by_series(&db_path, &selected_series_no) {
                                            Ok(all_files) => {
                                                // Get lib_path from archive_library_items
                                                let archive_library_items = ui.get_archive_library_items();
                                                let lib_path = if let Some(archive_item) = archive_library_items.row_data(selected_index as usize) {
                                                    archive_item.path.to_string()
                                                } else {
                                                    String::new()
                                                };
                                                
                                                // Extract fond_no from series_no
                                                let fond_no = selected_series_no.split('-').next().unwrap_or("").to_string();
                                                let mut deleted_count = 0;
                                                let mut folder_delete_errors = Vec::new();
                                                
                                                // Delete each selected file
                                                for i in 0..selected_files.row_count() {
                                                    if let Some(is_selected) = selected_files.row_data(i) {
                                                        if is_selected == 1 {
                                                            // Find the file by index in the UI list
                                                            let files_model = ui.get_files();
                                                            if let Some(file_item) = files_model.row_data(i) {
                                                                let file_no_str = file_item.file_no.clone();
                                                                
                                                                // Find the file in all_files by file_no
                                                                if let Some(file) = all_files.iter().find(|f| f.file_no == *file_no_str) {
                                                                    match FileService::delete_file(&db_path, &file.file_no) {
                                                                        Ok(true) => {
                                                                            println!("File deleted: {}", file.file_no);
                                                                            
                                                                            // Delete file folder
                                                                            let file_folder_path = format!("{}/{}/{}", lib_path, fond_no, file.file_no);
                                                                            if Path::new(&file_folder_path).exists() {
                                                                                match fs::remove_dir(&file_folder_path) {
                                                                                    Ok(_) => println!("File folder deleted: {}", file_folder_path),
                                                                                    Err(e) => {
                                                                                        eprintln!("Failed to delete file folder {}: {}", file_folder_path, e);
                                                                                        folder_delete_errors.push(format!("{}: {}", file_folder_path, e));
                                                                                    }
                                                                                }
                                                                            }
                                                                            
                                                                            deleted_count += 1;
                                                                        }
                                                                        Ok(false) => {
                                                                            eprintln!("File not found: {}", file_no_str);
                                                                        }
                                                                        Err(e) => {
                                                                            eprintln!("Failed to delete file {}: {}", file_no_str, e);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                
                                                if !folder_delete_errors.is_empty() {
                                                    ui.invoke_show_toast(format!("删除文件夹失败（文件夹不为空）: {}", folder_delete_errors.join(", ")).into());
                                                }
                                                
                                                if deleted_count > 0 {
                                                    // Reload files for the series
                                                    Self::reload_files_for_series(&db_path, &selected_series_no, &ui);
                                                    // Clear selection
                                                    ui.set_selected_files(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                                                    ui.set_selected_file(-1);
                                                    
                                                    if let Some(d) = dialog_weak.upgrade() {
                                                        let _ = d.hide();
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                ui.invoke_show_toast(format!("获取文件列表失败: {}", e).into());
                                            }
                                        }
                                    }
                                    Ok(None) => eprintln!("No archive selected"),
                                    Err(e) => eprintln!("Failed to get database path: {}", e),
                                }
                            }
                        });
                        
                        let dialog_weak = dialog.as_weak();
                        dialog.on_cancel(move || {
                            println!("Delete selected files cancelled");
                            if let Some(d) = dialog_weak.upgrade() {
                                let _ = d.hide();
                            }
                        });
                        
                        // Show the dialog
                        if let Err(e) = dialog.show() {
                            eprintln!("Failed to show delete confirmation dialog: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to create confirmation dialog: {}", e),
                }
            }
        });
    }

    fn setup_add_fonds_dialog(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_open_add_fonds_dialog(move || {
            if let Some(ui) = ui_weak.upgrade() {
                // Create and show the add fonds dialog as a separate window
                match crate::AddFondsDialog::new() {
                    Ok(dialog) => {
                        // Set the dialog language to match the main window
                        dialog.set_current_language(0);
                        
                        // Load classifications and schemas from database
                        let selected_index = ui.get_selected_archive();
                        if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_index) {
                            // Load top classifications
                            if let Ok(top_classifications) = ClassificationService::list_top(&db_path) {
                                let primary_names: Vec<slint::SharedString> = top_classifications.iter()
                                    .map(|c| format!("{} - {}", c.code, c.name).into())
                                    .collect();
                                dialog.set_primary_classifications((&primary_names[..]).into());
                                
                                // Load secondary classifications for each primary
                                let mut secondary_lists: Vec<Vec<slint::SharedString>> = Vec::new();
                                for top in &top_classifications {
                                    if let Ok(children) = ClassificationService::list_children(&db_path, &top.code) {
                                        let child_names: Vec<slint::SharedString> = children.iter()
                                            .map(|c| format!("{} - {}", c.code, c.name).into())
                                            .collect();
                                        secondary_lists.push(child_names);
                                    } else {
                                        secondary_lists.push(Vec::new());
                                    }
                                }
                                let secondary_models: Vec<slint::ModelRc<slint::SharedString>> = secondary_lists.into_iter()
                                    .map(|v| (&v[..]).into())
                                    .collect();
                                dialog.set_secondary_classifications((&secondary_models[..]).into());
                            }
                            
                            // Load available schemas
                            if let Ok(schemas) = SchemaService::list_schemas(&db_path) {
                                use crate::FondsSchemaOption;
                                let schema_options: Vec<FondsSchemaOption> = schemas.iter()
                                    .map(|s| FondsSchemaOption {
                                        code: s.schema_no.clone().into(),
                                        name: s.name.clone().into(),
                                    })
                                    .collect();
                                dialog.set_available_schema_items((&schema_options[..]).into());
                            }
                        }
                        
                        let dialog_weak_move = dialog.as_weak();
                        dialog.on_move_schema_to_selected(move |index: i32| {
                            if let Some(dialog) = dialog_weak_move.upgrade() {
                                let available = dialog.get_available_schema_items();
                                let chosen = dialog.get_chosen_schema_items();
                                
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
                                    dialog.set_available_schema_items((&available_vec[..]).into());
                                    dialog.set_chosen_schema_items((&chosen_vec[..]).into());
                                }
                            }
                        });
                        
                        let dialog_weak_move2 = dialog.as_weak();
                        dialog.on_move_schema_back(move |index: i32| {
                            if let Some(dialog) = dialog_weak_move2.upgrade() {
                                let available = dialog.get_available_schema_items();
                                let chosen = dialog.get_chosen_schema_items();
                                
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
                                    dialog.set_available_schema_items((&available_vec[..]).into());
                                    dialog.set_chosen_schema_items((&chosen_vec[..]).into());
                                }
                            }
                        });
                        
                        let archive_service = archive_service.clone();
                        let ui_weak2 = ui_weak.clone();
                        let dialog_weak = dialog.as_weak();
                        let selected_index = ui.get_selected_archive();
                        let archive_library_items = ui.get_archive_library_items();
                        
                        dialog.on_confirm(move || {
                            if let Some(dialog) = dialog_weak.upgrade() {
                                let fonds_name = dialog.get_fonds_name_input().to_string();
                                
                                // Get selected classification
                                let primary_idx = dialog.get_selected_primary_classification() as usize;
                                let secondary_idx = dialog.get_selected_secondary_classification() as usize;
                                let secondary_classifications = dialog.get_secondary_classifications();
                                
                                // Extract classification code from the secondary classification
                                // Format: "CODE - Name"
                                let classification_code = if secondary_classifications.row_count() > primary_idx {
                                    if let Some(secondary_list) = secondary_classifications.row_data(primary_idx) {
                                        if secondary_list.row_count() > secondary_idx {
                                            if let Some(secondary) = secondary_list.row_data(secondary_idx) {
                                                secondary.to_string()
                                                    .split(" - ")
                                                    .next()
                                                    .unwrap_or("")
                                                    .to_string()
                                            } else {
                                                String::new()
                                            }
                                        } else {
                                            String::new()
                                        }
                                    } else {
                                        String::new()
                                    }
                                } else {
                                    String::new()
                                };
                                
                                if classification_code.is_empty() {
                                    eprintln!("No classification selected");
                                    return;
                                }
                                
                                // Get selected schemas
                                let chosen_schemas = dialog.get_chosen_schema_items();
                                let mut schema_codes: Vec<String> = Vec::new();
                                for i in 0..chosen_schemas.row_count() {
                                    if let Some(schema) = chosen_schemas.row_data(i) {
                                        schema_codes.push(schema.code.to_string());
                                    }
                                }
                                
                                if schema_codes.is_empty() {
                                    eprintln!("No schemas selected");
                                    return;
                                }
                                
                                // Create fonds input
                                let input = CreateFondsInput {
                                    classification_code: classification_code.clone(),
                                    name: fonds_name.clone(),
                                    schema_codes,
                                    created_at: None, // Use current date
                                };
                                
                                // Call service to create fonds
                                match archive_service.get_database_path_by_index(selected_index) {
                                    Ok(Some(db_path)) => {
                                        match FondsService::create_fonds(&db_path, input) {
                                            Ok(result) => {
                                                println!("Fonds created: {} with {} series", result.fond_no, result.series_count);
                                                
                                                // Create fonds folder: lib_path/fond_no
                                                if let Some(archive_item) = archive_library_items.row_data(selected_index as usize) {
                                                    let fonds_path = format!("{}/{}", archive_item.path, result.fond_no);
                                                    if let Err(e) = std::fs::create_dir_all(&fonds_path) {
                                                        eprintln!("Failed to create fonds folder {}: {}", fonds_path, e);
                                                    } else {
                                                        println!("Fonds folder created: {}", fonds_path);
                                                    }
                                                }
                                                
                                                // Reload fonds list in UI
                                                if let Some(ui) = ui_weak2.upgrade() {
                                                    Self::reload_fonds(&db_path, &ui);
                                                }
                                                
                                                // Hide the dialog
                                                let _ = dialog.hide();
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to create fonds: {}", e);
                                                if let Some(ui) = ui_weak2.upgrade() {
                                                    ui.invoke_show_toast(format!("创建全宗失败: {}", e).into());
                                                }
                                            }
                                        }
                                    }
                                    Ok(None) => eprintln!("No archive selected"),
                                    Err(e) => eprintln!("Failed to get database path: {}", e),
                                }
                            }
                        });
                        
                        let dialog_weak2 = dialog.as_weak();
                        dialog.on_cancel(move || {
                            println!("Add fonds dialog cancelled");
                            if let Some(d) = dialog_weak2.upgrade() {
                                let _ = d.hide();
                            }
                        });
                        
                        // Show the dialog as a separate window (non-blocking)
                        if let Err(e) = dialog.show() {
                            eprintln!("Failed to show add fonds dialog: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to create add fonds dialog: {}", e),
                }
            }
        });
    }
    
    fn setup_select_series(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_select_series(move |index, series_no| {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_selected_series_index(index);
                ui.set_selected_series_no(series_no.clone());
                
                // Load files for the selected series
                let selected_archive = ui.get_selected_archive();
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_archive) {
                    Self::reload_files_for_series(&db_path, &series_no, &ui);
                }
            }
        });
    }

    fn setup_file_clicked(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_file_clicked(move |index: i32, ctrl: bool, shift: bool| {
            if let Some(ui) = ui_weak.upgrade() {
                let files = ui.get_files();
                let total_count = files.row_count() as i32;
                
                if index < 0 || index >= total_count {
                    return;
                }
                
                if ctrl {
                    // Ctrl+Click: Toggle selection
                    let mut selections = vec![0; total_count as usize];
                    let current_selections = ui.get_selected_files();
                    
                    // Copy current selections
                    for i in 0..current_selections.row_count() {
                        if i < selections.len() {
                            selections[i] = current_selections.row_data(i).unwrap_or(0);
                        }
                    }
                    
                    // Toggle current item
                    selections[index as usize] = if selections[index as usize] == 1 { 0 } else { 1 };
                    
                    let model = slint::ModelRc::new(slint::VecModel::from(selections));
                    ui.set_selected_files(model);
                    ui.set_selected_file(index);
                } else if shift {
                    // Shift+Click: Range selection
                    let last_index = ui.get_selected_file();
                    let start = last_index.min(index);
                    let end = last_index.max(index);
                    
                    let mut selections = vec![0; total_count as usize];
                    for i in start..=end {
                        selections[i as usize] = 1;
                    }
                    
                    let model = slint::ModelRc::new(slint::VecModel::from(selections));
                    ui.set_selected_files(model);
                    ui.set_selected_file(index);
                } else {
                    // Normal click: Clear multi-selection and select single item
                    ui.set_selected_file(index);
                    ui.set_selected_files(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                }
                
                // Trigger file_selected to load items for the selected file
                ui.invoke_file_selected(index);
            }
        });
    }

    fn setup_file_selected(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_file_selected(move |file_index| {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_archive = ui.get_selected_archive();
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_archive) {
                    let files = ui.get_files();
                    if file_index >= 0 && (file_index as usize) < files.row_count() {
                        if let Some(file_item) = files.row_data(file_index as usize) {
                            let file_no_str = file_item.file_no.clone();
                            match FondsService::list_items_by_file(&db_path, &file_no_str) {
                                Ok(items) => {
                                    let item_items: Vec<crate::ItemItem> = items.into_iter()
                                        .map(|i| crate::ItemItem {
                                            item_no: i.item_no.into(),
                                            name: i.name.into(),
                                            path: i.path.unwrap_or_default().into(),
                                        })
                                        .collect();
                                    set_items_with_crud_items(item_items, &ui);
                                    
                                    // Reset item selection
                                    ui.set_selected_item(0);
                                }
                                Err(e) => {
                                    eprintln!("Failed to load items for file {}: {}", file_no_str, e);
                                    clear_items(&ui);
                                    ui.set_selected_item(-1);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    fn setup_add_item(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_add_item(move || {
            if let Some(ui) = ui_weak.upgrade() {
                // Get the selected file
                let files = ui.get_files();
                let selected_file_index = ui.get_selected_file();
                
                if selected_file_index < 0 || (selected_file_index as usize) >= files.row_count() {
                    eprintln!("No file selected");
                    return;
                }
                
                if let Some(file_item) = files.row_data(selected_file_index as usize) {
                    let file_no = file_item.file_no.to_string();
                    let file_path = file_item.path.to_string();
                    
                    // Create and show the add item dialog as a separate window
                    match crate::AddItemDialog::new() {
                        Ok(dialog) => {
                            // Set the dialog language to match the main window
                            dialog.set_current_language(0);
                            
                            // Set the file base path for restriction
                            dialog.set_file_base_path(file_path.clone().into());
                            
                            let archive_service = archive_service.clone();
                            let ui_weak2 = ui_weak.clone();
                            let dialog_weak = dialog.as_weak();
                            let dialog_weak_for_browse = dialog_weak.clone();
                            let browse_base_path = file_path.clone();
                            
                            // Handle browse_file callback
                            dialog.on_browse_file(move || {
                                // Start from the file's base path, normalized for current OS
                                #[cfg(target_os = "windows")]
                                let normalized_browse_path = browse_base_path.replace("/", "\\");
                                #[cfg(not(target_os = "windows"))]
                                let normalized_browse_path = browse_base_path.clone();
                                
                                let start_dir = std::path::Path::new(&normalized_browse_path);
                                
                                // Try to pick a file first
                                let mut file_dialog = rfd::FileDialog::new();
                                if start_dir.exists() && start_dir.is_dir() {
                                    file_dialog = file_dialog.set_directory(start_dir);
                                } else {
                                    println!("Browse start directory does not exist or is not a dir: {}", normalized_browse_path);
                                }
                                
                                // First try to get a file, if cancelled, try folder
                                if let Some(path) = file_dialog.add_filter("All", &["*"]).pick_file() {
                                    if let Some(d) = dialog_weak_for_browse.upgrade() {
                                        let path_str = path.to_string_lossy().to_string();
                                        let file_name = std::path::Path::new(&path_str)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("")
                                            .to_string();
                                        d.set_selected_file_path(path_str.into());
                                        if !file_name.is_empty() {
                                            d.set_item_name_input(file_name.into());
                                        }
                                    }
                                } else {
                                    // If pick_file was cancelled, try pick_folder
                                    let mut folder_dialog = rfd::FileDialog::new();
                                    if start_dir.exists() && start_dir.is_dir() {
                                        folder_dialog = folder_dialog.set_directory(start_dir);
                                    }
                                    
                                    if let Some(path) = folder_dialog.pick_folder() {
                                        if let Some(d) = dialog_weak_for_browse.upgrade() {
                                            let path_str = path.to_string_lossy().to_string();
                                            let folder_name = std::path::Path::new(&path_str)
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("")
                                                .to_string();
                                            d.set_selected_file_path(path_str.into());
                                            if !folder_name.is_empty() {
                                                d.set_item_name_input(folder_name.into());
                                            }
                                        }
                                    }
                                }
                            });
                            
                            dialog.on_confirm_input(move |item_name: slint::SharedString, file_path: slint::SharedString| {
                                if let Some(ui) = ui_weak2.upgrade() {
                                    let item_name_str = item_name.to_string();
                                    let file_path_str = file_path.to_string();
                                    
                                    if item_name_str.is_empty() || file_path_str.is_empty() {
                                        eprintln!("Item name and file path cannot be empty");
                                        return;
                                    }
                                    
                                    let selected_archive = ui.get_selected_archive();
                                    
                                    match archive_service.get_database_path_by_index(selected_archive) {
                                        Ok(Some(db_path)) => {
                                            if !db_path.exists() {
                                                eprintln!("Database not found for selected archive");
                                                return;
                                            }
                                            
                                            // Create item input with path
                                            let input = CreateItemInput {
                                                file_no: file_no.clone(),
                                                name: item_name_str,
                                                path: Some(file_path_str),
                                                created_at: None,
                                            };
                                            
                                            // Call service to create item
                                            match FileService::create_item(&db_path, input) {
                                                Ok(result) => {
                                                    println!("Item created with ID: {}", result.item_no);
                                                    
                                                    // Reload items for the file as ItemItem
                                                    match FileService::list_items_by_file(&db_path, &file_no) {
                                                        Ok(items) => {
                                                            let item_items: Vec<crate::ItemItem> = items.into_iter()
                                                                .map(|i| crate::ItemItem {
                                                                    item_no: i.item_no.into(),
                                                                    name: i.name.into(),
                                                                    path: i.path.unwrap_or_default().into(),
                                                                })
                                                                .collect();
                                                            set_items_with_crud_items(item_items, &ui);
                                                            
                                                            // Reset item selection
                                                            ui.set_selected_item(0);
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Failed to reload items: {}", e);
                                                        }
                                                    }
                                                    
                                                    // Hide the dialog
                                                    if let Some(d) = dialog_weak.upgrade() {
                                                        let _ = d.hide();
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to create item: {}", e);
                                                }
                                            }
                                        }
                                        _ => {
                                            eprintln!("Failed to get database path");
                                        }
                                    }
                                }
                            });
                            
                            let dialog_weak_cancel = dialog.as_weak();
                            dialog.on_cancel_input(move || {
                                println!("Add item dialog cancelled");
                                if let Some(d) = dialog_weak_cancel.upgrade() {
                                    let _ = d.hide();
                                }
                            });
                            
                            let _ = dialog.show();
                        }
                        Err(e) => {
                            eprintln!("Failed to show add item dialog: {}", e);
                        }
                    }
                }
            }
        });
    }
    
    fn setup_delete_item(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_delete_item(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_item_index = ui.get_selected_item();
                let items = ui.get_file_item_list();
                
                if selected_item_index < 0 || (selected_item_index as usize) >= items.row_count() {
                    return;
                }
                
                let selected_archive = ui.get_selected_archive();
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_archive) {
                    if let Some(item) = items.row_data(selected_item_index as usize) {
                        let item_no = item.item_no.to_string();
                        
                        // Delete the item from database
                        match FileService::delete_item(&db_path, &item_no) {
                            Ok(deleted) => {
                                if deleted {
                                    println!("Item {} deleted successfully", item_no);
                                    
                                    // Reload items for the current file
                                    let files = ui.get_files();
                                    let selected_file = ui.get_selected_file();
                                    if selected_file >= 0 && (selected_file as usize) < files.row_count() {
                                        if let Some(file_item) = files.row_data(selected_file as usize) {
                                            let file_no = file_item.file_no.to_string();
                                            match FileService::list_items_by_file(&db_path, &file_no) {
                                                Ok(items) => {
                                                    let item_items: Vec<crate::ItemItem> = items.into_iter()
                                                        .map(|i| crate::ItemItem {
                                                            item_no: i.item_no.into(),
                                                            name: i.name.into(),
                                                            path: i.path.unwrap_or_default().into(),
                                                        })
                                                        .collect();
                                                    set_items_with_crud_items(item_items, &ui);
                                                    
                                                    // Adjust selection
                                                    let new_count = ui.get_file_item_list().row_count() as i32;
                                                    if selected_item_index >= new_count {
                                                        ui.set_selected_item(new_count - 1);
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to reload items: {}", e);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to delete item: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }
    
    fn setup_rename_item(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_rename_item(move |index, old_name| {
            if let Some(ui) = ui_weak.upgrade() {
                // Create a rename dialog
                match crate::RenameArchiveDialog::new() {
                    Ok(dialog) => {
                        dialog.set_current_language(0);
                        dialog.set_library_name_input(old_name.clone());
                        
                        let archive_service = archive_service.clone();
                        let ui_weak2 = ui_weak.clone();
                        let selected_file_no = ui.get_selected_file();
                        let dialog_weak = dialog.as_weak();
                        
                        dialog.on_confirm_input(move |new_name| {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let selected_archive = ui.get_selected_archive();
                                
                                match archive_service.get_database_path_by_index(selected_archive) {
                                    Ok(Some(db_path)) => {
                                        if !db_path.exists() {
                                            eprintln!("Database not found for selected archive");
                                            return;
                                        }
                                        
                                        // Get file_no from UI state
                                        let file_model = ui.get_files();
                                        if let Some(file_item) = file_model.row_data(selected_file_no as usize) {
                                            let file_no = file_item.file_no.clone();
                                            
                                            // Get items for the file to find the item number
                                            match FileService::list_items_by_file(&db_path, &file_no) {
                                                Ok(items) => {
                                                    // Find the item with matching old name (at the selected index)
                                                    if let Some(item) = items.get(index as usize) {
                                                        let item_no = item.item_no.clone();
                                                        match FileService::rename_item(&db_path, &item_no, &new_name) {
                                                            Ok(true) => {
                                                                println!("Item renamed: {} -> {}", old_name, new_name);
                                                                
                                                                // Reload items for the file
                                                                Self::reload_items_for_file(&db_path, &file_no, &ui);
                                                                
                                                                // Close the dialog
                                                                if let Some(d) = dialog_weak.upgrade() {
                                                                    let _ = d.hide();
                                                                }
                                                            }
                                                            Ok(false) => {
                                                                ui.invoke_show_toast("项目不存在".into());
                                                            }
                                                            Err(e) => {
                                                                ui.invoke_show_toast(format!("重命名项目失败: {}", e).into());
                                                            }
                                                        }
                                                    } else {
                                                        ui.invoke_show_toast("项目不存在".into());
                                                    }
                                                }
                                                Err(e) => {
                                                    ui.invoke_show_toast(format!("获取项目列表失败: {}", e).into());
                                                }
                                            }
                                        }
                                    }
                                    Ok(None) => eprintln!("No archive selected"),
                                    Err(e) => eprintln!("Failed to get database path: {}", e),
                                }
                            }
                        });
                        
                        let dialog_weak = dialog.as_weak();
                        dialog.on_cancel_input(move || {
                            println!("Rename item cancelled");
                            if let Some(d) = dialog_weak.upgrade() {
                                let _ = d.hide();
                            }
                        });
                        
                        // Show the dialog
                        if let Err(e) = dialog.show() {
                            eprintln!("Failed to show rename dialog: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to create rename dialog: {}", e),
                }
            }
        });
    }
    
    fn setup_delete_selected_items(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_delete_selected_items(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let items = ui.get_file_item_list();
                let selected_items = ui.get_selected_items();
                
                if selected_items.row_count() == 0 || items.row_count() == 0 {
                    return;
                }
                
                let selected_archive = ui.get_selected_archive();
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_archive) {
                    // Collect item_nos to delete
                    let mut items_to_delete = Vec::new();
                    for i in 0..selected_items.row_count() {
                        if let Some(is_selected) = selected_items.row_data(i) {
                            if is_selected != 0 {
                                if let Some(item) = items.row_data(i) {
                                    items_to_delete.push(item.item_no.to_string());
                                }
                            }
                        }
                    }
                    
                    // Delete each item
                    let mut deleted_count = 0;
                    for item_no in &items_to_delete {
                        if let Ok(true) = FileService::delete_item(&db_path, item_no) {
                            deleted_count += 1;
                        }
                    }
                    
                    if deleted_count > 0 {
                        println!("Deleted {} items", deleted_count);
                        
                        // Reload items for the current file
                        let files = ui.get_files();
                        let selected_file = ui.get_selected_file();
                        if selected_file >= 0 && (selected_file as usize) < files.row_count() {
                            if let Some(file_item) = files.row_data(selected_file as usize) {
                                let file_no = file_item.file_no.to_string();
                                match FileService::list_items_by_file(&db_path, &file_no) {
                                    Ok(items) => {
                                        let item_items: Vec<crate::ItemItem> = items.into_iter()
                                            .map(|i| crate::ItemItem {
                                                item_no: i.item_no.into(),
                                                name: i.name.into(),
                                                path: i.path.unwrap_or_default().into(),
                                            })
                                            .collect();
                                        set_items_with_crud_items(item_items, &ui);
                                        
                                        // Clear selection and reset
                                        ui.set_selected_item(0);
                                        let empty_selection: Vec<i32> = Vec::new();
                                        let selection_model = slint::ModelRc::new(slint::VecModel::from(empty_selection));
                                        ui.set_selected_items(selection_model);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to reload items: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    fn setup_item_clicked(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_item_clicked(move |index: i32, ctrl: bool, shift: bool| {
            println!("Item clicked: index={}, ctrl={}, shift={}", index, ctrl, shift);
            if let Some(ui) = ui_weak.upgrade() {
                let items = ui.get_file_item_list();
                let total_count = items.row_count() as i32;
                println!("Total items count: {}", total_count);
                
                if index < 0 || index >= total_count {
                    println!("Index out of bounds!");
                    return;
                }
                
                if ctrl {
                    // Ctrl+Click: Toggle selection
                    let mut selections = vec![0; total_count as usize];
                    let current_selections = ui.get_selected_items();
                    
                    // Copy current selections
                    for i in 0..current_selections.row_count() {
                        if i < selections.len() {
                            selections[i] = current_selections.row_data(i).unwrap_or(0);
                        }
                    }
                    
                    // Toggle current item
                    selections[index as usize] = if selections[index as usize] == 1 { 0 } else { 1 };
                    
                    let model = slint::ModelRc::new(slint::VecModel::from(selections));
                    ui.set_selected_items(model);
                    ui.set_selected_item(index);
                } else if shift {
                    // Shift+Click: Range selection
                    let last_index = ui.get_selected_item();
                    let start = last_index.min(index);
                    let end = last_index.max(index);
                    
                    let mut selections = vec![0; total_count as usize];
                    for i in start..=end {
                        selections[i as usize] = 1;
                    }
                    
                    let model = slint::ModelRc::new(slint::VecModel::from(selections));
                    ui.set_selected_items(model);
                    ui.set_selected_item(index);
                } else {
                    // Normal click: Clear multi-selection and select single item
                    ui.set_selected_item(index);
                    ui.set_selected_items(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                }
            }
        });
    }
    
    fn setup_fonds_selected(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_fonds_selected(move |fonds_index| {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_archive = ui.get_selected_archive();
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_archive) {
                    // Load series for the selected fonds
                    if let Ok(fonds_list) = FondsService::list_fonds(&db_path) {
                        if (fonds_index as usize) < fonds_list.len() {
                            let selected_fonds = &fonds_list[fonds_index as usize];
                            match FondsService::list_series(&db_path, &selected_fonds.fond_no) {
                                Ok(series_list) => {
                                    let series_items: Vec<crate::SeriesItem> = series_list.into_iter()
                                        .map(|s| crate::SeriesItem {
                                            series_no: s.series_no.into(),
                                            name: s.name.into(),
                                            fond_no: s.fond_no.into(),
                                        })
                                        .collect();
                                    set_series_with_crud_items(series_items, &ui);
                                    
                                    // Reset series selection
                                    ui.set_selected_series_index(-1);
                                    ui.set_selected_series_no("".into());
                                }
                                Err(e) => {
                                    eprintln!("Failed to load series for fonds {}: {}", selected_fonds.fond_no, e);
                                    clear_series(&ui);
                                    ui.set_selected_series_index(-1);
                                    ui.set_selected_series_no("".into());
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    /// Public method to refresh fonds data for a specific archive
    pub fn refresh_fonds_data_for_archive(&self, archive_index: i32, ui: &AppWindow) {
        self.reload_fonds_and_series_for_archive(archive_index, ui);
    }
    
    /// Helper to reload fonds and series lists into UI for a specific archive
    pub fn reload_fonds_and_series_for_archive(&self, archive_index: i32, ui: &AppWindow) {
        if let Ok(Some(db_path)) = self.archive_service.get_database_path_by_index(archive_index) {
            Self::reload_fonds_and_series(&db_path, ui);
        } else {
            // Clear data if no valid archive selected
            let empty_names: Vec<slint::SharedString> = Vec::new();
            let names_model = slint::ModelRc::new(slint::VecModel::from(empty_names));
            ui.set_fonds_names(names_model);

            clear_series(ui);

            ui.set_selected_fonds(0);
            ui.set_selected_series_index(-1);
            ui.set_selected_series_no("".into());
        }
    }
    
    /// Helper to reload fonds and series data into UI
    fn reload_fonds_and_series(db_path: &std::path::PathBuf, ui: &AppWindow) {
        // Get the selected archive path for calculating fonds paths
        let selected_archive = ui.get_selected_archive();
        let archive_library_items = ui.get_archive_library_items();
        let lib_path = if selected_archive >= 0 && (selected_archive as usize) < archive_library_items.row_count() {
            if let Some(archive_item) = archive_library_items.row_data(selected_archive as usize) {
                archive_item.path.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        // Load fonds names for dropdown and fonds items with paths
        match FondsService::list_fonds(db_path) {
            Ok(fonds_list) => {
                let names: Vec<slint::SharedString> = fonds_list.iter().map(|f| format!("{} - {}", f.fond_no, f.name).into()).collect();
                let names_model = slint::ModelRc::new(slint::VecModel::from(names));
                ui.set_fonds_names(names_model);
                
                // Create fonds items with calculated paths
                let fonds_items: Vec<crate::FondsItem> = fonds_list.into_iter()
                    .map(|f| {
                        let fonds_path = if !lib_path.is_empty() {
                            format!("{}/{}", lib_path, f.fond_no)
                        } else {
                            f.fond_no.clone()
                        };
                        crate::FondsItem {
                            fond_no: f.fond_no.into(),
                            name: f.name.into(),
                            classification_code: f.fond_classification_code.into(),
                            created_at: f.created_at.into(),
                            path: fonds_path.into(),
                        }
                    })
                    .collect();
                let fonds_model = slint::ModelRc::new(slint::VecModel::from(fonds_items));
                ui.set_fonds_items(fonds_model);
            }
            Err(e) => {
                eprintln!("Failed to load fonds: {}", e);
                let empty_names: Vec<slint::SharedString> = Vec::new();
                let names_model = slint::ModelRc::new(slint::VecModel::from(empty_names));
                ui.set_fonds_names(names_model);
                
                let empty_fonds: Vec<crate::FondsItem> = Vec::new();
                let fonds_model = slint::ModelRc::new(slint::VecModel::from(empty_fonds));
                ui.set_fonds_items(fonds_model);
            }
        }
        
        // Load series for the first fonds (if any)
        if let Ok(fonds_list) = FondsService::list_fonds(db_path) {
            if let Some(first_fonds) = fonds_list.first() {
                match FondsService::list_series(db_path, &first_fonds.fond_no) {
                    Ok(series_list) => {
                        let series_items: Vec<crate::SeriesItem> = series_list.into_iter()
                            .map(|s| crate::SeriesItem {
                                series_no: s.series_no.into(),
                                name: s.name.into(),
                                fond_no: s.fond_no.into(),
                            })
                            .collect();
                        set_series_with_crud_items(series_items, ui);
                    }
                    Err(e) => {
                        eprintln!("Failed to load series: {}", e);
                        clear_series(ui);
                    }
                }
            } else {
                // No fonds, clear series
                clear_series(ui);
            }
        }
        
        // Reset selections
        ui.set_selected_fonds(0);
        ui.set_selected_series_index(-1);
        ui.set_selected_series_no("".into());
    }
    
    /// Helper to reload fonds list into UI (backward compatible)
    fn reload_fonds(db_path: &std::path::PathBuf, ui: &AppWindow) {
        Self::reload_fonds_and_series(db_path, ui);
    }
    
    /// Helper to reload files for a specific series
    fn reload_files_for_series(db_path: &std::path::PathBuf, series_no: &str, ui: &AppWindow) {
        match FondsService::list_files_by_series(db_path, series_no) {
            Ok(files) => {
            // Get the selected archive path for calculating file paths
            let selected_archive = ui.get_selected_archive();
            let archive_library_items = ui.get_archive_library_items();
            let lib_path = if selected_archive >= 0 && (selected_archive as usize) < archive_library_items.row_count() {
                if let Some(archive_item) = archive_library_items.row_data(selected_archive as usize) {
                    archive_item.path.to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };                // Get the fond_no from series_no (assuming format is fond_no-series_no)
                let fond_no = series_no.split('-').next().unwrap_or(series_no);
                
                let file_items: Vec<crate::FileItem> = files.into_iter()
                    .map(|f| {
                        let file_path = if !lib_path.is_empty() {
                            format!("{}/{}/{}", lib_path, fond_no, f.file_no)
                        } else {
                            format!("{}/{}", fond_no, f.file_no)
                        };
                        crate::FileItem {
                            file_no: f.file_no.into(),
                            name: f.name.into(),
                            path: file_path.into(),
                        }
                    })
                    .collect();
                set_files_with_crud_items(file_items, ui);
                
                // Reset file selection and trigger file_selected
                ui.set_selected_file(0);
                ui.invoke_file_selected(0);
            }
            Err(e) => {
                eprintln!("Failed to load files for series {}: {}", series_no, e);
                clear_files(ui);
                ui.set_selected_file(-1);
                // Clear items
                clear_items(ui);
                ui.set_selected_item(-1);
            }
        }
    }
    
    fn reload_items_for_file(db_path: &std::path::PathBuf, file_no: &str, ui: &AppWindow) {
        match FileService::list_items_by_file(db_path, file_no) {
            Ok(items) => {
                let item_items: Vec<crate::ItemItem> = items.into_iter()
                    .map(|i| {
                        crate::ItemItem {
                            item_no: i.item_no.into(),
                            name: i.name.into(),
                            path: i.path.unwrap_or_default().into(),
                        }
                    })
                    .collect();
                set_items_with_crud_items(item_items, ui);
                
                // Reset item selection
                ui.set_selected_item(0);
            }
            Err(e) => {
                eprintln!("Failed to load items for file {}: {}", file_no, e);
                clear_items(ui);
                ui.set_selected_item(-1);
            }
        }
    }
    
    fn setup_archive_selected(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_archive_selected(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(index) {
                    if db_path.exists() {
                        Self::reload_fonds_and_series(&db_path, &ui);
                    }
                }
            }
        });
    }
    
    /// Load initial fonds for an archive
    pub fn load_initial_fonds(db_path: &std::path::PathBuf, ui: &AppWindow) {
        if db_path.exists() {
            Self::reload_fonds_and_series(db_path, ui);
            // Reset additional selection states when loading initial data
            ui.set_selected_item(0);
        }
    }

    fn setup_open_file(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_open_file(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let path_str = ui.get_open_file_path().to_string();
                println!("Opening file: '{}'", path_str);
                if !path_str.is_empty() {
                    // Normalize path separators for the current OS
                    #[cfg(target_os = "windows")]
                    let normalized_path = path_str.replace("/", "\\");
                    #[cfg(not(target_os = "windows"))]
                    let normalized_path = path_str.clone();
                    
                    println!("Normalized path: '{}'", normalized_path);
                    
                    // Try to open the file with the system's default program
                    #[cfg(target_os = "windows")]
                    {
                        // Use explorer.exe for folders, start for files
                        let path = std::path::Path::new(&normalized_path);
                        if path.is_dir() {
                            let result = std::process::Command::new("explorer")
                                .arg(&normalized_path)
                                .spawn();
                            println!("Explorer result: {:?}", result);
                        } else {
                            let result = std::process::Command::new("cmd")
                                .args(["/C", "start", "", &normalized_path])
                                .spawn();
                            println!("Start result: {:?}", result);
                        }
                    }
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open")
                            .arg(&normalized_path)
                            .spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&normalized_path)
                            .spawn();
                    }
                } else {
                    println!("File path is empty!");
                }
            }
        });
    }

    fn setup_open_item(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_open_item(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let path_str = ui.get_open_item_path().to_string();
                println!("Opening item: '{}'", path_str);
                if !path_str.is_empty() {
                    // Normalize path separators for the current OS
                    #[cfg(target_os = "windows")]
                    let normalized_path = path_str.replace("/", "\\");
                    #[cfg(not(target_os = "windows"))]
                    let normalized_path = path_str.clone();
                    
                    println!("Normalized path: '{}'", normalized_path);
                    
                    // Try to open the file or folder with the system's default program
                    #[cfg(target_os = "windows")]
                    {
                        let path = std::path::Path::new(&normalized_path);
                        if path.is_dir() {
                            let result = std::process::Command::new("explorer")
                                .arg(&normalized_path)
                                .spawn();
                            println!("Explorer result: {:?}", result);
                        } else {
                            let result = std::process::Command::new("cmd")
                                .args(["/C", "start", "", &normalized_path])
                                .spawn();
                            println!("Start result: {:?}", result);
                        }
                    }
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open")
                            .arg(&normalized_path)
                            .spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&normalized_path)
                            .spawn();
                    }
                } else {
                    println!("Item path is empty!");
                }
            }
        });
    }
    
    /// Handle file activation (double-click or Enter) - load items for the file
    fn setup_file_activated(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_file_activated(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                // Set selected file and trigger file_selected to load items
                ui.set_selected_file(index);
                ui.invoke_file_selected(index);
            }
        });
    }
    
    /// Handle item activation (double-click or Enter) - do nothing, use quick action button to open
    fn setup_item_activated(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_item_activated(move |_index| {
            if let Some(_ui) = ui_weak.upgrade() {
                // Item click doesn't open the file, use the quick action button instead
            }
        });
    }
    
    /// Handle series activation (double-click or Enter) - could open series folder
    fn setup_series_activated(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_series_activated(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                // For now, just select the series
                let series_list = ui.get_series_list();
                if index >= 0 && (index as usize) < series_list.row_count() {
                    if let Some(series) = series_list.row_data(index as usize) {
                        ui.set_selected_series_index(index);
                        ui.set_selected_series_no(series.series_no.clone());
                        ui.invoke_select_series(index, series.series_no.clone());
                    }
                }
            }
        });
    }
    
    /// Open file at specific index (via quick action button)
    fn setup_open_file_at(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_open_file_at(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                let files = ui.get_files();
                if index >= 0 && (index as usize) < files.row_count() {
                    if let Some(file) = files.row_data(index as usize) {
                        let path = file.path.to_string();
                        if !path.is_empty() {
                            ui.set_open_file_path(path.into());
                            ui.invoke_open_file();
                        }
                    }
                }
            }
        });
    }
    
    /// Open item at specific index (via quick action button)
    fn setup_open_item_at(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        
        ui.on_open_item_at(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                let item_list = ui.get_file_item_list();
                if index >= 0 && (index as usize) < item_list.row_count() {
                    if let Some(item) = item_list.row_data(index as usize) {
                        let path = item.path.to_string();
                        if !path.is_empty() {
                            ui.set_open_item_path(path.into());
                            ui.invoke_open_item();
                        }
                    }
                }
            }
        });
    }
    
    fn setup_rebuild_series(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_rebuild_series(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let selected_archive = ui.get_selected_archive();
                let selected_fonds = ui.get_selected_fonds();
                
                if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_archive) {
                    if let Ok(fonds_list) = FondsService::list_fonds(&db_path) {
                        if (selected_fonds as usize) < fonds_list.len() {
                            let fond = &fonds_list[selected_fonds as usize];
                            
                            match FondsService::generate_series_for_fond(&db_path, &fond.fond_no, &fond.created_at) {
                                Ok(series_count) => {
                                    println!("Rebuilt {} series for fonds {}", series_count, fond.fond_no);
                                    
                                    // Reload series list
                                    match FondsService::list_series(&db_path, &fond.fond_no) {
                                        Ok(series_list) => {
                                            let series_items: Vec<crate::SeriesItem> = series_list.into_iter()
                                                .map(|s| crate::SeriesItem {
                                                    series_no: s.series_no.into(),
                                                    name: s.name.into(),
                                                    fond_no: s.fond_no.into(),
                                                })
                                                .collect();
                                            set_series_with_crud_items(series_items, &ui);
                                            
                                            // Reset series selection
                                            ui.set_selected_series_index(-1);
                                            ui.set_selected_series_no("".into());
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to reload series after rebuild: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to rebuild series for fonds {}: {}", fond.fond_no, e);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    fn setup_delete_series(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_delete_series(move || {
            if let Some(ui) = ui_weak.upgrade() {
                // Get the selected series
                let selected_series_index = ui.get_selected_series_index();
                if selected_series_index < 0 {
                    eprintln!("No series selected");
                    return;
                }
                
                // Get series info from the model
                let series_list = ui.get_series_list();
                if let Some(series_item) = series_list.row_data(selected_series_index as usize) {
                    let series_no_str = series_item.series_no.clone();
                    let series_name_str = series_item.name.clone();
                    
                    // Get the selected fonds for context
                    let selected_fonds = ui.get_selected_fonds();
                    let fonds_names = ui.get_fonds_names();
                    
                    // Create and show confirmation dialog
                    match crate::ConfirmDialog::new() {
                        Ok(dialog) => {
                            dialog.set_current_language(0);
                            let confirm_message = ui.get_confirm_delete_series().to_string();
                            dialog.set_message(confirm_message.replace("{}", &series_name_str).into());
                            
                            let archive_service = archive_service.clone();
                            let ui_weak2 = ui_weak.clone();
                            let series_no_str = series_no_str.clone();
                            let dialog_weak = dialog.as_weak();
                            
                            dialog.on_confirm(move || {
                                if let Some(ui) = ui_weak2.upgrade() {
                                    let selected_index = ui.get_selected_archive();
                                    
                                    match archive_service.get_database_path_by_index(selected_index) {
                                        Ok(Some(db_path)) => {
                                            if !db_path.exists() {
                                                eprintln!("Database not found for selected archive");
                                                return;
                                            }
                                            
                                            // First check if series has files
                                            match FondsService::list_files_by_series(&db_path, &series_no_str) {
                                                Ok(files) => {
                                                    if !files.is_empty() {
                                                        let error_message = ui.get_cannot_delete_series_has_files().to_string();
                                                        ui.invoke_show_toast(error_message.into());
                                                        return;
                                                    }
                                                    
                                                    // Delete the series
                                                    match FondsService::delete_series(&db_path, &series_no_str) {
                                                        Ok(deleted) => {
                                                            if deleted {
                                                                println!("Series deleted: {}", series_no_str);
                                                                
                                                                // Reload series for the current fonds
                                                                let selected_fonds = ui.get_selected_fonds();
                                                                let fonds_names = ui.get_fonds_names();
                                                                if (selected_fonds as usize) < fonds_names.row_count() {
                                                                    if let Some(fonds_name) = fonds_names.row_data(selected_fonds as usize) {
                                                                        // Extract fond_no from fonds name (format: "fond_no - name")
                                                                        let fond_no = fonds_name.split(" - ").next().unwrap_or(&fonds_name);
                                                                        
                                                                        match FondsService::list_series(&db_path, fond_no) {
                                                                            Ok(series_list) => {
                                                                                let series_items: Vec<crate::SeriesItem> = series_list.into_iter()
                                                                                    .map(|s| crate::SeriesItem {
                                                                                        series_no: s.series_no.into(),
                                                                                        name: s.name.into(),
                                                                                        fond_no: s.fond_no.into(),
                                                                                    })
                                                                                    .collect();
                                                                                set_series_with_crud_items(series_items, &ui);
                                                                                
                                                                                // Reset series selection
                                                                                ui.set_selected_series_index(-1);
                                                                                ui.set_selected_series_no("".into());
                                                                                
                                                                                // Clear files and items
                                                                                clear_files(&ui);
                                                                                clear_items(&ui);
                                                                            }
                                                                            Err(e) => {
                                                                                eprintln!("Failed to reload series after deletion: {}", e);
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                
                                                                // Close the dialog
                                                                if let Some(d) = dialog_weak.upgrade() {
                                                                    let _ = d.hide();
                                                                }
                                                            } else {
                                                                ui.invoke_show_toast("系列不存在".into());
                                                            }
                                                        }
                                                        Err(e) => {
                                                            ui.invoke_show_toast(format!("删除系列失败: {}", e).into());
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    ui.invoke_show_toast(format!("检查系列文件失败: {}", e).into());
                                                }
                                            }
                                        }
                                        Ok(None) => eprintln!("No archive selected"),
                                        Err(e) => eprintln!("Failed to get database path: {}", e),
                                    }
                                }
                            });
                            
                            let dialog_weak = dialog.as_weak();
                            dialog.on_cancel(move || {
                                println!("Delete series cancelled");
                                if let Some(d) = dialog_weak.upgrade() {
                                    let _ = d.hide();
                                }
                            });
                            
                            // Show the dialog
                            if let Err(e) = dialog.show() {
                                eprintln!("Failed to show delete confirmation dialog: {}", e);
                            }
                        }
                        Err(e) => eprintln!("Failed to create confirmation dialog: {}", e),
                    }
                } else {
                    eprintln!("Invalid series selection");
                }
            }
        });
    }
}
