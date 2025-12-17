use crate::core::CrudViewModel;
use crate::core::CrudViewModelBase;
use crate::core::GenericRepository;
use crate::models::schema::Schema;
use crate::persistence::schema_repository::SchemaRepository;
use std::rc::Rc;
use std::cell::RefCell;
use slint::{Model, ComponentHandle};
use crate::{CrudListItem, AppWindow};
use chrono;

/// Schema管理ViewModel
///
/// 此ViewModel通过复用CrudViewModelBase trait和宏，提供了极简的实现。
/// 只需实现 `create_default()` 方法来定义新项的默认值。
pub struct SchemaViewModel {
    inner: Rc<RefCell<CrudViewModel<Schema, SchemaRepository>>>,
    schema_items_repo: Rc<RefCell<SchemaItemRepository>>,
    schemas: Vec<Schema>,
    schema_items: Vec<SchemaItem>,
    selected_index: Option<usize>,
}

impl SchemaViewModel {
    pub fn new(repo: Rc<RefCell<SchemaRepository>>, schema_items_repo: Rc<RefCell<SchemaItemRepository>>) -> Self {
        let inner = Rc::new(RefCell::new(CrudViewModel::new(repo)));
        Self { 
            inner,
            schema_items_repo,
            schemas: Vec::new(),
            schema_items: Vec::new(),
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

    /// 获取UI列表项
    pub fn get_items(&self) -> slint::ModelRc<crate::CrudListItem> {
        self.inner.borrow().get_items()
    }

    /// 更新数据库连接并重新加载数据
    pub fn update_connection(&mut self, new_conn: Rc<RefCell<diesel::SqliteConnection>>) {
        self.inner.borrow().get_repo().borrow_mut().update_connection(new_conn);
        self.load();
    }

    /// 加载数据
    pub fn load(&mut self) {
        // Load schemas
        self.inner.borrow_mut().load();
        self.schemas = self.inner.borrow().get_repo().borrow_mut().find_all().unwrap_or_default();

        // Load schema items
        let items_result = self.schema_items_repo.borrow_mut().find_all();
        if let Ok(items) = items_result {
            self.schema_items = items;
            log::info!("SchemaViewModel: Loaded {} schema items", self.schema_items.len());
        } else {
            log::error!("SchemaViewModel: Failed to load schema items");
        }
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
        schema_item_vm: Rc<RefCell<SchemaItemViewModel>>,
        ui_handle: &AppWindow,
    ) {
        use crate::core::CrudViewModelBase;

        // Add callback - 显示对话框
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_schema_add(move || {
            log::info!("SchemaViewModel::setup_callbacks: add triggered - showing dialog");
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_add_schema_dialog(true);
            }
        });

        // Confirm add schema callback
        let _vm_clone = vm.clone();
        let schema_item_vm_clone = Rc::clone(&schema_item_vm);
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_confirm_add_schema(move |fields| {
            log::info!("SchemaViewModel::setup_callbacks: confirm add schema triggered");
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
                    // 创建新的Schema
                    let mut new_schema = Schema {
                        id: 0,
                        schema_no: code,
                        name,
                        sort_order: 0,
                        ..Default::default()
                    };

                    // 添加到数据库
                    _vm_clone.borrow_mut().inner.borrow_mut().add(&mut new_schema);
                    log::info!("Successfully added new schema with id {}", new_schema.id);
                    // 重新加载数据
                    _vm_clone.borrow_mut().load();
                    let items = _vm_clone.borrow().get_items();
                    ui.set_schema_list_items(items);

                    // 设置选中新添加的schema
                    let items = _vm_clone.borrow().get_items();
                    let new_index = items.row_count() - 1;
                    _vm_clone.borrow_mut().set_selected_index(Some(new_index));

                    // 设置schema_item的selected_schema_id
                    schema_item_vm_clone.borrow_mut().set_selected_schema_id(Some(new_schema.id));
                    // 加载schema items
                    schema_item_vm_clone.borrow().load();
                    let detail_items = schema_item_vm_clone.borrow().get_items();
                    ui.set_detail_list_items(detail_items);

                    ui.set_show_add_schema_dialog(false);
                } else {
                    log::warn!("Code or name is empty, not adding schema");
                }
            }
        });

        // Cancel add schema callback
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_cancel_add_schema(move || {
            log::info!("SchemaViewModel::setup_callbacks: cancel add schema triggered");
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_add_schema_dialog(false);
            }
        });

        // Schema activated callback - 设置选中索引和加载schema items
        let vm_clone = vm.clone();
        let schema_item_vm_clone = Rc::clone(&schema_item_vm);
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_schema_activated(move |index| {
            log::info!("SchemaViewModel::setup_callbacks: schema activated for index {}", index);
            vm_clone.borrow_mut().set_selected_index(Some(index as usize));
            
            // 获取选中schema的id
            if let Some(item) = vm_clone.borrow().get_items().row_data(index as usize) {
                schema_item_vm_clone.borrow_mut().set_selected_schema_id(Some(item.id));
                schema_item_vm_clone.borrow().load();
                if let Some(ui) = ui_weak.upgrade() {
                    let items = schema_item_vm_clone.borrow().get_items();
                    ui.set_detail_list_items(items);
                }
            }
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

    /// 自定义delete方法 - 检查是否是Year Schema
    pub fn delete(&self, index: i32) -> Result<(), String> {
        let index_usize = index as usize;
        
        // 获取要删除的项
        if let Some(item) = self.inner.borrow().items.row_data(index_usize) {
            // 检查是否是Year Schema
            if let Ok(Some(schema)) = self.inner.borrow().get_repo().borrow_mut().find_by_id(item.id) {
                if schema.schema_no == "Year" {
                    return Err("Cannot delete the schema with code 'Year'".to_string());
                }
            }
        }
        
        self.inner.borrow_mut().delete(index_usize)
    }
}

// 使用宏自动生成 CrudViewModelBase trait 实现
// crate::impl_crud_vm_base!(SchemaViewModel, "SchemaViewModel", Schema);

// impl crate::CrudViewModelBase for SchemaViewModel {
//     fn vm_name() -> &'static str {
//         "SchemaViewModel"
//     }

//     fn load(&self) {
//         // Do nothing, we have custom load
//     }

//     fn get_items(&self) -> slint::ModelRc<crate::CrudListItem> {
//         self.inner.borrow().get_items()
//     }

//     fn add(&self) {
//         let mut item = Self::create_default();
//         self.inner.borrow_mut().add(&mut item);
//     }

//     fn delete(&self, index: i32) -> Result<(), String> {
//         self.delete(index)
//     }
// }

use crate::models::schema_item::SchemaItem;
use crate::persistence::schema_item_repository::SchemaItemRepository;

/// SchemaItem管理ViewModel
pub struct SchemaItemViewModel {
    inner: CrudViewModel<SchemaItem, SchemaItemRepository>,
    repo: Rc<RefCell<SchemaItemRepository>>,
    selected_schema_id: Option<i32>,
    selected_index: Option<usize>,
}

impl SchemaItemViewModel {
    /// 创建新的 SchemaItemViewModel 实例
    pub fn new(repo: Rc<RefCell<SchemaItemRepository>>) -> Self {
        let inner = CrudViewModel::new(Rc::clone(&repo));
        Self {
            inner,
            repo,
            selected_schema_id: None,
            selected_index: None,
        }
    }

    /// 更新数据库连接并重新加载数据
    pub fn update_connection(&self, new_conn: Rc<RefCell<diesel::SqliteConnection>>) {
        self.repo.borrow_mut().update_connection(new_conn.clone());
        self.inner.get_repo().borrow_mut().update_connection(new_conn);
        self.load();
    }

    /// 设置选中的schema id
    pub fn set_selected_schema_id(&mut self, schema_id: Option<i32>) {
        self.selected_schema_id = schema_id;
        self.load();
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
        // Add callback - 显示对话框
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_schema_item_add(move || {
            log::info!("SchemaItemViewModel::setup_callbacks: add triggered - showing dialog");
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_add_schema_item_dialog(true);
            }
        });

        // Confirm add schema item callback
        let _vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_confirm_add_schema_item(move |fields| {
            log::info!("SchemaItemViewModel::setup_callbacks: confirm add schema item triggered");
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
                    // 创建新的SchemaItem
                    let mut new_item = SchemaItem {
                        id: 0,
                        schema_id: _vm_clone.borrow().selected_schema_id.unwrap_or(0),
                        item_no: code,
                        item_name: name,
                        created_by: String::new(),
                        created_machine: String::new(),
                        created_at: chrono::Utc::now().naive_utc(),
                    };

                    // 添加到数据库
                    _vm_clone.borrow().inner.add(&mut new_item);
                    log::info!("Successfully added new schema item");
                    // 重新加载数据
                    _vm_clone.borrow().load();
                    let items = _vm_clone.borrow().get_items();
                    ui.set_detail_list_items(items);
                    ui.set_show_add_schema_item_dialog(false);
                } else {
                    log::warn!("Code or name is empty, not adding schema item");
                }
            }
        });

        // Cancel add schema item callback
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_cancel_add_schema_item(move || {
            log::info!("SchemaItemViewModel::setup_callbacks: cancel add schema item triggered");
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_add_schema_item_dialog(false);
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
            let mut item = SchemaItem {
                id: 0,
                schema_id,
                item_no: format!("I{:03}", count),
                item_name: "新条目".to_string(),
                created_by: String::new(),
                created_machine: String::new(),
                created_at: chrono::Utc::now().naive_utc(),
            };
            self.inner.add(&mut item);
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
        if let Some(schema_id) = self.selected_schema_id {
            match self.repo.borrow_mut().find_by_schema_id(schema_id) {
                Ok(items) => self.inner.set_items(items),
                Err(e) => log::error!("Failed to load schema items: {}", e),
            }
        } else {
            self.inner.set_items(vec![]);
        }
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