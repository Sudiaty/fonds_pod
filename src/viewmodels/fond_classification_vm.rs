//! Fond Classification View Model - MVVM architecture (Simplified)
//! 
//! Uses generic CRUD operations from core module for simplified implementation.

use crate::models::fond_classification::FondClassification;
use crate::persistence::fond_classification_repository::FondClassificationsRepository;
use crate::services::SettingsService;
use crate::{AppWindow, CrudListItem, DialogField};
use crate::core::{
    GenericRepository, ActiveableRepository, 
    CrudListState, CrudListItemConvertible, CrudResult,
};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;

// ============================================================================
// UI Item - 简化的 UI 显示项
// ============================================================================

/// Fond classification UI item with CrudListItemConvertible implementation
#[derive(Clone, Debug)]
pub struct FondClassificationUIItem {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub parent_id: Option<i32>,
    pub active: bool,
}

impl From<&FondClassification> for FondClassificationUIItem {
    fn from(c: &FondClassification) -> Self {
        Self { id: c.id, code: c.code.clone(), name: c.name.clone(), parent_id: c.parent_id, active: c.active }
    }
}

impl CrudListItemConvertible for FondClassificationUIItem {
    fn crud_id(&self) -> i32 { self.id }
    fn crud_title(&self) -> String { self.code.clone() }
    fn crud_subtitle(&self) -> String { self.name.clone() }
    fn crud_active(&self) -> bool { self.active }
}

// ============================================================================
// ViewModel - 精简实现
// ============================================================================

/// Fond Classification ViewModel using generic CrudListState
#[derive(Clone)]
pub struct FondClassificationViewModel {
    pub top_state: CrudListState<FondClassificationUIItem>,
    pub child_state: CrudListState<FondClassificationUIItem>,
    pub current_db_path: Option<String>,
    settings_service: Rc<SettingsService>,
}

impl Default for FondClassificationViewModel {
    fn default() -> Self {
        Self {
            top_state: CrudListState::new(),
            child_state: CrudListState::new(),
            current_db_path: None,
            settings_service: Rc::new(SettingsService::new()),
        }
    }
}

impl FondClassificationViewModel {
    pub fn new(settings_service: Rc<SettingsService>) -> Self {
        Self { settings_service, ..Default::default() }
    }

    // ========================================================================
    // 仓储操作 - 通用辅助方法
    // ========================================================================
    
    fn with_repo<F, R>(&mut self, f: F) -> CrudResult<R>
    where F: FnOnce(&mut FondClassificationsRepository) -> CrudResult<R>
    {
        if self.current_db_path.is_none() {
            self.current_db_path = self.settings_service.get_last_opened_library()?;
        }
        let db_path = self.current_db_path.as_ref().ok_or("未选择数据库")?;
        let db_file = Path::new(db_path).join(".fondspod.db");
        let mut conn = crate::persistence::establish_connection(&db_file)?;
        let mut repo = FondClassificationsRepository::new(&mut conn);
        f(&mut repo)
    }

    // ========================================================================
    // CRUD 操作 - 统一接口
    // ========================================================================

    pub fn load_all(&mut self) -> CrudResult<Vec<FondClassification>> {
        self.with_repo(|repo| repo.find_all())
    }

    pub fn delete(&mut self, id: i32) -> CrudResult<()> {
        self.with_repo(|repo| repo.delete(id))
    }

    pub fn create(&mut self, entity: FondClassification) -> CrudResult<i32> {
        self.with_repo(|repo| repo.create(entity))
    }

    pub fn activate(&mut self, id: i32) -> CrudResult<()> {
        self.with_repo(|repo| repo.activate(id))
    }

    pub fn deactivate(&mut self, id: i32) -> CrudResult<()> {
        self.with_repo(|repo| repo.deactivate(id))
    }

    // ========================================================================
    // 数据加载
    // ========================================================================

    pub fn load_classifications(&mut self) -> CrudResult<()> {
        let all = self.load_all()?;
        self.top_state.set_items(all.iter().filter(|c| c.parent_id.is_none()).map(Into::into).collect());
        self.child_state.set_items(all.iter().filter(|c| c.parent_id.is_some()).map(Into::into).collect());
        Ok(())
    }

    pub fn load_children_for(&mut self, parent_id: i32) -> CrudResult<()> {
        let all = self.load_all()?;
        self.child_state.set_items(all.iter().filter(|c| c.parent_id == Some(parent_id)).map(Into::into).collect());
        Ok(())
    }

    /// Called when page is entered - load data and update UI
    pub fn on_page_enter(&mut self, ui: &AppWindow) {
        if self.load_classifications().is_err() { return; }
        self.update_ui(ui);
        // Auto-load child classifications for first top classification
        if let Some(first) = self.top_state.items.first() {
            let pid = first.id;
            self.top_state.set_selected_index(0);
            if self.load_children_for(pid).is_ok() { self.update_child_ui(ui); }
        }
    }

    // ========================================================================
    // UI 更新辅助
    // ========================================================================

    fn to_crud_items(items: &[FondClassificationUIItem]) -> ModelRc<CrudListItem> {
        let vec: Vec<CrudListItem> = items.iter().map(|c| CrudListItem {
            id: c.crud_id(), title: c.crud_title().into(), subtitle: c.crud_subtitle().into(), active: c.crud_active(),
        }).collect();
        ModelRc::new(VecModel::from(vec))
    }

    pub fn update_ui(&self, ui: &AppWindow) {
        ui.set_classification_crud_items(Self::to_crud_items(&self.top_state.items));
        ui.set_child_crud_items(Self::to_crud_items(&self.child_state.items));
    }

    pub fn update_child_ui(&self, ui: &AppWindow) {
        ui.set_child_crud_items(Self::to_crud_items(&self.child_state.items));
    }

    // ========================================================================
    // 添加分类
    // ========================================================================

    fn extract_code_name(fields: &[DialogField]) -> CrudResult<(String, String)> {
        let code = fields.iter().find(|f| f.label == "label_code").map(|f| f.value.to_string()).unwrap_or_default();
        let name = fields.iter().find(|f| f.label == "label_name").map(|f| f.value.to_string()).unwrap_or_default();
        if code.trim().is_empty() || name.trim().is_empty() {
            return Err("分类代码和名称不能为空".into());
        }
        Ok((code, name))
    }

    pub fn add_top(&mut self, fields: &[DialogField]) -> CrudResult<()> {
        let (code, name) = Self::extract_code_name(fields)?;
        let existing = self.load_all()?;
        if existing.iter().any(|c| c.code == code) { return Err("分类代码已存在".into()); }
        self.create(FondClassification { code, name, parent_id: None, active: true, sort_order: existing.len() as i32, ..Default::default() })?;
        Ok(())
    }

    pub fn add_child(&mut self, fields: &[DialogField]) -> CrudResult<()> {
        let (code, name) = Self::extract_code_name(fields)?;
        let existing = self.load_all()?;
        if existing.iter().any(|c| c.code == code) { return Err("分类代码已存在".into()); }
        
        let parent_id = self.top_state.selected_item().map(|s| s.id)
            .or_else(|| self.top_state.items.first().map(|s| s.id))
            .or_else(|| existing.iter().find(|c| c.parent_id.is_none()).map(|c| c.id))
            .ok_or("没有顶级分类可用，请先添加顶级分类")?;
        
        self.create(FondClassification { code, name, parent_id: Some(parent_id), active: true, sort_order: existing.len() as i32, ..Default::default() })?;
        Ok(())
    }

    // ========================================================================
    // UI Callbacks - 精简版
    // ========================================================================

    pub fn setup_callbacks(vm: Rc<RefCell<Self>>, ui: &AppWindow) {
        vm.borrow().update_ui(ui);
        
        // 加载分类
        ui.on_load_fond_classifications({ let vm = vm.clone(); let ui = ui.as_weak();
            move || { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                if vm.load_classifications().is_ok() { vm.update_ui(&ui); }
            }}
        });

        // 顶级分类点击
        ui.on_classification_item_clicked({ let vm = vm.clone(); let ui = ui.as_weak();
            move |idx| { if let Some(ui) = ui.upgrade() { vm.borrow_mut().top_state.set_selected_index(idx); ui.set_selected_classification(idx); }}
        });

        // 顶级分类激活 - 加载子分类
        ui.on_classification_item_activated({ let vm = vm.clone(); let ui = ui.as_weak();
            move |idx| { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                vm.top_state.set_selected_index(idx);
                if let Some(item) = vm.top_state.selected_item() {
                    let pid = item.id;
                    if vm.load_children_for(pid).is_ok() { vm.update_child_ui(&ui); }
                }
            }}
        });

        // 选择分类
        ui.on_select_fond_classification({ let vm = vm.clone();
            move |idx| { vm.borrow_mut().top_state.set_selected_index(idx); }
        });

        // 删除顶级分类
        ui.on_delete_fond_classification({ let vm = vm.clone(); let ui = ui.as_weak();
            move || { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                if let Some(id) = vm.top_state.selected_id() {
                    if vm.delete(id).is_ok() && vm.load_classifications().is_ok() {
                        vm.update_ui(&ui); ui.invoke_show_toast("分类删除成功".into());
                    }
                } else { ui.invoke_show_toast("未选择分类".into()); }
            }}
        });

        // 激活/停用顶级分类
        ui.on_activate_top_classification({ let vm = vm.clone(); let ui = ui.as_weak();
            move |id| { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                if vm.activate(id).is_ok() && vm.load_classifications().is_ok() { vm.update_ui(&ui); }
            }}
        });
        ui.on_deactivate_top_classification({ let vm = vm.clone(); let ui = ui.as_weak();
            move |id| { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                if vm.deactivate(id).is_ok() && vm.load_classifications().is_ok() { vm.update_ui(&ui); }
            }}
        });

        // 子分类点击
        ui.on_child_classification_clicked({ let vm = vm.clone(); let ui = ui.as_weak();
            move |idx| { if let Some(ui) = ui.upgrade() { vm.borrow_mut().child_state.set_selected_index(idx); ui.set_selected_child(idx); }}
        });
        ui.on_child_classification_activated({ let vm = vm.clone();
            move |idx| { vm.borrow_mut().child_state.set_selected_index(idx); }
        });

        // 删除子分类
        ui.on_delete_child_classification({ let vm = vm.clone(); let ui = ui.as_weak();
            move || { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                if let Some(id) = vm.child_state.selected_id() {
                    if vm.delete(id).is_ok() && vm.load_classifications().is_ok() {
                        vm.update_ui(&ui); ui.invoke_show_toast("子分类删除成功".into());
                    }
                } else { ui.invoke_show_toast("未选择子分类".into()); }
            }}
        });

        // 激活/停用子分类
        ui.on_activate_child_classification({ let vm = vm.clone(); let ui = ui.as_weak();
            move |id| { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                if vm.activate(id).is_ok() {
                    if let Some(pid) = vm.top_state.selected_item().map(|s| s.id) { let _ = vm.load_children_for(pid); }
                    vm.update_child_ui(&ui);
                }
            }}
        });
        ui.on_deactivate_child_classification({ let vm = vm.clone(); let ui = ui.as_weak();
            move |id| { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                if vm.deactivate(id).is_ok() {
                    if let Some(pid) = vm.top_state.selected_item().map(|s| s.id) { let _ = vm.load_children_for(pid); }
                    vm.update_child_ui(&ui);
                }
            }}
        });

        // 添加分类对话框
        ui.on_add_classification({ let ui = ui.as_weak();
            move || { if let Some(ui) = ui.upgrade() { ui.set_show_add_top_classification_dialog(true); }}
        });
        ui.on_add_child_classification({ let ui = ui.as_weak();
            move || { if let Some(ui) = ui.upgrade() { ui.set_show_add_child_classification_dialog(true); }}
        });

        // 确认/取消添加顶级分类
        ui.on_confirm_add_top_classification({ let vm = vm.clone(); let ui = ui.as_weak();
            move |fields| { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                let fields_vec: Vec<DialogField> = (0..fields.row_count()).filter_map(|i| fields.row_data(i)).collect();
                match vm.add_top(&fields_vec) {
                    Ok(_) => { if vm.load_classifications().is_ok() { vm.update_ui(&ui); ui.invoke_show_toast("顶级分类添加成功".into()); }}
                    Err(e) => ui.invoke_show_toast(format!("添加失败: {}", e).into()),
                }
                ui.set_show_add_top_classification_dialog(false);
            }}
        });
        ui.on_cancel_add_top_classification({ let ui = ui.as_weak();
            move || { if let Some(ui) = ui.upgrade() { ui.set_show_add_top_classification_dialog(false); }}
        });

        // 确认/取消添加子分类
        ui.on_confirm_add_child_classification({ let vm = vm.clone(); let ui = ui.as_weak();
            move |fields| { if let Some(ui) = ui.upgrade() { let mut vm = vm.borrow_mut();
                vm.top_state.set_selected_index(ui.get_selected_classification());
                let fields_vec: Vec<DialogField> = (0..fields.row_count()).filter_map(|i| fields.row_data(i)).collect();
                match vm.add_child(&fields_vec) {
                    Ok(_) => {
                        if let Some(pid) = vm.top_state.selected_item().map(|s| s.id) { let _ = vm.load_children_for(pid); }
                        vm.update_child_ui(&ui); ui.invoke_show_toast("子分类添加成功".into());
                    }
                    Err(e) => ui.invoke_show_toast(format!("添加失败: {}", e).into()),
                }
                ui.set_show_add_child_classification_dialog(false);
            }}
        });
        ui.on_cancel_add_child_classification({ let ui = ui.as_weak();
            move || { if let Some(ui) = ui.upgrade() { ui.set_show_add_child_classification_dialog(false); }}
        });

        // 导入/导出（待实现）
        ui.on_export_classifications({ let ui = ui.as_weak();
            move || { if let Some(ui) = ui.upgrade() { ui.invoke_show_toast("导出分类功能待实现".into()); }}
        });
        ui.on_import_classifications({ let ui = ui.as_weak();
            move || { if let Some(ui) = ui.upgrade() { ui.invoke_show_toast("导入分类功能待实现".into()); }}
        });
    }
}
