/// System tray initialization module
/// Handles tray icon creation and tray menu setup

use tray_item::{TrayItem, IconSource};
use crate::AppWindow;
use slint::ComponentHandle;

/// Initialize system tray icon with menu items
pub fn initialize_tray(ui: &AppWindow) {
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
}
