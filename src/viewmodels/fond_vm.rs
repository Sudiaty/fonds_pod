use crate::core::CrudViewModel;
use crate::core::CrudViewModelBase;
use crate::models::Fond;
use crate::persistence::FondsRepository;
use crate::AppWindow;
use crate::CrudListItem;
use slint::{ComponentHandle, Model};
use std::cell::RefCell;
use std::rc::Rc;

/// Fond（全宗）管理ViewModel
///
/// 此ViewModel通过复用CrudViewModelBase trait和宏，提供了极简的实现。
/// 只需实现 `create_default()` 方法来定义新项的默认值。
pub struct FondViewModel {
    inner: CrudViewModel<Fond, FondsRepository>,
    library_path: Option<String>,
}
impl FondViewModel {
    /// 创建新的FondViewModel实例
    pub fn new(repo: Rc<RefCell<FondsRepository>>, library_path: Option<String>) -> Self {
        let inner = CrudViewModel::new(repo);
        Self { inner, library_path }
    }

    /// 创建默认的Fond实例 - 由 `impl_crud_vm_base!` 宏使用
    fn create_default() -> Fond {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);

        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        Fond {
            id: 0,
            fond_no: format!("F{:03}", count),
            fond_classification_code: String::new(),
            name: "新全宗".to_string(),
            ..Default::default()
        }
    }

    /// 根据索引获取全宗项
    pub fn get_by_index(&self, index: usize) -> Option<CrudListItem> {
        self.inner.items.row_data(index)
    }

    /// 更新数据库连接并重新加载数据
    pub fn update_connection(&self, new_conn: Rc<RefCell<diesel::SqliteConnection>>) {
        self.inner.get_repo().borrow_mut().update_connection(new_conn);
        self.load();
    }

    /// 更新数据库连接和library路径
    pub fn update_connection_with_library(&mut self, new_conn: Rc<RefCell<diesel::SqliteConnection>>, library_path: Option<String>) {
        self.library_path = library_path;
        self.update_connection(new_conn);
    }

    /// 设置library路径
    pub fn set_library_path(&mut self, path: Option<String>) {
        self.library_path = path;
    }

    /// 为UI设置CRUD回调 - 标准实现在这里
    pub fn setup_callbacks(vm: Rc<RefCell<Self>>, ui_handle: &AppWindow) {
        use crate::core::CrudViewModelBase;

        // Add callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_fond_add(move || {
            log::info!("FondViewModel::setup_callbacks: add triggered");
            if let Some(ui) = ui_weak.upgrade() {
                vm_clone.borrow().add();
                let items = vm_clone.borrow().get_items();
                log::info!(
                    "FondViewModel::setup_callbacks: Setting {} items to UI",
                    items.row_count()
                );
                ui.set_fond_items(items);
            }
        });

        // Delete callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_fond_delete(move |idx| {
            log::info!(
                "FondViewModel::setup_callbacks: delete triggered for index {}",
                idx
            );
            if let Some(ui) = ui_weak.upgrade() {
                let _ = vm_clone.borrow().delete(idx);
                let items = vm_clone.borrow().get_items();
                log::info!(
                    "FondViewModel::setup_callbacks: Setting {} items to UI",
                    items.row_count()
                );
                ui.set_fond_items(items);
            }
        });
    }
}

// 实现自定义的CrudViewModelBase trait，覆盖宏生成的版本
impl CrudViewModelBase for FondViewModel {
    fn vm_name() -> &'static str {
        "FondViewModel"
    }

    fn load(&self) {
        log::info!("{}: Loading data", Self::vm_name());
        self.inner.load();
        log::info!(
            "{}: Loaded {} items",
            Self::vm_name(),
            self.inner.items.row_count()
        );
    }

    fn get_items(&self) -> slint::ModelRc<crate::CrudListItem> {
        self.inner.get_items()
    }

    fn add(&self) {
        log::info!("{}: Adding new item", Self::vm_name());
        let mut new_item = Self::create_default();
        self.inner.add(&mut new_item);
        
        // 创建全宗文件夹
        if let Some(library_path) = &self.library_path {
            let fond_dir = std::path::Path::new(library_path).join(&new_item.fond_no);
            if let Err(e) = std::fs::create_dir_all(&fond_dir) {
                log::error!("Failed to create fond directory {:?}: {}", fond_dir, e);
            } else {
                log::info!("Created fond directory: {:?}", fond_dir);
            }
        }
        
        log::info!(
            "{}: Added item, total count: {}",
            Self::vm_name(),
            self.inner.items.row_count()
        );
    }

    fn delete(&self, index: i32) -> Result<(), String> {
        log::info!("{}: Deleting item at index {}", Self::vm_name(), index);
        if index >= 0 {
            match self.inner.delete(index as usize) {
                Ok(_) => {
                    log::info!(
                        "{}: Deleted item, remaining count: {}",
                        Self::vm_name(),
                        self.inner.items.row_count()
                    );
                    Ok(())
                }
                Err(e) => {
                    log::error!("{}: Failed to delete item: {}", Self::vm_name(), e);
                    Err(e)
                }
            }
        } else {
            Err("无效索引".to_string())
        }
    }

    fn activate(&self, id: i32) {
        log::info!("{}: Activating item with id {}", Self::vm_name(), id);
        self.inner.activate(id);
        log::info!(
            "{}: Activated item, current count: {}",
            Self::vm_name(),
            self.inner.items.row_count()
        );
    }

    fn deactivate(&self, id: i32) {
        log::info!("{}: Deactivating item with id {}", Self::vm_name(), id);
        self.inner.deactivate(id);
        log::info!(
            "{}: Deactivated item, current count: {}",
            Self::vm_name(),
            self.inner.items.row_count()
        );
    }
}
