use crate::core::{CrudViewModel, ToCrudListItem, ActiveableRepository, GenericRepository};
use crate::models::fond_classification::{FondClassification, ClassificationJson};
use crate::persistence::FondClassificationsRepository;
use crate::AppWindow;
use crate::CrudListItem;
use crate::CrudViewModelBase;
use slint::{ComponentHandle, Model, Timer};
use std::cell::RefCell;
use std::rc::Rc;

/// Fond Classification管理ViewModel
///
/// 此ViewModel通过复用CrudViewModelBase trait和宏，提供了极简的实现。
/// 只需实现 `create_default()` 方法来定义新项的默认值。
pub struct FondClassificationViewModel {
    inner: CrudViewModel<FondClassification, FondClassificationsRepository>,
    child_items: Rc<slint::VecModel<CrudListItem>>,
    selected_top_classification_id: Option<i32>,
    repo: Rc<RefCell<FondClassificationsRepository>>,
}

impl FondClassificationViewModel {
    /// 创建新的 FondClassificationViewModel 实例
    pub fn new(repo: Rc<RefCell<FondClassificationsRepository>>) -> Self {
        let inner = CrudViewModel::new(Rc::clone(&repo));
        let child_items = Rc::new(slint::VecModel::default());
        Self {
            inner,
            child_items,
            selected_top_classification_id: None,
            repo,
        }
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

    /// 初始化子分类：如果有顶级分类，自动激活第一个并加载其子分类
    pub fn initialize_child_classifications(&mut self) {
        if let Some(first_item) = self.inner.items.row_data(0) {
            log::info!("Initializing child classifications for first item: id={}, name={}", first_item.id, first_item.title);
            self.load_child_classifications(Some(first_item.id));
        } else {
            log::info!("No top-level classifications found, clearing child items");
            self.child_items.set_vec(Vec::new());
        }
    }

    /// 加载子分类数据
    pub fn load_child_classifications(&mut self, parent_id: Option<i32>) {
        self.selected_top_classification_id = parent_id;
        let child_classifications = {
            let mut repo = self.repo.borrow_mut();
            repo.find_by_parent_id(parent_id).unwrap_or_default()
        };
        let crud_items: Vec<CrudListItem> = child_classifications
            .iter()
            .map(|item| item.to_crud_list_item())
            .collect();
        (*self.child_items).set_vec(crud_items.clone());
        log::info!("Loaded {} child classifications for parent_id {:?}", crud_items.len(), parent_id);
    }

    /// 获取子分类项
    pub fn get_child_items(&self) -> slint::ModelRc<crate::CrudListItem> {
        slint::ModelRc::new(Rc::clone(&self.child_items))
    }

    /// 删除子分类
    pub fn delete_child(&self, index: usize) -> Result<(), String> {
        if let Some(item) = (*self.child_items).row_data(index) {
            let mut repo = self.repo.borrow_mut();
            match repo.delete(item.id) {
                Ok(_) => {
                    (*self.child_items).remove(index);
                    log::info!("Deleted child classification at index {}", index);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to delete child classification at index {}: {}", index, e);
                    Err(format!("删除子分类失败: {}", e))
                }
            }
        } else {
            Err("子分类未找到".to_string())
        }
    }

    /// 激活子分类
    pub fn activate_child(&mut self, id: i32) {
        {
            let mut repo = self.repo.borrow_mut();
            if repo.activate(id).is_ok() {
                log::info!("Activated child classification with id {}", id);
            } else {
                log::error!("Failed to activate child classification with id {}", id);
                return;
            }
        }
        // 重新加载子分类
        self.load_child_classifications(self.selected_top_classification_id);
    }

    /// 停用子分类
    pub fn deactivate_child(&mut self, id: i32) {
        {
            let mut repo = self.repo.borrow_mut();
            if repo.deactivate(id).is_ok() {
                log::info!("Deactivated child classification with id {}", id);
            } else {
                log::error!("Failed to deactivate child classification with id {}", id);
                return;
            }
        }
        // 重新加载子分类
        self.load_child_classifications(self.selected_top_classification_id);
    }

    /// 导出分类到JSON文件
    pub fn export_classifications(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::collections::HashMap;

        let mut repo = self.repo.borrow_mut();

        // 获取所有分类
        let all_classifications = repo.find_all().unwrap_or_default();

        // 构建父子关系映射
        let mut parent_map: HashMap<Option<i32>, Vec<&FondClassification>> = HashMap::new();
        for classification in &all_classifications {
            parent_map.entry(classification.parent_id).or_insert_with(Vec::new).push(classification);
        }

        // 递归构建JSON结构
        fn build_json_tree(
            parent_id: Option<i32>,
            parent_map: &HashMap<Option<i32>, Vec<&FondClassification>>
        ) -> Vec<ClassificationJson> {
            let mut result = Vec::new();
            if let Some(classifications) = parent_map.get(&parent_id) {
                for classification in classifications {
                    let mut json_classification = ClassificationJson::from_fond_classification(classification);
                    json_classification.children = build_json_tree(Some(classification.id), parent_map);
                    result.push(json_classification);
                }
            }
            result
        }

        let json_classifications = build_json_tree(None, &parent_map);

        // 序列化为JSON
        let json_string = serde_json::to_string_pretty(&json_classifications)?;

        // 写入文件
        std::fs::write(file_path, json_string)?;

        log::info!("Successfully exported {} top-level classifications to {}", json_classifications.len(), file_path);
        Ok(())
    }

    /// 从JSON文件导入分类
    pub fn import_classifications(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 读取JSON文件
        let json_string = std::fs::read_to_string(file_path)?;
        let json_classifications: Vec<ClassificationJson> = serde_json::from_str(&json_string)?;

        let mut repo = self.repo.borrow_mut();

        // 在导入前清理所有现有数据
        log::info!("Clearing existing classification data before import");
        repo.delete_all()?;

        // 递归导入分类
        fn import_recursive(
            classifications: &[ClassificationJson],
            parent_id: Option<i32>,
            sort_order_start: i32,
            repo: &mut FondClassificationsRepository
        ) -> Result<(), Box<dyn std::error::Error>> {
            let mut current_sort_order = sort_order_start;

            for json_classification in classifications {
                // 创建分类
                let fond_classification = json_classification.to_fond_classification(parent_id, current_sort_order);
                let created_id = repo.create(fond_classification)?;
                current_sort_order += 1;

                // 递归导入子分类
                import_recursive(&json_classification.children, Some(created_id), 0, repo)?;
            }

            Ok(())
        }

        // 开始导入
        import_recursive(&json_classifications, None, 0, &mut *repo)?;

        log::info!("Successfully imported classifications from {}", file_path);
        Ok(())
    }

    /// 测试导入功能（用于开发调试）
    pub fn test_import_from_json_string(&self, json_string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_classifications: Vec<ClassificationJson> = serde_json::from_str(json_string)?;

        log::info!("Parsed {} top-level classifications from JSON", json_classifications.len());

        // 递归打印分类结构
        fn print_classifications(classifications: &[ClassificationJson], level: usize) {
            for classification in classifications {
                let indent = "  ".repeat(level);
                log::info!("{}{} ({}) - active: {}", indent, classification.name, classification.code, classification.active);
                if !classification.children.is_empty() {
                    print_classifications(&classification.children, level + 1);
                }
            }
        }

        print_classifications(&json_classifications, 0);
        Ok(())
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

        // Classification item activated callback - 加载子分类
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_classification_item_activated(move |index| {
            log::info!("FondClassificationViewModel::setup_callbacks: classification item activated for index {}", index);
            if let Some(ui) = ui_weak.upgrade() {
                let item_id = {
                    let vm_ref = vm_clone.borrow();
                    vm_ref.get_by_index(index as usize).map(|item| item.id)
                };
                if let Some(id) = item_id {
                    vm_clone.borrow_mut().load_child_classifications(Some(id));
                    let child_items = vm_clone.borrow().get_child_items();
                    ui.set_child_crud_items(child_items);
                }
            }
        });

        // Add child classification callback - 显示添加对话框
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_add_child_classification(move || {
            log::info!("FondClassificationViewModel::setup_callbacks: add child classification triggered - showing dialog");
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_add_child_classification_dialog(true);
            }
        });

        // Confirm add child classification callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_confirm_add_child_classification(move |fields| {
            log::info!("FondClassificationViewModel::setup_callbacks: confirm add child classification triggered");
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
                    let selected_parent_id = {
                        let vm_ref = vm_clone.borrow();
                        vm_ref.selected_top_classification_id
                    };

                    if selected_parent_id.is_none() {
                        log::warn!("No top classification selected, cannot add child classification");
                        return;
                    }

                    // 创建新的子分类
                    let new_classification = FondClassification {
                        id: 0,
                        code,
                        name,
                        parent_id: selected_parent_id,
                        active: true,
                        sort_order: 0,
                        ..Default::default()
                    };

                    // 添加到数据库
                    let create_result = {
                        let repo_ref = vm_clone.borrow().repo.clone();
                        let mut repo = repo_ref.borrow_mut();
                        repo.create(new_classification)
                    };

                    if let Ok(_) = create_result {
                        log::info!("Successfully added new child classification");
                    } else {
                        log::error!("Failed to add new child classification");
                        return;
                    }

                    // 重新加载子分类数据
                    {
                        let parent_id = {
                            let vm_ref = vm_clone.borrow();
                            vm_ref.selected_top_classification_id
                        };
                        vm_clone.borrow_mut().load_child_classifications(parent_id);
                    }
                    let child_items = vm_clone.borrow().get_child_items();
                    ui.set_child_crud_items(child_items);
                    ui.set_show_add_child_classification_dialog(false);
                } else {
                    log::warn!("Code or name is empty, not adding child classification");
                }
            }
        });

        // Cancel add child classification callback
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_cancel_add_child_classification(move || {
            log::info!("FondClassificationViewModel::setup_callbacks: cancel add child classification triggered");
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_show_add_child_classification_dialog(false);
            }
        });

        // Delete child classification callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_delete_child_classification(move |idx| {
            log::info!("FondClassificationViewModel::setup_callbacks: delete child classification triggered for index {}", idx);
            if let Some(ui) = ui_weak.upgrade() {
                match vm_clone.borrow().delete_child(idx as usize) {
                    Ok(_) => {
                        let child_items = vm_clone.borrow().get_child_items();
                        ui.set_child_crud_items(child_items);
                    }
                    Err(e) => {
                        ui.set_toast_message(e.into());
                        ui.set_toast_visible(true);
                        
                        let ui_weak_clone = ui_weak.clone();
                        Timer::single_shot(std::time::Duration::from_secs(3), move || {
                            if let Some(ui) = ui_weak_clone.upgrade() {
                                ui.set_toast_visible(false);
                            }
                        });
                    }
                }
            }
        });

        // Activate child classification callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_activate_child_classification(move |id| {
            log::info!("FondClassificationViewModel::setup_callbacks: activate child classification triggered for id {}", id);
            if let Some(ui) = ui_weak.upgrade() {
                vm_clone.borrow_mut().activate_child(id);
                let child_items = vm_clone.borrow().get_child_items();
                ui.set_child_crud_items(child_items);
            }
        });

        // Deactivate child classification callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_deactivate_child_classification(move |id| {
            log::info!("FondClassificationViewModel::setup_callbacks: deactivate child classification triggered for id {}", id);
            if let Some(ui) = ui_weak.upgrade() {
                vm_clone.borrow_mut().deactivate_child(id);
                let child_items = vm_clone.borrow().get_child_items();
                ui.set_child_crud_items(child_items);
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
                match vm_clone.borrow().delete(idx) {
                    Ok(_) => {
                        let items = vm_clone.borrow().get_items();
                        log::info!(
                            "FondClassificationViewModel::setup_callbacks: Setting {} items to UI",
                            items.row_count()
                        );
                        ui.set_classification_crud_items(items);
                    }
                    Err(e) => {
                        ui.set_toast_message(e.into());
                        ui.set_toast_visible(true);
                        
                        let ui_weak_clone = ui_weak.clone();
                        Timer::single_shot(std::time::Duration::from_secs(3), move || {
                            if let Some(ui) = ui_weak_clone.upgrade() {
                                ui.set_toast_visible(false);
                            }
                        });
                    }
                }
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

        // Export classifications callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_export_classifications(move || {
            log::info!("FondClassificationViewModel::setup_callbacks: export classifications triggered");
            // 使用文件对话框选择保存位置
            use rfd::FileDialog;
            if let Some(path) = FileDialog::new()
                .add_filter("JSON files", &["json"])
                .set_file_name("fond_classifications_export.json")
                .save_file() {
                match vm_clone.borrow().export_classifications(&path.to_string_lossy()) {
                    Ok(_) => {
                        log::info!("Successfully exported classifications to {}", path.display());
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_toast_message("分类导出成功".into());
                            ui.set_toast_visible(true);
                            
                            let ui_weak_clone = ui_weak.clone();
                            Timer::single_shot(std::time::Duration::from_secs(3), move || {
                                if let Some(ui) = ui_weak_clone.upgrade() {
                                    ui.set_toast_visible(false);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to export classifications: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_toast_message(format!("分类导出失败: {}", e).into());
                            ui.set_toast_visible(true);
                            
                            let ui_weak_clone = ui_weak.clone();
                            Timer::single_shot(std::time::Duration::from_secs(3), move || {
                                if let Some(ui) = ui_weak_clone.upgrade() {
                                    ui.set_toast_visible(false);
                                }
                            });
                        }
                    }
                }
            }
        });

        // Import classifications callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_import_classifications(move || {
            log::info!("FondClassificationViewModel::setup_callbacks: import classifications triggered");
            // 使用文件对话框选择文件
            use rfd::FileDialog;
            if let Some(path) = FileDialog::new()
                .add_filter("JSON files", &["json"])
                .pick_file() {
                // 先执行导入操作
                let import_result = vm_clone.borrow().import_classifications(&path.to_string_lossy());

                match import_result {
                    Ok(_) => {
                        log::info!("Successfully imported classifications from {}", path.display());
                        // 重新加载数据
                        if let Some(ui) = ui_weak.upgrade() {
                            // 重置ViewModel状态 - 分离借用操作
                            {
                                vm_clone.borrow_mut().selected_top_classification_id = None;
                            }
                            {
                                vm_clone.borrow().child_items.set_vec(Vec::new());
                            }

                            // 重新加载顶级分类数据
                            vm_clone.borrow().load();

                            let items = vm_clone.borrow().get_items();
                            ui.set_classification_crud_items(items);

                            // 更新子分类UI（显示空列表）
                            let child_items = vm_clone.borrow().get_child_items();
                            ui.set_child_crud_items(child_items);

                            ui.set_toast_message("分类导入成功".into());
                            ui.set_toast_visible(true);
                            
                            let ui_weak_clone = ui_weak.clone();
                            Timer::single_shot(std::time::Duration::from_secs(3), move || {
                                if let Some(ui) = ui_weak_clone.upgrade() {
                                    ui.set_toast_visible(false);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to import classifications: {}", e);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_toast_message(format!("分类导入失败: {}", e).into());
                            ui.set_toast_visible(true);
                            
                            let ui_weak_clone = ui_weak.clone();
                            Timer::single_shot(std::time::Duration::from_secs(3), move || {
                                if let Some(ui) = ui_weak_clone.upgrade() {
                                    ui.set_toast_visible(false);
                                }
                            });
                        }
                    }
                }
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
