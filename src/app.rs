use std::rc::Rc;
use std::cell::RefCell;

use fonds_pod_lib::viewmodels::SchemaViewModel;

use crate::MainWindow;

// 使用类型别名简化 Rc<RefCell<T>> 的使用
type SharedVm<T> = Rc<RefCell<T>>;

pub struct App {
    // 持有共享资源
    // pub db_pool: SqlitePool,
    
    // 持有所有 ViewModels 的共享指针
    pub schema_vm: SharedVm<SchemaViewModel>,
    // pub todo_vm: SharedVm<TodoListViewModel>,
    // pub settings_vm: SharedVm<SettingsViewModel>,
    // ... 其他 VM ...
}

impl App {
    pub fn initialize(ui_handle: &MainWindow) -> Self {
        // 在这里集中初始化所有的 VM
        // let todo_vm = Rc::new(RefCell::new(
        //     TodoListViewModel::new(ui_handle.as_weak(), pool.clone())
        // ));
        
        // let settings_vm = Rc::new(RefCell::new(
        //     SettingsViewModel::new(/* ... */)
        // ));

        App { 
            schema_vm: Rc::new(RefCell::new(
                SchemaViewModel::new()
            )),
        }
    }

    // 设置所有 UI 回调的专用方法
    pub fn setup_ui_callbacks(&self, _ui_handle: &MainWindow) {
        // 将 main.rs 中的回调逻辑移到这里
        // let todo_vm_clone = Rc::clone(&self.todo_vm);
        // ui_handle.on_request_add_task(move |title| {
        //     todo_vm_clone.borrow_mut().add_task(title.into());
        // });
        
        // 设置 settings_vm 的回调...
    }
}
