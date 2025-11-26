/// UI event handlers - connects UI callbacks to application services
use crate::services::{SchemaService, ArchiveService, ClassificationService, FondsService, FileService, CreateFileInput, CreateFondsInput};
use crate::domain::ConfigRepository;
use crate::infrastructure::persistence::queries;
use std::rc::Rc;
use std::fs;
use std::path::Path;

// Import AppWindow and ComponentHandle trait from parent module (main.rs)
use crate::AppWindow;
use crate::FileInputDialog;
use crate::ConfirmDialog;
use slint::{ComponentHandle, Model, VecModel};

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
        self.setup_schema_item_clicked(ui);
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
            if let Some(ui) = ui_weak.upgrade() {
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
                    .set_title(&ui.get_confirm_delete())
                    .set_description(&ui.get_confirm_delete_classification().replace("{}", &schema_no))
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
                    if selected_schemas.row_data(i as usize) == Some(1) {
                        count += 1;
                    }
                }
                
                // Show confirmation dialog
                let confirmed = rfd::MessageDialog::new()
                    .set_title(&ui.get_confirm_delete())
                    .set_description(&ui.get_confirm_delete_selected_classifications().replace("{}", &count.to_string()))
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
                            if selected_schemas.row_data(i as usize) == Some(1) {
                                if let Some(schema_item) = schema_items.row_data(i as usize) {
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
                
                match archive_service.get_database_path_by_index(selected_index) {
                    Ok(Some(db_path)) => {
                        if db_path.exists() {
                            Self::reload_schema_items(&db_path, &schema_no, &ui);
                        }
                    }
                    _ => {}
                }
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
                        
                        match archive_service.get_database_path_by_index(selected_archive) {
                            Ok(Some(db_path)) => {
                                if db_path.exists() {
                                    Self::reload_schema_items(&db_path, &schema_no, &ui);
                                }
                            }
                            _ => {}
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
                    .set_title(&ui.get_confirm_delete())
                    .set_description(&ui.get_confirm_delete_classification().replace("{}", &item_no))
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
                            .set_title(&ui.get_confirm_delete())
                            .set_description(&ui.get_confirm_delete_selected_classifications().replace("{}", &count.to_string()))
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
                ui.invoke_load_schemas(slint::ModelRc::from(model).into());
                
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
                ui.invoke_load_schema_items(slint::ModelRc::from(model).into());
                
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
                    let path_str = path.to_string_lossy().to_string().into();
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
                                .set_title(&ui.get_confirm_delete())
                                .set_description(&ui.get_confirm_delete_classification().replace("{}", &library_name))
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
            if let Some(ui) = ui_weak.upgrade() {
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
        let archive_service = self.archive_service.clone();
        
        ui.on_cancel_settings(move || {
            if let Some(ui) = ui_weak.upgrade() {
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
            if let Some(ui) = ui_weak.upgrade() {
                match crate::ClassificationInputDialog::new() {
                    Ok(dialog) => {
                        dialog.set_current_language(0);
                        let archive_service = archive_service.clone(); let ui_weak2 = ui_weak.clone(); let dialog_weak = dialog.as_weak();
                        dialog.on_confirm_input(move |code: slint::SharedString, name: slint::SharedString| {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let code_s = code.to_string(); let name_s = name.to_string(); if code_s.is_empty() || name_s.is_empty() { return; }
                                let sel_index = ui.get_selected_archive();
                                match archive_service.get_database_path_by_index(sel_index) {
                                    Ok(Some(db_path)) => {
                                        match ClassificationService::create_top(&db_path, code_s.clone(), name_s) {
                                            Ok(_) => { Self::reload_top(&db_path, &ui); if let Some(d) = dialog_weak.upgrade() { let _ = d.hide(); } }
                                            Err(e) => { let msg = if e.to_string().contains("UNIQUE constraint failed") { ui.get_code_exists().replace("{}", &code_s) } else { ui.get_create_failed().replace("{}", &e.to_string()) }; ui.invoke_show_toast(msg.into()); }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        });
                        let dialog_weak2 = dialog.as_weak(); dialog.on_cancel_input(move || { if let Some(d) = dialog_weak2.upgrade() { let _ = d.hide(); } }); let _ = dialog.show();
                    }
                    Err(e) => eprintln!("Failed to create classification dialog: {}", e),
                }
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
                    let confirmed = rfd::MessageDialog::new().set_title(&ui.get_confirm_delete()).set_description(&ui.get_confirm_delete_classification().replace("{}", &code)).set_buttons(rfd::MessageButtons::YesNo).show();
                    if confirmed != rfd::MessageDialogResult::Yes { return; }
                    match archive_service.get_database_path_by_index(sel_index) {
                        Ok(Some(db_path)) => {
                            match ClassificationService::delete(&db_path, &code) {
                                Ok(true) => { Self::reload_top(&db_path, &ui); }
                                Ok(false) => ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &code).into()),
                                Err(e) => ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &e.to_string()).into()),
                            }
                        }
                        _ => {}
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
                let confirmed = rfd::MessageDialog::new().set_title(&ui.get_confirm_delete()).set_description(&ui.get_confirm_delete_selected_classifications().replace("{}", &count.to_string())).set_buttons(rfd::MessageButtons::YesNo).show(); if confirmed != rfd::MessageDialogResult::Yes { return; }
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        let mut deleted = 0;
                        for i in 0..selections.row_count() { if selections.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); match ClassificationService::delete(&db_path, &code) { Ok(true) => deleted += 1, Ok(false) => ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &code).into()), Err(e) => ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &e.to_string()).into()) } } } }
                        if deleted > 0 { Self::reload_top(&db_path, &ui); ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::<i32>::default())); }
                    }
                    _ => {}
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
                if let Some(item) = items_model.row_data(index as usize) { let code = item.code.to_string(); let sel_arch = ui.get_selected_archive(); match archive_service.get_database_path_by_index(sel_arch) { Ok(Some(db_path)) => { if db_path.exists() { Self::reload_children(&db_path, &code, &ui); } } _ => {} } }
            }
        });
    }

    fn setup_classification_selected(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_classification_selected(move |_i: i32| {
            if let Some(ui) = ui_weak.upgrade() { let idx = ui.get_selected_classification(); let items = ui.get_classification_items(); if idx >=0 && (idx as usize) < items.row_count() { if let Some(item) = items.row_data(idx as usize) { let code = item.code.to_string(); let sel_arch = ui.get_selected_archive(); match archive_service.get_database_path_by_index(sel_arch) { Ok(Some(db_path)) => { if db_path.exists() { Self::reload_children(&db_path, &code, &ui); } } _ => {} } } } }
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
                                    let sel_index = ui.get_selected_archive(); match archive_service.get_database_path_by_index(sel_index) { Ok(Some(db_path)) => {
                                        match ClassificationService::create_child(&db_path, parent_code_clone.clone(), code_s.clone(), name_s) {
                                            Ok(_) => { Self::reload_children(&db_path, &parent_code_clone, &ui); if let Some(d) = dialog_weak.upgrade() { let _ = d.hide(); } }
                                            Err(e) => { let msg = if e.to_string().contains("UNIQUE constraint failed") { ui.get_code_exists().replace("{}", &code_s) } else { ui.get_create_failed().replace("{}", &e.to_string()) }; ui.invoke_show_toast(msg.into()); }
                                        }
                                    } _ => {} }
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
                    let confirmed = rfd::MessageDialog::new().set_title(&ui.get_confirm_delete()).set_description(&ui.get_confirm_delete_child_classification().replace("{}", &code)).set_buttons(rfd::MessageButtons::YesNo).show();
                    if confirmed != rfd::MessageDialogResult::Yes { return; }
                    let sel_arch = ui.get_selected_archive(); match archive_service.get_database_path_by_index(sel_arch) { Ok(Some(db_path)) => {
                        match ClassificationService::delete(&db_path, &code) { Ok(true) => { Self::reload_children(&db_path, &parent_code, &ui); } Ok(false) => ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &code).into()), Err(e) => ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &e.to_string()).into()) }
                    } _ => {} }
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
                let confirmed = rfd::MessageDialog::new().set_title(&ui.get_confirm_delete()).set_description(&ui.get_confirm_delete_selected_children().replace("{}", &count.to_string())).set_buttons(rfd::MessageButtons::YesNo).show(); if confirmed != rfd::MessageDialogResult::Yes { return; }
                let sel_arch = ui.get_selected_archive(); let parent_code = parents.row_data(parent_idx as usize).unwrap().code.to_string();
                match archive_service.get_database_path_by_index(sel_arch) {
                    Ok(Some(db_path)) => {
                        let mut deleted = 0; for i in 0..selections.row_count() { if selections.row_data(i)==Some(1) { if let Some(child) = children.row_data(i) { let code = child.code.to_string(); match ClassificationService::delete(&db_path, &code) { Ok(true)=>deleted+=1, Ok(false)=>ui.invoke_show_toast(ui.get_cannot_delete().replace("{}", &code).into()), Err(e)=>ui.invoke_show_toast(ui.get_delete_failed().replace("{}", &format!("'{}': {}", code, e)).into()) } } } }
                        if deleted>0 { Self::reload_children(&db_path, &parent_code, &ui); ui.set_selected_children(slint::ModelRc::new(slint::VecModel::<i32>::default())); }
                    }
                    _ => {}
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
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive();
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        if ui.get_selected_classifications().row_count() > 0 {
                            let selected = ui.get_selected_classifications(); let items = ui.get_classification_items();
                            for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(ui.get_activate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } } } }
                            Self::reload_top(&db_path, &ui); ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let current_index = ui.get_selected_classification(); let list = ui.get_classification_items();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(ui.get_activate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } else { Self::reload_top(&db_path, &ui); } } }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    fn setup_deactivate_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_deactivate_classification(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive();
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        if ui.get_selected_classifications().row_count() > 0 {
                            let selected = ui.get_selected_classifications(); let items = ui.get_classification_items();
                            for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(ui.get_deactivate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } } } }
                            Self::reload_top(&db_path, &ui); ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let current_index = ui.get_selected_classification(); let list = ui.get_classification_items();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(ui.get_deactivate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } else { Self::reload_top(&db_path, &ui); } } }
                        }
                    }
                    _ => {}
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
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        if ui.get_selected_children().row_count() > 0 {
                            let selected = ui.get_selected_children(); let items = ui.get_classification_children();
                            for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(ui.get_activate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } } } }
                            Self::reload_children(&db_path, &parent_code, &ui); ui.set_selected_children(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let current_index = ui.get_selected_child(); let list = ui.get_classification_children();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(ui.get_activate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } else { Self::reload_children(&db_path, &parent_code, &ui); } } }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    fn setup_deactivate_child_classification(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak(); let archive_service = self.archive_service.clone();
        ui.on_deactivate_child_classification(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let sel_index = ui.get_selected_archive(); let parent_idx = ui.get_selected_classification(); let parents = ui.get_classification_items(); if parent_idx < 0 || parent_idx as usize >= parents.row_count() { return; }
                let parent_code = parents.row_data(parent_idx as usize).unwrap().code.to_string();
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        if ui.get_selected_children().row_count() > 0 {
                            let selected = ui.get_selected_children(); let items = ui.get_classification_children();
                            for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(ui.get_deactivate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } } } }
                            Self::reload_children(&db_path, &parent_code, &ui); ui.set_selected_children(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let current_index = ui.get_selected_child(); let list = ui.get_classification_children();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(ui.get_deactivate_failed().replace("{}", &code).replace("{}", &e.to_string()).into()); } else { Self::reload_children(&db_path, &parent_code, &ui); } } }
                        }
                    }
                    _ => {}
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
                if let Some(first) = list.first() { 
                    Self::reload_children(db_path, &first.code, ui); 
                } else {
                    // If no top classifications, clear children as well
                    let empty_children: Vec<crate::ClassificationItem> = Vec::new();
                    let children_model = slint::ModelRc::new(slint::VecModel::from(empty_children));
                    ui.set_classification_children(children_model);
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
    fonds_service: Rc<FondsService>,
    file_service: Rc<FileService>,
}

impl<CR: ConfigRepository + 'static> FondsHandler<CR> {
    pub fn new(archive_service: Rc<ArchiveService<CR>>, fonds_service: Rc<FondsService>, file_service: Rc<FileService>) -> Self {
        Self { 
            archive_service, 
            fonds_service,
            file_service,
        }
    }
    
    /// Setup fonds callbacks for the UI
    pub fn setup_callbacks(&self, ui: &AppWindow) {
        self.setup_add_fonds_dialog(ui);
        self.setup_add_file(ui);
        self.setup_delete_file(ui);
        self.setup_delete_selected_files(ui);
        self.setup_file_clicked(ui);
        self.setup_select_series(ui);
        self.setup_file_selected(ui);
        self.setup_fonds_selected(ui);
        self.setup_archive_selected(ui);
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
                        let file_service = file_service.clone();
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
                    let file_no_str = file_item.file_no.clone();
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
                                                            if let Some(file_item) = files_model.row_data(i as usize) {
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
                                    let item_names: Vec<slint::SharedString> = items.into_iter()
                                        .map(|i| i.name.into())
                                        .collect();
                                    let items_model = slint::ModelRc::new(slint::VecModel::from(item_names));
                                    ui.set_item_list(items_model);
                                    
                                    // Reset item selection
                                    ui.set_selected_item(0);
                                }
                                Err(e) => {
                                    eprintln!("Failed to load items for file {}: {}", file_no_str, e);
                                    let empty_items: Vec<slint::SharedString> = Vec::new();
                                    let items_model = slint::ModelRc::new(slint::VecModel::from(empty_items));
                                    ui.set_item_list(items_model);
                                    ui.set_selected_item(-1);
                                }
                            }
                        }
                    }
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
                                    let series_model = slint::ModelRc::new(slint::VecModel::from(series_items));
                                    ui.set_series_list(series_model);
                                    
                                    // Reset series selection
                                    ui.set_selected_series_index(-1);
                                    ui.set_selected_series_no("".into());
                                }
                                Err(e) => {
                                    eprintln!("Failed to load series for fonds {}: {}", selected_fonds.fond_no, e);
                                    let empty_series: Vec<crate::SeriesItem> = Vec::new();
                                    let series_model = slint::ModelRc::new(slint::VecModel::from(empty_series));
                                    ui.set_series_list(series_model);
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

            let empty_series: Vec<crate::SeriesItem> = Vec::new();
            let series_model = slint::ModelRc::new(slint::VecModel::from(empty_series));
            ui.set_series_list(series_model);

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
                        let series_model = slint::ModelRc::new(slint::VecModel::from(series_items));
                        ui.set_series_list(series_model);
                    }
                    Err(e) => {
                        eprintln!("Failed to load series: {}", e);
                        let empty_series: Vec<crate::SeriesItem> = Vec::new();
                        let series_model = slint::ModelRc::new(slint::VecModel::from(empty_series));
                        ui.set_series_list(series_model);
                    }
                }
            } else {
                // No fonds, clear series
                let empty_series: Vec<crate::SeriesItem> = Vec::new();
                let series_model = slint::ModelRc::new(slint::VecModel::from(empty_series));
                ui.set_series_list(series_model);
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
                let files_model = slint::ModelRc::new(slint::VecModel::from(file_items));
                ui.set_files(files_model);
                
                // Reset file selection and trigger file_selected
                ui.set_selected_file(0);
                ui.invoke_file_selected(0);
            }
            Err(e) => {
                eprintln!("Failed to load files for series {}: {}", series_no, e);
                let empty_files: Vec<crate::FileItem> = Vec::new();
                let files_model = slint::ModelRc::new(slint::VecModel::from(empty_files));
                ui.set_files(files_model);
                ui.set_selected_file(-1);
                // Clear items
                let empty_items: Vec<slint::SharedString> = Vec::new();
                let items_model = slint::ModelRc::new(slint::VecModel::from(empty_items));
                ui.set_item_list(items_model);
                ui.set_selected_item(-1);
            }
        }
    }
    
    fn setup_archive_selected(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_archive_selected(move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                match archive_service.get_database_path_by_index(index) {
                    Ok(Some(db_path)) => {
                        if db_path.exists() {
                            Self::reload_fonds_and_series(&db_path, &ui);
                        }
                    }
                    _ => {}
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
}
