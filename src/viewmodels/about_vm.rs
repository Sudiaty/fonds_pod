/// About View Model - MVVM architecture
/// Manages the state and business logic for the about page
use crate::AppWindow;
use std::cell::RefCell;
use std::rc::Rc;
use serde_json;
use slint::ComponentHandle;

/// About ViewModel - handles state and business logic for about page
pub struct AboutViewModel {
    pub app_version: String,
}

impl Default for AboutViewModel {
    fn default() -> Self {
        Self {
            app_version: String::new(),
        }
    }
}

impl AboutViewModel {
    pub fn new(app_version: &str) -> Self {
        Self {
            app_version: app_version.to_string(),
        }
    }

    /// Check for application updates
    pub fn check_update(&self, ui_handle: &AppWindow) {
        log::info!("Check update requested for version: {}", self.app_version);
        
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
            ui_handle.invoke_show_toast("Failed to check for updates".into());
            return;
        };
        
        if self.app_version.trim_start_matches('v') == latest_version {
            ui_handle.invoke_show_toast("Already up to date".into());
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
                                    ui_handle.invoke_show_toast("Update downloaded. Restarting...".into());
                                    // Run batch script and exit
                                    let _ = std::process::Command::new("cmd")
                                        .args(&["/C", "start", "", &batch_path.to_string_lossy()])
                                        .spawn();
                                    std::process::exit(0);
                                } else {
                                    ui_handle.invoke_show_toast("Failed to create update script".into());
                                }
                            } else {
                                ui_handle.invoke_show_toast("Failed to save update file".into());
                            }
                        } else {
                            ui_handle.invoke_show_toast("Failed to download update".into());
                        }
                    }
                    Err(_) => {
                        ui_handle.invoke_show_toast("Failed to download update".into());
                    }
                }
            } else {
                ui_handle.invoke_show_toast("Cannot determine executable path".into());
            }
        }
    }

    /// Setup all about page callbacks
    pub fn setup_callbacks(vm: Rc<RefCell<Self>>, ui_handle: &AppWindow) {
        // Set app version
        {
            let vm_ref = vm.borrow();
            ui_handle.set_app_version(vm_ref.app_version.clone().into());
        }

        // Check update callback
        ui_handle.on_check_update({
            let vm = Rc::clone(&vm);
            let ui_weak = ui_handle.as_weak();
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let vm_ref = vm.borrow();
                    vm_ref.check_update(&ui);
                }
            }
        });
    }
}
