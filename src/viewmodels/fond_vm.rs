use crate::core::CrudViewModel;
use crate::models::Fond;
use crate::persistence::FondsRepository;
use std::rc::Rc;
use std::cell::RefCell;
use slint::{ModelRc, Model, ComponentHandle};
use crate::CrudListItem;
use crate::AppWindow;

pub struct FondViewModel {
    inner: CrudViewModel<Fond, FondsRepository>,
}

impl FondViewModel {
    /// 创建新的 FondViewModel 实例
    pub fn new(repo: Rc<RefCell<FondsRepository>>) -> Self {
        let inner = CrudViewModel::new(repo);
        Self { inner }
    }

    /// 加载所有全宗数据
    pub fn load(&self) {
        log::info!("FondViewModel: Loading fonds data");
        self.inner.load();
        log::info!("FondViewModel: Loaded {} fonds", self.inner.items.row_count());
    }

    /// 添加新的全宗
    pub fn add(&self) {
        log::info!("FondViewModel: Adding new fond");
        let new_fond = Fond {
            id: 0,
            fond_no: format!("F{:03}", chrono::Local::now().timestamp() % 1000),
            fond_classification_code: String::new(),
            name: "新全宗".to_string(),
            ..Default::default()
        };
        self.inner.add(new_fond);
        log::info!("FondViewModel: Added fond, total count: {}", self.inner.items.row_count());
    }
    
    /// 根据索引删除全宗
    pub fn delete(&self, index: i32) {
        log::info!("FondViewModel: Deleting fond at index {}", index);
        if index >= 0 {
            self.inner.delete(index as usize);
            log::info!("FondViewModel: Deleted fond, remaining count: {}", self.inner.items.row_count());
        }
    }

    /// 更新全宗（预留方法）
    #[allow(unused_variables)]
    pub fn update(&self, index: usize, fond: Fond) {
        // TODO: 在 CrudViewModel 中添加 update 方法
        // self.inner.update(index, fond);
    }

    /// 根据索引获取全宗（预留方法）
    pub fn get_by_index(&self, index: usize) -> Option<CrudListItem> {
        self.inner.items.row_data(index)
    }

    /// 获取所有全宗列表项
    pub fn get_items(&self) -> ModelRc<CrudListItem> {
        self.inner.get_items()
    }

    /// 刷新数据
    pub fn refresh(&self) {
        self.load();
    }

    /// 设置UI回调
    pub fn setup_callbacks(vm: Rc<RefCell<Self>>, ui_handle: &AppWindow) {
        // Add callback
        let fond_vm = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_fond_add(move || {
            log::info!("FondViewModel::setup_callbacks: fond_add triggered");
            if let Some(ui) = ui_weak.upgrade() {
                fond_vm.borrow().add();
                let items = fond_vm.borrow().get_items();
                log::info!("FondViewModel::setup_callbacks: Setting {} fond items to UI", items.row_count());
                ui.set_fond_items(items);
            }
        });
        
        // Delete callback
        let fond_vm = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_fond_delete(move |idx| {
            log::info!("FondViewModel::setup_callbacks: fond_delete triggered for index {}", idx);
            if let Some(ui) = ui_weak.upgrade() {
                fond_vm.borrow().delete(idx);
                let items = fond_vm.borrow().get_items();
                log::info!("FondViewModel::setup_callbacks: Setting {} fond items to UI", items.row_count());
                ui.set_fond_items(items);
            }
        });
    }
}
