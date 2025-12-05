use std::rc::Rc;
use std::cell::RefCell;

use slint::ComponentHandle;
use fonds_pod_lib::viewmodels::{SchemaViewModel, SettingsViewModel, AboutViewModel};
use fonds_pod_lib::services::SettingsService;
use fonds_pod_lib::AppWindow;

// 使用类型别名简化 Rc<RefCell<T>> 的使用
type SharedVm<T> = Rc<RefCell<T>>;

pub struct App {
    // 持有所有 ViewModels 的共享指针
    pub schema_vm: SharedVm<SchemaViewModel>,
    pub settings_vm: SharedVm<SettingsViewModel>,
    pub about_vm: SharedVm<AboutViewModel>,
}

impl App {
    pub fn initialize(ui_handle: &AppWindow) -> Self {
        let schema_vm = Rc::new(RefCell::new(
            SchemaViewModel::new(ui_handle.as_weak())
        ));

        let settings_service = Rc::new(SettingsService::new());
        let settings_vm = Rc::new(RefCell::new(
            SettingsViewModel::new(Rc::clone(&settings_service))
        ));
        
        // Load initial settings
        if let Ok(libraries) = settings_service.list_archive_libraries() {
            if let Ok(language) = settings_service.get_language() {
                let mut vm = settings_vm.borrow_mut();
                vm.load_from_service(language, libraries);
                // Initialize UI with loaded settings
                vm.init_ui(ui_handle);
            }
        }

        let about_vm = Rc::new(RefCell::new(
            AboutViewModel::new(crate::APP_VERSION)
        ));

        App {
            schema_vm,
            settings_vm,
            about_vm,
        }
    }

    /// Setup all UI callbacks
    pub fn setup_ui_callbacks(&self, ui_handle: &AppWindow) {
        // ========== 初始化 UI 数据 ==========
        {
            let vm = self.settings_vm.borrow();
            vm.init_ui(ui_handle);
        }
        
        // ========== 设置通用回调 ==========
        ui_handle.on_page_changed({
            move |page_name| {
                log::info!("Navigated to page: {}", page_name);
            }
        });
        
        ui_handle.on_show_toast({
            let ui_weak = ui_handle.as_weak();
            move |message| {
                if let Some(ui) = ui_weak.upgrade() {
                    log::info!("Toast message: {}", message);
                    ui.set_toast_message(message);
                    ui.set_toast_visible(true);
                }
            }
        });
        
        // ========== About Page Callbacks ==========
        AboutViewModel::setup_callbacks(Rc::clone(&self.about_vm), ui_handle);
        
        // ========== Settings Page Callbacks ==========
        SettingsViewModel::setup_callbacks(Rc::clone(&self.settings_vm), ui_handle);
    }
}
