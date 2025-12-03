/// UI callbacks setup module
/// Handles all UI event callbacks including page changes, archive selection, and window management

use std::rc::Rc;
use crate::infrastructure::FileConfigRepository;
use crate::services::ArchiveService;
use crate::presentation::{SchemaHandler, ClassificationHandler, FondsHandler};
use crate::AppWindow;
use slint::ComponentHandle;

/// Set up all UI event callbacks
pub fn setup_ui_callbacks(
    ui: &AppWindow,
    archive_service: Rc<ArchiveService<FileConfigRepository>>,
    fonds_handler: Rc<FondsHandler<FileConfigRepository>>,
) {
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
}
