/// Application initialization module
/// Handles UI creation, service initialization, and handler setup

use std::rc::Rc;
use std::error::Error;
use crate::infrastructure::FileConfigRepository;
use crate::services::{ArchiveService, FileService};
use crate::presentation::{SchemaHandler, ArchiveHandler, SettingsHandler, ClassificationHandler, FondsHandler};
use crate::AppWindow;

/// Container to hold all handlers and keep them alive
pub struct AppHandlers {
    _archive_handler: ArchiveHandler<FileConfigRepository>,
    _schema_handler: SchemaHandler<FileConfigRepository>,
    _classification_handler: ClassificationHandler<FileConfigRepository>,
    _settings_handler: SettingsHandler<FileConfigRepository>,
    _fonds_handler: FondsHandler<FileConfigRepository>,
}

/// Initialize the application UI and services
pub fn initialize_app(
) -> Result<(AppWindow, Rc<ArchiveService<FileConfigRepository>>, AppHandlers), Box<dyn Error>> {
    // Initialize configuration repository first to load language settings
    let config_repo = FileConfigRepository::new();
    let temp_archive_service = ArchiveService::new(config_repo);

    // Initialize application
    println!("FondsPod starting...");
    
    let ui = AppWindow::new().map_err(|e| { eprintln!("UI create failed: {}", e); e })?;

    // Set application version
    ui.set_app_version(crate::APP_VERSION.into());

    // Load language setting and set UI, then select bundled translation
    if let Ok(settings) = temp_archive_service.get_settings() {
        let language_index = if settings.language == "zh_CN" { 0 } else { 1 };
        ui.set_selected_language(language_index);
        
        // Select bundled translation based on settings
        if !settings.language.is_empty() {
            let _ = slint::select_bundled_translation(&settings.language);
        }
    }

    let archive_service = Rc::new(temp_archive_service);

    // Initialize services and handlers, then set up callbacks
    let handlers = setup_handlers(&ui, archive_service.clone())?;

    Ok((ui, archive_service, handlers))
}

/// Set up all handlers and their callbacks
fn setup_handlers(
    ui: &AppWindow,
    archive_service: Rc<ArchiveService<FileConfigRepository>>,
) -> Result<AppHandlers, Box<dyn Error>> {
    // Initialize presentation handlers
    let archive_handler = ArchiveHandler::new(archive_service.clone());
    let schema_handler = SchemaHandler::new(archive_service.clone());
    let classification_handler = ClassificationHandler::new(archive_service.clone());
    let settings_handler = SettingsHandler::new(archive_service.clone());
    let file_service = Rc::new(FileService);
    let fonds_handler = FondsHandler::new(archive_service.clone(), file_service);
    
    // Initialize UI state
    archive_handler.initialize(ui)?;
    
    // Load initial schemas if an archive is selected
    let selected_index = ui.get_selected_archive();
    if let Ok(Some(db_path)) = archive_service.get_database_path_by_index(selected_index) {
        SchemaHandler::<FileConfigRepository>::load_initial_schemas(&db_path, ui);
        ClassificationHandler::<FileConfigRepository>::load_initial_classifications(&db_path, ui);
        FondsHandler::<FileConfigRepository>::load_initial_fonds(&db_path, ui);
    }
    
    // Setup all UI callbacks through presentation layer
    schema_handler.setup_callbacks(ui);
    classification_handler.setup_callbacks(ui);
    archive_handler.setup_callbacks(ui);
    settings_handler.setup_callbacks(ui);
    fonds_handler.setup_callbacks(ui);

    Ok(AppHandlers {
        _archive_handler: archive_handler,
        _schema_handler: schema_handler,
        _classification_handler: classification_handler,
        _settings_handler: settings_handler,
        _fonds_handler: fonds_handler,
    })
}
