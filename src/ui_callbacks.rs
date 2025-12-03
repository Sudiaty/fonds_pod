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

    // Handle check update
    let ui_weak_for_update = ui.as_weak();
    ui.on_check_update(move || {
        if let Some(ui) = ui_weak_for_update.upgrade() {
            let current_version = ui.get_app_version().to_string();
            
            // Fetch latest version from GitHub API
            let client = reqwest::blocking::Client::new();
            let latest_version = match client
                .get("https://api.github.com/repos/Sudiaty/fonds_pod/releases/latest")
                .header("User-Agent", "fonds_pod")
                .send()
            {
                Ok(response) => {
                    if let Ok(json) = response.json::<serde_json::Value>() {
                        json["tag_name"]
                            .as_str()
                            .map(|s| s.trim_start_matches('v').to_string())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            };
            
            let Some(latest_version) = latest_version else {
                ui.invoke_show_toast("Failed to check for updates".into());
                return;
            };
            
            if current_version.trim_start_matches('v') == latest_version {
                ui.invoke_show_toast("Already up to date".into());
            } else {
                // Download update
                let url = format!("https://github.com/Sudiaty/fonds_pod/releases/download/v{}/fonds-pod-{}-windows-x86_64.exe", latest_version, latest_version);
                if let Ok(exe_path) = std::env::current_exe() {
                    let temp_path = exe_path.with_file_name("fonds_pod_update.exe");
                    let batch_path = exe_path.with_file_name("update.bat");
                    match reqwest::blocking::get(&url) {
                        Ok(response) => {
                            if let Ok(bytes) = response.bytes() {
                                if std::fs::write(&temp_path, &bytes).is_ok() {
                                    // Create batch script to replace exe after program exits
                                    let batch_content = format!(
                                        "@echo off\r\n\
                                        timeout /t 2 /nobreak >nul\r\n\
                                        copy /Y \"{}\" \"{}\"\r\n\
                                        del \"{}\"\r\n\
                                        start \"\" \"{}\"\r\n\
                                        del \"%~f0\"\r\n",
                                        temp_path.to_string_lossy(),
                                        exe_path.to_string_lossy(),
                                        temp_path.to_string_lossy(),
                                        exe_path.to_string_lossy()
                                    );
                                    if std::fs::write(&batch_path, &batch_content).is_ok() {
                                        ui.invoke_show_toast("Update downloaded. Restarting...".into());
                                        // Run batch script and exit
                                        let _ = std::process::Command::new("cmd")
                                            .args(&["/C", "start", "", &batch_path.to_string_lossy()])
                                            .spawn();
                                        std::process::exit(0);
                                    } else {
                                        ui.invoke_show_toast("Failed to create update script".into());
                                    }
                                } else {
                                    ui.invoke_show_toast("Failed to save update file".into());
                                }
                            } else {
                                ui.invoke_show_toast("Failed to download update".into());
                            }
                        }
                        Err(_) => {
                            ui.invoke_show_toast("Failed to download update".into());
                        }
                    }
                } else {
                    ui.invoke_show_toast("Cannot determine executable path".into());
                }
            }
        }
    });
}
