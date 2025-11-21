// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// New layered architecture
mod domain;
mod infrastructure;
mod services;
mod presentation;

use std::error::Error;
use std::rc::Rc;
use infrastructure::{FileConfigRepository};
use services::ArchiveService;
use presentation::{SchemaHandler, ArchiveHandler, SettingsHandler, ClassificationHandler};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize application
    println!("FondsPod starting...");
    
    let ui = AppWindow::new()?;
    
    // Initialize services (Dependency Injection)
    let config_repo = FileConfigRepository::new();
    let archive_service = Rc::new(ArchiveService::new(config_repo));
    
    // Initialize presentation handlers
    let archive_handler = ArchiveHandler::new(archive_service.clone());
    let schema_handler = SchemaHandler::new(archive_service.clone());
    let classification_handler = ClassificationHandler::new(archive_service.clone());
    let settings_handler = SettingsHandler::new(archive_service.clone());
    
    // Initialize UI state
    archive_handler.initialize(&ui)?;
    
    // Load initial schemas if an archive is selected
    let selected_index = ui.get_selected_archive();
    if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_index) {
        SchemaHandler::<FileConfigRepository>::load_initial_schemas(&db_path, &ui);
        ClassificationHandler::<FileConfigRepository>::load_initial_classifications(&db_path, &ui);
    }
    
    // Setup all UI callbacks through presentation layer
    schema_handler.setup_callbacks(&ui);
    classification_handler.setup_callbacks(&ui);
    archive_handler.setup_callbacks(&ui);
    settings_handler.setup_callbacks(&ui);
    
    // Legacy callbacks (to be migrated)
    let ui_weak = ui.as_weak();
    ui.on_add_fonds(move || {
        if let Some(_ui) = ui_weak.upgrade() {
            println!("Add fonds clicked");
            // TODO: implement add fonds logic
        }
    });
    
    let ui_weak = ui.as_weak();
    ui.on_add_file(move || {
        if let Some(_ui) = ui_weak.upgrade() {
            println!("Add file clicked");
            // TODO: implement add file logic
        }
    });
    
    ui.run()?;
    
    Ok(())
}
