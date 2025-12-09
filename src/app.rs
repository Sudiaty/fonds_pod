use std::rc::Rc;
use std::cell::RefCell;

use slint::ComponentHandle;
use fonds_pod_lib::viewmodels::{SettingsViewModel, AboutViewModel, HomeViewModel};
use fonds_pod_lib::services::SettingsService;
use fonds_pod_lib::AppWindow;

// 使用类型别名简化 Rc<RefCell<T>> 的使用
type SharedVm<T> = Rc<RefCell<T>>;

pub struct App {
    pub settings_vm: SharedVm<SettingsViewModel>,
    pub about_vm: SharedVm<AboutViewModel>,
    pub home_vm: SharedVm<HomeViewModel>,
}

impl App {
    pub fn initialize(ui_handle: &AppWindow) -> Self {
        let settings_service = Rc::new(SettingsService::new());
        
        // 初始化所有 ViewModels
        let settings_vm = Rc::new(RefCell::new(
            SettingsViewModel::new(Rc::clone(&settings_service))
        ));
        let about_vm = Rc::new(RefCell::new(
            AboutViewModel::new(crate::APP_VERSION)
        ));
        let home_vm = Rc::new(RefCell::new(
            HomeViewModel::new(Rc::clone(&settings_service))
        ));

        // 加载初始设置
        if let Ok(libraries) = settings_service.list_archive_libraries() {
            if let Ok(language) = settings_service.get_language() {
                let mut vm = settings_vm.borrow_mut();
                vm.load_from_service(language, libraries);
                vm.init_ui(ui_handle);
            }
        }

        App {
            settings_vm,
            about_vm,
            home_vm,
        }
    }

    /// Setup all UI callbacks - 将业务逻辑委托给各个 ViewModel
    pub fn setup_ui_callbacks(&self, ui_handle: &AppWindow) {
        // ========== 初始化各 ViewModel 的 UI 和回调 ==========
        self.settings_vm.borrow().init_ui(ui_handle);
        SettingsViewModel::setup_callbacks(Rc::clone(&self.settings_vm), ui_handle);
        
        AboutViewModel::setup_callbacks(Rc::clone(&self.about_vm), ui_handle);
        
        HomeViewModel::setup_callbacks(Rc::clone(&self.home_vm), ui_handle);
        
        // ========== 设置通用回调 ==========
        self.setup_common_callbacks(ui_handle);
    }
    
    fn setup_common_callbacks(&self, ui_handle: &AppWindow) {
        // 页面切换回调
        ui_handle.on_page_changed({
            let ui_weak = ui_handle.as_weak();
            move |page_name| {
                log::info!("Navigated to page: {}", page_name);
                // Removed fond_classification_vm logic
            }
        });
        
        // Toast 消息回调
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
    }
}
