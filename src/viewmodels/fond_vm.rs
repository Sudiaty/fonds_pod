use crate::core::CrudViewModel;
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
}
impl FondViewModel {
    /// 创建新的FondViewModel实例
    pub fn new(repo: Rc<RefCell<FondsRepository>>) -> Self {
        let inner = CrudViewModel::new(repo);
        Self { inner }
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

// 使用宏自动生成 CrudViewModelBase trait 实现
crate::impl_crud_vm_base!(FondViewModel, "FondViewModel", Fond);
