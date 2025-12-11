/// About View Model - MVVM architecture
/// Manages the state and business logic for the about page
use crate::AppWindow;
use std::cell::RefCell;
use std::rc::Rc;

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
    pub fn check_update(&self) {
        log::info!("Check update requested for version: {}", self.app_version);
        // TODO: Implement actual update check logic
        // This could involve:
        // 1. Making HTTP request to update server
        // 2. Comparing versions
        // 3. Showing update dialog if new version available
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
            move || {
                let vm_ref = vm.borrow();
                vm_ref.check_update();
            }
        });
    }
}
