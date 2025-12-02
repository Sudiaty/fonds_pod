// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 从环境变量读取版本号（由GitHub Actions注入）
const APP_VERSION: &str = env!("APP_VERSION");

// New layered architecture
mod domain;
mod infrastructure;
mod services;
mod presentation;

use std::error::Error;
use std::rc::Rc;
use infrastructure::{FileConfigRepository};
use services::{ArchiveService, FileService};
use presentation::{SchemaHandler, ArchiveHandler, SettingsHandler, ClassificationHandler, FondsHandler};

use tray_item::{TrayItem, IconSource};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize configuration repository first to load language settings
    let config_repo = FileConfigRepository::new();
    let temp_archive_service = ArchiveService::new(config_repo);

    // Initialize application
    println!("FondsPod starting...");
    
    let ui = AppWindow::new().map_err(|e| { eprintln!("UI create failed: {}", e); e })?;

    // Set application version
    ui.set_app_version(APP_VERSION.into());

    // Load language setting and set UI, then select bundled translation
    if let Ok(settings) = temp_archive_service.get_settings() {
        let language_index = if settings.language == "zh_CN" { 0 } else { 1 };
        ui.set_selected_language(language_index);
        
        // Select bundled translation based on settings
        if !settings.language.is_empty() {
            let _ = slint::select_bundled_translation(&settings.language);
        }
    }

    // Initialize services (Dependency Injection) - reuse the config_repo
    let archive_service = Rc::new(temp_archive_service);
    
    // Initialize presentation handlers
    let archive_handler = ArchiveHandler::new(archive_service.clone());
    let schema_handler = SchemaHandler::new(archive_service.clone());
    let classification_handler = ClassificationHandler::new(archive_service.clone());
    let settings_handler = SettingsHandler::new(archive_service.clone());
    let file_service = Rc::new(FileService);
    let fonds_handler = FondsHandler::new(archive_service.clone(), file_service);
    
    // Initialize UI state
    archive_handler.initialize(&ui)?;
    
    // Load initial schemas if an archive is selected
    let selected_index = ui.get_selected_archive();
    if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_index) {
        SchemaHandler::<FileConfigRepository>::load_initial_schemas(&db_path, &ui);
        ClassificationHandler::<FileConfigRepository>::load_initial_classifications(&db_path, &ui);
        FondsHandler::<FileConfigRepository>::load_initial_fonds(&db_path, &ui);
    }
    
    // Setup all UI callbacks through presentation layer
    schema_handler.setup_callbacks(&ui);
    classification_handler.setup_callbacks(&ui);
    archive_handler.setup_callbacks(&ui);
    settings_handler.setup_callbacks(&ui);
    fonds_handler.setup_callbacks(&ui);
    
    // Get translated strings for tray menu
    let tray_show_text = ui.get_tray_show_window().to_string();
    let tray_quit_text = ui.get_tray_quit().to_string();

    // Create tray icon - use Resource with the name defined in resources.rc
    // "IDI_TRAYICON" is the resource identifier
    let ui_weak_for_tray = ui.as_weak();
    if let Ok(mut tray) = TrayItem::new("FondsPod", IconSource::Resource("IDI_TRAYICON")) {
        tray.add_menu_item(&tray_show_text, move || {
            // Show window by invoking from main thread
            if let Some(ui) = ui_weak_for_tray.upgrade() {
                ui.window().show().unwrap();
            }
        }).unwrap();
        tray.add_menu_item(&tray_quit_text, || {
            std::process::exit(0);
        }).unwrap();
        // Keep tray alive by not letting it drop
        std::mem::forget(tray);
    } else {
        eprintln!("Failed to create tray icon");
    }

    // Handle window close to minimize to tray (hide window)
    ui.window().on_close_requested(|| {
        // Hide window instead of closing
        slint::CloseRequestResponse::HideWindow
    });
    
    // Handle page changes to refresh data when switching to any page
    let archive_service_clone = archive_service.clone();
    let ui_weak_for_page_change = ui.as_weak();
    ui.on_page_changed(move |page_id| {
        if let Some(ui) = ui_weak_for_page_change.upgrade() {
            let selected_archive = ui.get_selected_archive();
            if let Ok(Some(db_path)) = archive_service_clone.get_database_path_by_index(selected_archive) {
                match page_id.as_str() {
                    "fonds" => {
                        FondsHandler::<FileConfigRepository>::load_initial_fonds(&db_path, &ui);
                    }
                    "schema" => {
                        SchemaHandler::<FileConfigRepository>::load_initial_schemas(&db_path, &ui);
                    }
                    "classification" => {
                        ClassificationHandler::<FileConfigRepository>::load_initial_classifications(&db_path, &ui);
                    }
                    _ => {
                        // Other pages don't need database-dependent initialization
                    }
                }
            }
        }
    });
    
    // Handle archive selection changes - refresh data for current page if needed
    let fonds_handler_clone = fonds_handler.clone();
    let archive_service_clone_for_archive = archive_service.clone();
    let ui_weak_for_archive = ui.as_weak();
    ui.on_archive_selected(move |archive_index| {
        if let Some(ui) = ui_weak_for_archive.upgrade() {
            // Always clear and reload fonds data when archive changes
            fonds_handler_clone.refresh_fonds_data_for_archive(archive_index, &ui);
            // Also refresh other data
            if let Ok(Some(db_path)) = archive_service_clone_for_archive.get_database_path_by_index(archive_index) {
                SchemaHandler::<FileConfigRepository>::load_initial_schemas(&db_path, &ui);
                ClassificationHandler::<FileConfigRepository>::load_initial_classifications(&db_path, &ui);
            }
        }
    });
    
    ui.run()?;
    
    Ok(())
}
