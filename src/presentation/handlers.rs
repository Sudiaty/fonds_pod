/// UI event handlers - connects UI callbacks to application services
use crate::services::{SchemaService, ArchiveService, ClassificationService};
use crate::domain::ConfigRepository;
use std::rc::Rc;

// Import AppWindow and ComponentHandle trait from parent module (main.rs)
use crate::AppWindow;
use slint::{ComponentHandle, Model};

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
                        let current_language = ui.invoke_get_language();
                        dialog.set_current_language(current_language);
                        
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
                                                    format!("分类方案代码 '{}' 已存在", schema_code_str)
                                                } else {
                                                    format!("创建失败: {}", e)
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
                    .set_title("确认删除")
                    .set_description(&format!("确定要删除分类方案 '{}' 吗？", schema_no))
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
                                ui.invoke_show_toast("无法删除该分类方案，可能已被使用或受保护".into());
                            }
                            Err(e) => {
                                ui.invoke_show_toast(format!("删除失败: {}", e).into());
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
                    .set_title("确认删除")
                    .set_description(&format!("确定要删除选中的 {} 个分类方案吗？", count))
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
                                            ui.invoke_show_toast(format!("无法删除 '{}'，可能已被使用或受保护", schema_no).into());
                                        }
                                        Err(e) => {
                                            ui.invoke_show_toast(format!("删除 '{}' 失败: {}", schema_no, e).into());
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
                        let current_language = ui.invoke_get_language();
                        dialog.set_current_language(current_language);
                        
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
                                                    format!("分类项代码 '{}' 已存在", item_code_str)
                                                } else {
                                                    format!("添加失败: {}", e)
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
                    .set_title("确认删除")
                    .set_description(&format!("确定要删除分类项 '{}' 吗？", item_no))
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
                            .set_title("确认删除")
                            .set_description(&format!("确定要删除选中的 {} 个分类项吗？", count))
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
                                .set_title("确认删除")
                                .set_description(&format!("确定要删除档案库 '{}' 吗？", library_name))
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
                        let current_language = ui.invoke_get_language();
                        dialog.set_current_language(current_language);
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
        
        // Set language
        let language_index = if settings.language == "en" { 1 } else { 0 };
        ui.invoke_set_language(language_index);
        
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
                let language_index = ui.invoke_get_language();
                let language = if language_index == 1 { "en" } else { "zh" };
                
                if let Err(e) = archive_service.set_language(language.to_string()) {
                    eprintln!("Failed to save language: {}", e);
                }
            }
        });
    }
    
    fn setup_cancel_settings(&self, ui: &AppWindow) {
        let ui_weak = ui.as_weak();
        let archive_service = self.archive_service.clone();
        
        ui.on_cancel_settings(move || {
            if let Some(ui) = ui_weak.upgrade() {
                match archive_service.get_settings() {
                    Ok(settings) => {
                        let language_index = if settings.language == "en" { 1 } else { 0 };
                        ui.invoke_set_language(language_index);
                    }
                    Err(e) => eprintln!("Failed to load settings: {}", e),
                }
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
                        dialog.set_current_language(ui.invoke_get_language());
                        let archive_service = archive_service.clone(); let ui_weak2 = ui_weak.clone(); let dialog_weak = dialog.as_weak();
                        dialog.on_confirm_input(move |code: slint::SharedString, name: slint::SharedString| {
                            if let Some(ui) = ui_weak2.upgrade() {
                                let code_s = code.to_string(); let name_s = name.to_string(); if code_s.is_empty() || name_s.is_empty() { return; }
                                let sel_index = ui.get_selected_archive();
                                match archive_service.get_database_path_by_index(sel_index) {
                                    Ok(Some(db_path)) => {
                                        match ClassificationService::create_top(&db_path, code_s.clone(), name_s) {
                                            Ok(_) => { Self::reload_top(&db_path, &ui); if let Some(d) = dialog_weak.upgrade() { let _ = d.hide(); } }
                                            Err(e) => { let msg = if e.to_string().contains("UNIQUE constraint failed") { format!("分类代码 '{}' 已存在", code_s) } else { format!("创建失败: {}", e) }; ui.invoke_show_toast(msg.into()); }
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
                    let confirmed = rfd::MessageDialog::new().set_title("确认删除").set_description(&format!("确定要删除分类 '{}' 吗？", code)).set_buttons(rfd::MessageButtons::YesNo).show();
                    if confirmed != rfd::MessageDialogResult::Yes { return; }
                    match archive_service.get_database_path_by_index(sel_index) {
                        Ok(Some(db_path)) => {
                            match ClassificationService::delete(&db_path, &code) {
                                Ok(true) => { Self::reload_top(&db_path, &ui); }
                                Ok(false) => ui.invoke_show_toast(format!("无法删除分类 '{}': 有子项或被引用", code).into()),
                                Err(e) => ui.invoke_show_toast(format!("删除失败: {}", e).into()),
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
                let confirmed = rfd::MessageDialog::new().set_title("确认删除").set_description(&format!("确定删除选中的 {} 个分类?", count)).set_buttons(rfd::MessageButtons::YesNo).show(); if confirmed != rfd::MessageDialogResult::Yes { return; }
                match archive_service.get_database_path_by_index(sel_index) {
                    Ok(Some(db_path)) => {
                        let mut deleted = 0;
                        for i in 0..selections.row_count() { if selections.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); match ClassificationService::delete(&db_path, &code) { Ok(true) => deleted += 1, Ok(false) => ui.invoke_show_toast(format!("无法删除 '{}': 被引用或有子项", code).into()), Err(e) => ui.invoke_show_toast(format!("删除 '{}' 失败: {}", code, e).into()) } } } }
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
                            dialog.set_current_language(ui.invoke_get_language()); dialog.set_parent_display(parent_display.into());
                            let archive_service = archive_service.clone(); let ui_weak2 = ui_weak.clone(); let dialog_weak = dialog.as_weak(); let parent_code_clone = parent_code.clone();
                            dialog.on_confirm_input(move |code: slint::SharedString, name: slint::SharedString| {
                                if let Some(ui) = ui_weak2.upgrade() { let code_s = code.to_string(); let name_s = name.to_string(); if code_s.is_empty() || name_s.is_empty() { return; }
                                    let sel_index = ui.get_selected_archive(); match archive_service.get_database_path_by_index(sel_index) { Ok(Some(db_path)) => {
                                        match ClassificationService::create_child(&db_path, parent_code_clone.clone(), code_s.clone(), name_s) {
                                            Ok(_) => { Self::reload_children(&db_path, &parent_code_clone, &ui); if let Some(d) = dialog_weak.upgrade() { let _ = d.hide(); } }
                                            Err(e) => { let msg = if e.to_string().contains("UNIQUE constraint failed") { format!("分类代码 '{}' 已存在", code_s) } else { format!("添加失败: {}", e) }; ui.invoke_show_toast(msg.into()); }
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
                    let confirmed = rfd::MessageDialog::new().set_title("确认删除").set_description(&format!("确定要删除二级分类 '{}' 吗？", code)).set_buttons(rfd::MessageButtons::YesNo).show();
                    if confirmed != rfd::MessageDialogResult::Yes { return; }
                    let sel_arch = ui.get_selected_archive(); match archive_service.get_database_path_by_index(sel_arch) { Ok(Some(db_path)) => {
                        match ClassificationService::delete(&db_path, &code) { Ok(true) => { Self::reload_children(&db_path, &parent_code, &ui); } Ok(false) => ui.invoke_show_toast(format!("无法删除分类 '{}': 有子项或被引用", code).into()), Err(e) => ui.invoke_show_toast(format!("删除失败: {}", e).into()) }
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
                let confirmed = rfd::MessageDialog::new().set_title("确认删除").set_description(&format!("确定删除选中的 {} 个二级分类?", count)).set_buttons(rfd::MessageButtons::YesNo).show(); if confirmed != rfd::MessageDialogResult::Yes { return; }
                let sel_arch = ui.get_selected_archive(); let parent_code = parents.row_data(parent_idx as usize).unwrap().code.to_string();
                match archive_service.get_database_path_by_index(sel_arch) {
                    Ok(Some(db_path)) => {
                        let mut deleted = 0; for i in 0..selections.row_count() { if selections.row_data(i)==Some(1) { if let Some(child) = children.row_data(i) { let code = child.code.to_string(); match ClassificationService::delete(&db_path, &code) { Ok(true)=>deleted+=1, Ok(false)=>ui.invoke_show_toast(format!("无法删除 '{}': 有子项或被引用", code).into()), Err(e)=>ui.invoke_show_toast(format!("删除 '{}' 失败: {}", code, e).into()) } } } }
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
                            for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(format!("激活 '{}' 失败: {}", code, e).into()); } } } }
                            Self::reload_top(&db_path, &ui); ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let current_index = ui.get_selected_classification(); let list = ui.get_classification_items();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(format!("激活 '{}' 失败: {}", code, e).into()); } else { Self::reload_top(&db_path, &ui); } } }
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
                            for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(format!("停用 '{}' 失败: {}", code, e).into()); } } } }
                            Self::reload_top(&db_path, &ui); ui.set_selected_classifications(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let current_index = ui.get_selected_classification(); let list = ui.get_classification_items();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(format!("停用 '{}' 失败: {}", code, e).into()); } else { Self::reload_top(&db_path, &ui); } } }
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
                            for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(format!("激活 '{}' 失败: {}", code, e).into()); } } } }
                            Self::reload_children(&db_path, &parent_code, &ui); ui.set_selected_children(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let current_index = ui.get_selected_child(); let list = ui.get_classification_children();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::activate(&db_path, &code) { ui.invoke_show_toast(format!("激活 '{}' 失败: {}", code, e).into()); } else { Self::reload_children(&db_path, &parent_code, &ui); } } }
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
                            for i in 0..selected.row_count() { if selected.row_data(i) == Some(1) { if let Some(item) = items.row_data(i) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(format!("停用 '{}' 失败: {}", code, e).into()); } } } }
                            Self::reload_children(&db_path, &parent_code, &ui); ui.set_selected_children(slint::ModelRc::new(slint::VecModel::<i32>::default()));
                        } else {
                            let current_index = ui.get_selected_child(); let list = ui.get_classification_children();
                            if current_index >= 0 && (current_index as usize) < list.row_count() { if let Some(item) = list.row_data(current_index as usize) { let code = item.code.to_string(); if let Err(e) = ClassificationService::deactivate(&db_path, &code) { ui.invoke_show_toast(format!("停用 '{}' 失败: {}", code, e).into()); } else { Self::reload_children(&db_path, &parent_code, &ui); } } }
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
                                        ui.invoke_show_toast(format!("导出失败: {}", e).into());
                                    } else {
                                        ui.invoke_show_toast("导出成功".into());
                                    }
                                }
                            }
                            Err(e) => ui.invoke_show_toast(format!("导出失败: {}", e).into()),
                        }
                    }
                    _ => ui.invoke_show_toast("未选择档案库".into()),
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
                                            ui.invoke_show_toast("导入成功".into());
                                        }
                                        Err(e) => ui.invoke_show_toast(format!("导入失败: {}", e).into()),
                                    }
                                }
                                Err(e) => ui.invoke_show_toast(format!("读取文件失败: {}", e).into()),
                            }
                        }
                    }
                    _ => ui.invoke_show_toast("未选择档案库".into()),
                }
            }
        });
    }

    fn reload_top(db_path: &std::path::PathBuf, ui: &AppWindow) {
        match ClassificationService::list_top(db_path) {
            Ok(list) => {
                let items: Vec<crate::ClassificationItem> = list.iter().map(|c| crate::ClassificationItem { code: c.code.as_str().into(), name: c.name.as_str().into(), is_active: c.is_active }).collect();
                let model = slint::ModelRc::new(slint::VecModel::from(items)); ui.set_classification_items(model);
                if let Some(first) = list.first() { Self::reload_children(db_path, &first.code, ui); }
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

    pub fn load_initial_classifications(db_path: &std::path::PathBuf, ui: &AppWindow) { if db_path.exists() { Self::reload_top(db_path, ui); } }
}
