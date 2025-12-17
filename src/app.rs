use std::cell::RefCell;
use std::rc::Rc;

use fonds_pod_lib::services::SettingsService;
use fonds_pod_lib::viewmodels::{
    AboutViewModel, FondClassificationViewModel, FondViewModel, HomeViewModel, SchemaViewModel, SchemaItemViewModel, SettingsViewModel,
};
use fonds_pod_lib::AppWindow;
use fonds_pod_lib::CrudViewModelBase;
use slint::{ComponentHandle, Model, Timer};

// 使用类型别名简化 Rc<RefCell<T>> 的使用
type SharedVm<T> = Rc<RefCell<T>>;

pub struct App {
    pub settings_vm: SharedVm<SettingsViewModel>,
    pub about_vm: SharedVm<AboutViewModel>,
    pub home_vm: SharedVm<HomeViewModel>,
    pub fond_vm: SharedVm<FondViewModel>,
    pub fond_classification_vm: SharedVm<FondClassificationViewModel>,
    pub schema_vm: SharedVm<SchemaViewModel>,
    pub schema_item_vm: SharedVm<SchemaItemViewModel>,
}

impl App {
    /// 获取数据库连接 - 从last_opened_library设置中获取SQLite数据库路径
    fn get_database_connection(settings_service: &SettingsService) -> Rc<RefCell<diesel::SqliteConnection>> {
        let db_path = if let Ok(Some(path)) = settings_service.get_last_opened_library() {
            let db = std::path::PathBuf::from(&path).join(".fondspod.db");
            log::info!("App: Using database at: {:?}", db);
            db
        } else {
            log::warn!("App: No last_opened_library found, using in-memory database");
            std::path::PathBuf::from(":memory:")
        };

        fonds_pod_lib::persistence::establish_connection(&db_path).unwrap_or_else(|_| {
            fonds_pod_lib::persistence::establish_connection(&std::path::PathBuf::from(
                ":memory:",
            ))
            .unwrap()
        })
    }

    pub fn initialize(ui_handle: &AppWindow) -> Self {
        let settings_service = Rc::new(SettingsService::new());

        // Initialize Fond ViewModel
        let fond_vm = Rc::new(RefCell::new(Self::initialize_fond_vm(&settings_service)));
        fond_vm.borrow().load();

        // Initialize Settings ViewModel
        let settings_vm = Rc::new(RefCell::new(SettingsViewModel::new(Rc::clone(
            &settings_service,
        ))));

        // Initialize About ViewModel
        let about_vm = Rc::new(RefCell::new(AboutViewModel::new(crate::APP_VERSION)));

        // Initialize Home ViewModel
        let home_vm = Rc::new(RefCell::new(HomeViewModel::new(Rc::clone(
            &settings_service,
        ))));

        // Load initial settings
        if let Ok(libraries) = settings_service.list_archive_libraries() {
            if let Ok(language) = settings_service.get_language() {
                let mut vm = settings_vm.borrow_mut();
                vm.load_from_service(language, libraries);
                vm.init_ui(ui_handle);
            }
        }

        // Initialize Fond Classification ViewModel
        let fond_classification_vm = Rc::new(RefCell::new(
            Self::initialize_fond_classification_vm(&settings_service),
        ));
        fond_classification_vm.borrow().load();

        // Initialize Schema ViewModel
        let schema_vm = Rc::new(RefCell::new(Self::initialize_schema_vm(&settings_service)));
        schema_vm.borrow_mut().load();

        // Initialize Schema Item ViewModel
        let schema_item_vm = Rc::new(RefCell::new(Self::initialize_schema_item_vm(&settings_service)));
        schema_item_vm.borrow().load();

        App {
            settings_vm,
            about_vm,
            home_vm,
            fond_vm,
            fond_classification_vm,
            schema_vm,
            schema_item_vm,
        }
    }

    /// 初始化Fond ViewModel和数据库连接
    fn initialize_fond_vm(settings_service: &SettingsService) -> FondViewModel {
        let conn = Self::get_database_connection(settings_service);

        let repo = Rc::new(RefCell::new(
            fonds_pod_lib::FondsRepository::new(conn),
        ));
        FondViewModel::new(repo)
    }

    /// 初始化Fond Classification ViewModel和数据库连接
    fn initialize_fond_classification_vm(
        settings_service: &SettingsService,
    ) -> FondClassificationViewModel {
        let conn = Self::get_database_connection(settings_service);

        let repo = Rc::new(RefCell::new(
            fonds_pod_lib::FondClassificationsRepository::new(conn),
        ));
        FondClassificationViewModel::new(repo)
    }

    /// 初始化Schema ViewModel和数据库连接
    fn initialize_schema_vm(settings_service: &SettingsService) -> SchemaViewModel {
        let conn = Self::get_database_connection(settings_service);

        let schema_repo = Rc::new(RefCell::new(
            fonds_pod_lib::SchemaRepository::new(Rc::clone(&conn)),
        ));
        let schema_items_repo = Rc::new(RefCell::new(
            fonds_pod_lib::SchemaItemRepository::new(Rc::clone(&conn)),
        ));
        let mut vm = SchemaViewModel::new(schema_repo, schema_items_repo);
        vm.load();
        vm
    }

    /// 初始化Schema Item ViewModel和数据库连接
    fn initialize_schema_item_vm(settings_service: &SettingsService) -> SchemaItemViewModel {
        let conn = Self::get_database_connection(settings_service);

        let schema_item_repo = Rc::new(RefCell::new(
            fonds_pod_lib::SchemaItemRepository::new(conn),
        ));
        SchemaItemViewModel::new(schema_item_repo)
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
        FondClassificationViewModel::setup_callbacks(
            Rc::clone(&self.fond_classification_vm),
            ui_handle,
        );
        SchemaViewModel::setup_callbacks(
            Rc::clone(&self.schema_vm),
            Rc::clone(&self.schema_item_vm),
            ui_handle,
        );
        SchemaItemViewModel::setup_callbacks(Rc::clone(&self.schema_item_vm), ui_handle);

        // Initial load for Fond VM
        let items = self.fond_vm.borrow().get_items();
        log::info!(
            "App: Initial setup: Setting {} fond items to UI",
            items.row_count()
        );
        ui_handle.set_fond_items(items);

        // Initial load for Fond Classification VM
        let classification_items = self.fond_classification_vm.borrow().get_items();
        log::info!(
            "App: Initial setup: Setting {} classification items to UI",
            classification_items.row_count()
        );
        ui_handle.set_classification_crud_items(classification_items.clone());

        // Initialize child classifications for the first item if available
        self.fond_classification_vm.borrow_mut().initialize_child_classifications();
        let child_items = self.fond_classification_vm.borrow().get_child_items();
        ui_handle.set_child_crud_items(child_items);

        // Initial load for Schema VM
        let schema_items = self.schema_vm.borrow().get_items();
        log::info!(
            "App: Initial setup: Setting {} schema items to UI",
            schema_items.row_count()
        );
        ui_handle.set_schema_list_items(schema_items);

        let schema_item_items = self.schema_item_vm.borrow().get_items();
        ui_handle.set_detail_list_items(schema_item_items);

        // Setup common callbacks
        ui_handle.on_page_changed({
            let fond_vm = Rc::clone(&self.fond_vm);
            let fond_classification_vm = Rc::clone(&self.fond_classification_vm);
            let schema_vm = Rc::clone(&self.schema_vm);
            let schema_item_vm = Rc::clone(&self.schema_item_vm);
            let home_vm = Rc::clone(&self.home_vm);
            let ui_weak = ui_handle.as_weak();
            move |page_name| {
                log::info!("App: Navigated to page: {}", page_name);
                if let Some(ui) = ui_weak.upgrade() {
                    match page_name.as_str() {
                        "home" => {
                            if let Ok(mut vm) = home_vm.try_borrow_mut() {
                                if let Err(e) = vm.load_libraries() {
                                    log::error!("Failed to reload libraries on page change: {}", e);
                                } else {
                                    vm.init_ui(&ui);
                                }
                            }
                        }
                        _ => {
                            // Get current last_opened_library for database-dependent pages
                            let last_opened_library = home_vm.borrow().last_opened_library.clone();
                            if !last_opened_library.is_empty() {
                                let db_path = std::path::PathBuf::from(&last_opened_library).join(".fondspod.db");
                                let new_conn = fonds_pod_lib::persistence::establish_connection(&db_path).unwrap_or_else(|_| {
                                    fonds_pod_lib::persistence::establish_connection(&std::path::PathBuf::from(":memory:")).unwrap()
                                });

                                match page_name.as_str() {
                                    "fonds" => {
                                        fond_vm.borrow().update_connection(new_conn);
                                        let items = fond_vm.borrow().get_items();
                                        ui.set_fond_items(items);
                                    }
                                    "classification" => {
                                        let mut vm = fond_classification_vm.borrow_mut();
                                        vm.update_connection(new_conn);
                                        vm.initialize_child_classifications();
                                        let classification_items = vm.get_items();
                                        ui.set_classification_crud_items(classification_items.clone());
                                        let child_items = vm.get_child_items();
                                        ui.set_child_crud_items(child_items);
                                    }
                                    "schema" => {
                                        schema_vm.borrow_mut().update_connection(new_conn.clone());
                                        let schema_items = schema_vm.borrow().get_items();
                                        ui.set_schema_list_items(schema_items);
                                        schema_item_vm.borrow().update_connection(new_conn);
                                        let schema_item_items = schema_item_vm.borrow().get_items();
                                        ui.set_detail_list_items(schema_item_items);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        });

        ui_handle.on_show_toast({
            let ui_weak = ui_handle.as_weak();
            move |message| {
                if let Some(ui) = ui_weak.upgrade() {
                    log::info!("App: Toast message: {}", message);
                    ui.set_toast_message(message);
                    ui.set_toast_visible(true);
                    
                    // Schedule toast to disappear after 3 seconds using Slint Timer
                    let ui_weak_clone = ui_weak.clone();
                    Timer::single_shot(std::time::Duration::from_secs(3), move || {
                        if let Some(ui) = ui_weak_clone.upgrade() {
                            ui.set_toast_visible(false);
                        }
                    });
                }
            }
        });
    }
}
