use crate::core::CrudViewModel;
use crate::models::fond_classification::FondClassification;
use crate::persistence::FondClassificationsRepository;
use crate::AppWindow;
use crate::CrudListItem;
use crate::CrudViewModelBase;
use slint::{ComponentHandle, Model};
use std::cell::RefCell;
use std::rc::Rc;

/// Fond Classification管理ViewModel
///
/// 此ViewModel通过复用CrudViewModelBase trait和宏，提供了极简的实现。
/// 只需实现 `create_default()` 方法来定义新项的默认值。
pub struct FondClassificationViewModel {
    inner: CrudViewModel<FondClassification, FondClassificationsRepository>,
}

impl FondClassificationViewModel {
    /// 创建新的 FondClassificationViewModel 实例
    pub fn new(repo: Rc<RefCell<FondClassificationsRepository>>) -> Self {
        let inner = CrudViewModel::new(repo);
        Self { inner }
    }

    /// 创建默认的FondClassification实例 - 由 `impl_crud_vm_base!` 宏使用
    fn create_default() -> FondClassification {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);

        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        FondClassification {
            id: 0,
            code: format!("C{:03}", count),
            name: "新分类".to_string(),
            parent_id: None,
            active: true,
            sort_order: 0,
            ..Default::default()
        }
    }

    /// 根据索引获取分类项
    pub fn get_by_index(&self, index: usize) -> Option<CrudListItem> {
        self.inner.items.row_data(index)
    }

    /// 为UI设置CRUD回调 - 标准实现在这里
    pub fn setup_callbacks(vm: Rc<RefCell<Self>>, ui_handle: &AppWindow) {
        use crate::core::CrudViewModelBase;

        // Add callback - 显示添加对话框
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_add_classification(move || {
            log::info!("FondClassificationViewModel::setup_callbacks: add triggered - showing dialog");
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_add_top_classification_dialog(true);
            }
        });

        // Confirm add top classification callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_confirm_add_top_classification(move |fields| {
            log::info!("FondClassificationViewModel::setup_callbacks: confirm add top classification triggered");
            if let Some(ui) = ui_weak.upgrade() {
                // 解析字段
                let mut code = String::new();
                let mut name = String::new();
                for field in fields.iter() {
                    if field.label == "label_code" {
                        code = field.value.to_string();
                    } else if field.label == "label_name" {
                        name = field.value.to_string();
                    }
                }

                if !code.is_empty() && !name.is_empty() {
                    // 创建新的分类
                    let new_classification = FondClassification {
                        id: 0,
                        code,
                        name,
                        parent_id: None,
                        active: true,
                        sort_order: 0,
                        ..Default::default()
                    };

                    // 添加到数据库
                    vm_clone.borrow().inner.add(new_classification);
                    log::info!("Successfully added new top classification");
                    // 重新加载数据以确保排序正确
                    vm_clone.borrow().load();
                    let items = vm_clone.borrow().get_items();
                    ui.set_classification_crud_items(items);
                    ui.set_show_add_top_classification_dialog(false);
                } else {
                    log::warn!("Code or name is empty, not adding classification");
                }
            }
        });

        // Cancel add top classification callback
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_cancel_add_top_classification(move || {
            log::info!("FondClassificationViewModel::setup_callbacks: cancel add top classification triggered");
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_add_top_classification_dialog(false);
            }
        });

        // Delete callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_delete_fond_classification(move |idx| {
            log::info!(
                "FondClassificationViewModel::setup_callbacks: delete triggered for index {}",
                idx
            );
            if let Some(ui) = ui_weak.upgrade() {
                vm_clone.borrow().delete(idx);
                let items = vm_clone.borrow().get_items();
                log::info!(
                    "FondClassificationViewModel::setup_callbacks: Setting {} items to UI",
                    items.row_count()
                );
                ui.set_classification_crud_items(items);
            }
        });

        // Activate callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_activate_top_classification(move |id| {
            log::info!(
                "FondClassificationViewModel::setup_callbacks: activate triggered for id {}",
                id
            );
            if let Some(ui) = ui_weak.upgrade() {
                vm_clone.borrow().activate(id);
                let items = vm_clone.borrow().get_items();
                log::info!(
                    "FondClassificationViewModel::setup_callbacks: Setting {} items to UI",
                    items.row_count()
                );
                ui.set_classification_crud_items(items);
            }
        });

        // Deactivate callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_deactivate_top_classification(move |id| {
            log::info!(
                "FondClassificationViewModel::setup_callbacks: deactivate triggered for id {}",
                id
            );
            if let Some(ui) = ui_weak.upgrade() {
                vm_clone.borrow().deactivate(id);
                let items = vm_clone.borrow().get_items();
                log::info!(
                    "FondClassificationViewModel::setup_callbacks: Setting {} items to UI",
                    items.row_count()
                );
                ui.set_classification_crud_items(items);
            }
        });
    }
}

// 使用宏自动生成 ActiveableCrudViewModel trait 实现
crate::impl_activeable_crud_vm_base!(
    FondClassificationViewModel,
    "FondClassificationViewModel",
    FondClassification
);
