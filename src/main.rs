// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 从环境变量读取版本号（由GitHub Actions注入）
pub const APP_VERSION: &str = env!("APP_VERSION");

slint::include_modules!();

// New layered architecture
mod domain;
mod infrastructure;
mod services;
mod presentation;
mod app_init;
mod ui_callbacks;
mod tray_init;

use std::error::Error;
use std::rc::Rc;
use app_init::initialize_app;
use ui_callbacks::setup_ui_callbacks;
use tray_init::initialize_tray;
use presentation::FondsHandler;
use services::FileService;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize application (UI + services + handlers)
    let (ui, archive_service, _handlers) = initialize_app()?;
    
    // Create fonds handler for callbacks (stored in _handlers, kept alive)
    let fonds_handler = Rc::new(
        FondsHandler::new(archive_service.clone(), Rc::new(FileService))
    );
    
    // Initialize system tray
    initialize_tray(&ui);
    
    // Setup all UI event callbacks
    setup_ui_callbacks(&ui, archive_service, fonds_handler);
    
    // Run the UI event loop
    ui.run()?;
    
    Ok(())
}
