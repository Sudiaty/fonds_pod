use crate::core::CrudViewModel;
use crate::models::schema::Schema;
use crate::persistence::schema_repository::SchemaRepository;
use std::rc::Rc;
use std::cell::RefCell;
use slint::{Model, ComponentHandle};
use crate::{CrudListItem, AppWindow};

/// Schema管理ViewModel
///
/// 此ViewModel通过复用CrudViewModelBase trait和宏，提供了极简的实现。
/// 只需实现 `create_default()` 方法来定义新项的默认值。
pub struct SchemaViewModel {
    inner: CrudViewModel<Schema, SchemaRepository>,
    selected_index: Option<usize>,
}

impl SchemaViewModel {
    pub fn new(repo: Rc<RefCell<SchemaRepository>>) -> Self {
        let inner = CrudViewModel::new(repo);
        Self { 
            inner,
            selected_index: None,
        }
    }

    /// 创建默认的Schema实例 - 由 `impl_crud_vm_base!` 宏使用
    fn create_default() -> Schema {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);

        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        Schema {
            id: 0,
            schema_no: format!("S{:03}", count),
            name: "新案卷目录".to_string(),
            sort_order: 0,
            ..Default::default()
        }
    }

    /// 根据索引获取Schema项
    pub fn get_by_index(&self, index: usize) -> Option<CrudListItem> {
        self.inner.items.row_data(index)
    }

    /// 设置选中的索引
    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.selected_index = index;
    }

    /// 获取选中的索引
    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// 为UI设置CRUD回调 - 标准实现在这里
    pub fn setup_callbacks(
        vm: Rc<RefCell<Self>>,
        ui_handle: &AppWindow,
    ) {
        use crate::core::CrudViewModelBase;

        // Add callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_schema_add(move || {
            log::info!("SchemaViewModel::setup_callbacks: add triggered");
            vm_clone.borrow().add();
            if let Some(ui) = ui_weak.upgrade() {
                let items = vm_clone.borrow().get_items();
                ui.set_schema_list_items(items);
            }
        });

        // Schema activated callback - 设置选中索引
        let vm_clone = vm.clone();
        ui_handle.on_schema_activated(move |index| {
            log::info!("SchemaViewModel::setup_callbacks: schema activated for index {}", index);
            vm_clone.borrow_mut().set_selected_index(Some(index as usize));
        });

        // Delete callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_schema_delete(move || {
            log::info!("SchemaViewModel::setup_callbacks: delete triggered");
            
            // 先获取选中的索引，避免RefCell双重借用
            let selected_index = vm_clone.borrow().get_selected_index();
            
            if let Some(index) = selected_index {
                // 执行删除操作
                let delete_result = vm_clone.borrow().delete(index as i32);
                
                match delete_result {
                    Ok(_) => {
                        log::info!("SchemaViewModel::setup_callbacks: Successfully deleted item at index {}", index);
                        // 清除选中状态
                        vm_clone.borrow_mut().set_selected_index(None);
                        if let Some(ui) = ui_weak.upgrade() {
                            let items = vm_clone.borrow().get_items();
                            ui.set_schema_list_items(items);
                        }
                    }
                    Err(e) => {
                        log::error!("SchemaViewModel::setup_callbacks: Failed to delete item at index {}: {}", index, e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_toast_message(e.into());
                            ui.set_toast_visible(true);
                            
                            let ui_weak_clone = ui_weak.clone();
                            slint::Timer::single_shot(std::time::Duration::from_secs(3), move || {
                                if let Some(ui) = ui_weak_clone.upgrade() {
                                    ui.set_toast_visible(false);
                                }
                            });
                        }
                    }
                }
            } else {
                log::warn!("SchemaViewModel::setup_callbacks: No item selected for deletion");
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_toast_message("请先选择要删除的案卷目录".into());
                    ui.set_toast_visible(true);
                    
                    let ui_weak_clone = ui_weak.clone();
                    slint::Timer::single_shot(std::time::Duration::from_secs(3), move || {
                        if let Some(ui) = ui_weak_clone.upgrade() {
                            ui.set_toast_visible(false);
                        }
                    });
                }
            }
        });
    }
}

// 使用宏自动生成 CrudViewModelBase trait 实现
crate::impl_crud_vm_base!(SchemaViewModel, "SchemaViewModel", Schema);

use crate::models::schema_item::SchemaItem;
use crate::persistence::schema_item_repository::SchemaItemRepository;

/// SchemaItem管理ViewModel
pub struct SchemaItemViewModel {
    inner: CrudViewModel<SchemaItem, SchemaItemRepository>,
    selected_schema_id: Option<i32>,
    selected_index: Option<usize>,
}

impl SchemaItemViewModel {
    /// 创建新的 SchemaItemViewModel 实例
    pub fn new(repo: Rc<RefCell<SchemaItemRepository>>) -> Self {
        let inner = CrudViewModel::new(repo);
        Self {
            inner,
            selected_schema_id: None,
            selected_index: None,
        }
    }

    /// 设置选中的schema id
    pub fn set_selected_schema_id(&mut self, schema_id: Option<i32>) {
        self.selected_schema_id = schema_id;
        // TODO: filter items
    }

    /// 设置选中的索引
    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.selected_index = index;
    }

    /// 获取选中的索引
    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// 为UI设置CRUD回调
    pub fn setup_callbacks(
        vm: Rc<RefCell<Self>>,
        ui_handle: &AppWindow,
    ) {
        // Add callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_schema_item_add(move || {
            log::info!("SchemaItemViewModel::setup_callbacks: add triggered");
            vm_clone.borrow().add();
            if let Some(ui) = ui_weak.upgrade() {
                let items = vm_clone.borrow().get_items();
                ui.set_detail_list_items(items);
            }
        });

        // Schema item activated callback - 设置选中索引
        let vm_clone = vm.clone();
        ui_handle.on_schema_item_activated(move |index| {
            log::info!("SchemaItemViewModel::setup_callbacks: schema item activated for index {}", index);
            vm_clone.borrow_mut().set_selected_index(Some(index as usize));
        });

        // Delete callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_schema_item_delete(move || {
            log::info!("SchemaItemViewModel::setup_callbacks: delete triggered");
            
            // 先获取选中的索引，避免RefCell双重借用
            let selected_index = vm_clone.borrow().get_selected_index();
            
            if let Some(index) = selected_index {
                // 执行删除操作
                let delete_result = vm_clone.borrow().delete(index as i32);
                
                match delete_result {
                    Ok(_) => {
                        log::info!("SchemaItemViewModel::setup_callbacks: Successfully deleted item at index {}", index);
                        // 清除选中状态
                        vm_clone.borrow_mut().set_selected_index(None);
                        if let Some(ui) = ui_weak.upgrade() {
                            let items = vm_clone.borrow().get_items();
                            ui.set_detail_list_items(items);
                        }
                    }
                    Err(e) => {
                        log::error!("SchemaItemViewModel::setup_callbacks: Failed to delete item at index {}: {}", index, e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_toast_message(e.into());
                            ui.set_toast_visible(true);
                            
                            let ui_weak_clone = ui_weak.clone();
                            slint::Timer::single_shot(std::time::Duration::from_secs(3), move || {
                                if let Some(ui) = ui_weak_clone.upgrade() {
                                    ui.set_toast_visible(false);
                                }
                            });
                        }
                    }
                }
            } else {
                log::warn!("SchemaItemViewModel::setup_callbacks: No item selected for deletion");
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_toast_message("请先选择要删除的条目".into());
                    ui.set_toast_visible(true);
                    
                    let ui_weak_clone = ui_weak.clone();
                    slint::Timer::single_shot(std::time::Duration::from_secs(3), move || {
                        if let Some(ui) = ui_weak_clone.upgrade() {
                            ui.set_toast_visible(false);
                        }
                    });
                }
            }
        });
    }

    /// 自定义add方法，设置schema_id
    pub fn add(&self) {
        if let Some(schema_id) = self.selected_schema_id {
            use std::sync::atomic::{AtomicU32, Ordering};
            static COUNTER: AtomicU32 = AtomicU32::new(1);

            let count = COUNTER.fetch_add(1, Ordering::SeqCst);
            let item = SchemaItem {
                id: 0,
                schema_id,
                item_no: format!("I{:03}", count),
                item_name: "新条目".to_string(),
                created_by: String::new(),
                created_machine: String::new(),
                created_at: chrono::Utc::now().naive_utc(),
            };
            self.inner.add(item);
        }
    }

    /// 自定义delete方法
    pub fn delete(&self, index: i32) -> Result<(), String> {
        self.inner.delete(index as usize)
    }

    /// 自定义get_items方法
    pub fn get_items(&self) -> slint::ModelRc<CrudListItem> {
        self.inner.get_items()
    }

    /// 自定义load方法
    pub fn load(&self) {
        self.inner.load();
    }
}

impl crate::CrudViewModelBase for SchemaItemViewModel {
    fn vm_name() -> &'static str {
        "SchemaItemViewModel"
    }

    fn load(&self) {
        self.load();
    }

    fn get_items(&self) -> slint::ModelRc<crate::CrudListItem> {
        self.get_items()
    }

    fn add(&self) {
        self.add();
    }

    fn delete(&self, index: i32) -> Result<(), String> {
        self.delete(index)
    }
}