use std::rc::Rc;
use std::cell::RefCell;

use slint::{ComponentHandle, Model};
use fonds_pod_lib::viewmodels::{SettingsViewModel, AboutViewModel, HomeViewModel, FondViewModel};
use fonds_pod_lib::services::SettingsService;
use fonds_pod_lib::AppWindow;

// 使用类型别名简化 Rc<RefCell<T>> 的使用
type SharedVm<T> = Rc<RefCell<T>>;

pub struct App {
    pub settings_vm: SharedVm<SettingsViewModel>,
    pub about_vm: SharedVm<AboutViewModel>,
    pub home_vm: SharedVm<HomeViewModel>,
    pub fond_vm: SharedVm<FondViewModel>,
}

impl App {
    pub fn initialize(ui_handle: &AppWindow) -> Self {
        let settings_service = Rc::new(SettingsService::new());
        
        // Initialize Fond ViewModel
        let fond_vm = Rc::new(RefCell::new(
            Self::initialize_fond_vm(&settings_service)
        ));
        fond_vm.borrow().load();

        // Initialize Settings ViewModel
        let settings_vm = Rc::new(RefCell::new(
            SettingsViewModel::new(Rc::clone(&settings_service))
        ));

        // Initialize About ViewModel
        let about_vm = Rc::new(RefCell::new(
            AboutViewModel::new(crate::APP_VERSION)
        ));

        // Initialize Home ViewModel
        let home_vm = Rc::new(RefCell::new(
            HomeViewModel::new(Rc::clone(&settings_service))
        ));

        // Load initial settings
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
            fond_vm,
        }
    }

    /// 初始化Fond ViewModel和数据库连接
    fn initialize_fond_vm(settings_service: &SettingsService) -> FondViewModel {
        // Initialize DB connection from last_opened_library in settings
        let db_path = if let Ok(Some(path)) = settings_service.get_last_opened_library() {
            let db = std::path::PathBuf::from(&path).join(".fondspod.db");
            log::info!("App: Using database at: {:?}", db);
            db
        } else {
            log::warn!("App: No last_opened_library found, using in-memory database");
            std::path::PathBuf::from(":memory:")
        };
        
        let conn = fonds_pod_lib::persistence::establish_connection(&db_path).unwrap_or_else(|_| {
            fonds_pod_lib::persistence::establish_connection(&std::path::PathBuf::from(":memory:")).unwrap()
        });

        let repo = Rc::new(RefCell::new(fonds_pod_lib::persistence::FondsRepository::new(conn)));
        FondViewModel::new(repo)
    }

    /// Setup all UI callbacks - 由各个 ViewModel 负责自己的回调
    pub fn setup_ui_callbacks(&self, ui_handle: &AppWindow) {
        // Initialize ViewModel UIs
        self.settings_vm.borrow().init_ui(ui_handle);
        
        // Setup ViewModel callbacks
        SettingsViewModel::setup_callbacks(Rc::clone(&self.settings_vm), ui_handle);
        AboutViewModel::setup_callbacks(Rc::clone(&self.about_vm), ui_handle);
        HomeViewModel::setup_callbacks(Rc::clone(&self.home_vm), ui_handle);
        FondViewModel::setup_callbacks(Rc::clone(&self.fond_vm), ui_handle);

        // Initial load for Fond VM
        let items = self.fond_vm.borrow().get_items();
        log::info!("App: Initial setup: Setting {} fond items to UI", items.row_count());
        ui_handle.set_fond_items(items);

        // Setup common callbacks
        ui_handle.on_page_changed({
            move |page_name| {
                log::info!("App: Navigated to page: {}", page_name);
            }
        });
        
        ui_handle.on_show_toast({
            let ui_weak = ui_handle.as_weak();
            move |message| {
                if let Some(ui) = ui_weak.upgrade() {
                    log::info!("App: Toast message: {}", message);
                    ui.set_toast_message(message);
                    ui.set_toast_visible(true);
                }
            }
        });
    }
}
