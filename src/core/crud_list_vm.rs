//! CRUD List ViewModel - 通用CRUD列表视图模型
//!
//! 提供与 `CrudList` UI 组件配合使用的通用 ViewModel 逻辑，
//! 支持与 `GenericRepository` 和 `ActiveableRepository` 集成。

use std::error::Error;
use std::marker::PhantomData;
use crate::core::{Creatable, GenericRepository, Activeable, ActiveableRepository};

// ============================================================================
// UI 列表项 Trait
// ============================================================================

/// UI 列表项 trait - 约定可转换为 CrudListItem 的数据结构
///
/// 业务实体需实现此 trait 以支持在 CrudList 中显示
pub trait CrudListItemConvertible {
    /// 获取 ID（用于数据库操作）
    fn crud_id(&self) -> i32;
    
    /// 获取标题（显示在列表项第一行）
    fn crud_title(&self) -> String;
    
    /// 获取副标题（显示在列表项第二行）
    fn crud_subtitle(&self) -> String;
    
    /// 获取激活状态（用于显示激活/停用状态）
    fn crud_active(&self) -> bool;
}

/// 自动为实现了 `Creatable + Activeable` 的实体实现 `CrudListItemConvertible`
/// 需要实体额外提供 title 和 subtitle
pub trait CrudListDisplayable: Creatable + Activeable {
    /// 获取显示标题
    fn display_title(&self) -> String;
    
    /// 获取显示副标题
    fn display_subtitle(&self) -> String;
}

/// 为 `CrudListDisplayable` 自动实现 `CrudListItemConvertible`
impl<T: CrudListDisplayable> CrudListItemConvertible for T {
    fn crud_id(&self) -> i32 {
        self.id()
    }
    
    fn crud_title(&self) -> String {
        self.display_title()
    }
    
    fn crud_subtitle(&self) -> String {
        self.display_subtitle()
    }
    
    fn crud_active(&self) -> bool {
        self.active()
    }
}

// ============================================================================
// CRUD 列表操作结果
// ============================================================================

/// CRUD 操作结果，包含成功信息或错误
pub type CrudResult<T> = Result<T, Box<dyn Error>>;

// ============================================================================
// 通用 CRUD 列表操作 Trait
// ============================================================================

/// 通用 CRUD 列表操作 trait
/// 
/// 提供与 `GenericRepository` 集成的标准 CRUD 操作
pub trait CrudListOperations<E: Creatable> {
    /// 获取仓储（用于执行数据库操作）
    type Repository: GenericRepository<E>;
    
    /// 执行仓储操作的辅助方法
    fn with_repository<F, R>(&self, f: F) -> CrudResult<R>
    where
        F: FnOnce(&mut Self::Repository) -> CrudResult<R>;
    
    /// 加载所有记录
    fn load_all(&self) -> CrudResult<Vec<E>> {
        self.with_repository(|repo| repo.find_all())
    }
    
    /// 根据 ID 查找记录
    fn find_by_id(&self, id: i32) -> CrudResult<Option<E>> {
        self.with_repository(|repo| repo.find_by_id(id))
    }
    
    /// 创建记录
    fn create(&self, entity: E) -> CrudResult<i32> {
        self.with_repository(|repo| repo.create(entity))
    }
    
    /// 更新记录
    fn update(&self, entity: &E) -> CrudResult<()> {
        self.with_repository(|repo| repo.update(entity))
    }
    
    /// 删除记录
    fn delete(&self, id: i32) -> CrudResult<()> {
        self.with_repository(|repo| repo.delete(id))
    }
}

/// 可激活的 CRUD 列表操作 trait
/// 
/// 提供与 `ActiveableRepository` 集成的激活/停用操作
pub trait CrudListActiveableOperations<E: Creatable + Activeable>: CrudListOperations<E> 
where
    Self::Repository: ActiveableRepository<E>,
{
    /// 激活记录
    fn activate(&self, id: i32) -> CrudResult<()> {
        self.with_repository(|repo| repo.activate(id))
    }
    
    /// 停用记录
    fn deactivate(&self, id: i32) -> CrudResult<()> {
        self.with_repository(|repo| repo.deactivate(id))
    }
    
    /// 查找所有激活的记录
    fn find_active(&self) -> CrudResult<Vec<E>> {
        self.with_repository(|repo| repo.find_active())
    }
    
    /// 查找所有停用的记录
    fn find_inactive(&self) -> CrudResult<Vec<E>> {
        self.with_repository(|repo| repo.find_inactive())
    }
}

// ============================================================================
// 通用 CRUD 列表 ViewModel 状态
// ============================================================================

/// 通用 CRUD 列表状态
/// 
/// 管理列表项和选中状态
#[derive(Clone, Debug)]
pub struct CrudListState<E> {
    /// 所有列表项
    pub items: Vec<E>,
    /// 当前选中的索引（-1 表示未选中）
    pub selected_index: i32,
    /// 标记类型
    _marker: PhantomData<E>,
}

impl<E> Default for CrudListState<E> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: -1,
            _marker: PhantomData,
        }
    }
}

impl<E> CrudListState<E> {
    /// 创建新的列表状态
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 设置列表项
    pub fn set_items(&mut self, items: Vec<E>) {
        self.items = items;
        // 如果有项目，默认选中第一个
        if !self.items.is_empty() && self.selected_index < 0 {
            self.selected_index = 0;
        } else if self.items.is_empty() {
            self.selected_index = -1;
        } else if self.selected_index as usize >= self.items.len() {
            self.selected_index = (self.items.len() - 1) as i32;
        }
    }
    
    /// 获取当前选中的项目
    pub fn selected_item(&self) -> Option<&E> {
        if self.selected_index >= 0 && (self.selected_index as usize) < self.items.len() {
            Some(&self.items[self.selected_index as usize])
        } else {
            None
        }
    }
    
    /// 获取当前选中项目的可变引用
    pub fn selected_item_mut(&mut self) -> Option<&mut E> {
        if self.selected_index >= 0 && (self.selected_index as usize) < self.items.len() {
            Some(&mut self.items[self.selected_index as usize])
        } else {
            None
        }
    }
    
    /// 设置选中索引
    pub fn set_selected_index(&mut self, index: i32) {
        if index < 0 || (index as usize) < self.items.len() {
            self.selected_index = index;
        }
    }
    
    /// 清空列表
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected_index = -1;
    }
    
    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    /// 获取项目数量
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<E: CrudListItemConvertible> CrudListState<E> {
    /// 获取选中项目的 ID
    pub fn selected_id(&self) -> Option<i32> {
        self.selected_item().map(|e| e.crud_id())
    }
}

// ============================================================================
// 数据库连接辅助
// ============================================================================

/// 数据库路径管理 trait
/// 
/// 用于 ViewModel 管理数据库连接路径
pub trait DatabasePathProvider {
    /// 获取当前数据库路径
    fn get_db_path(&self) -> Option<&str>;
    
    /// 设置当前数据库路径
    fn set_db_path(&mut self, path: Option<String>);
    
    /// 加载数据库路径（从设置服务等）
    fn load_db_path(&mut self) -> CrudResult<()>;
}

// ============================================================================
// 宏：自动实现 CrudListDisplayable
// ============================================================================

/// 为实体自动实现 CrudListDisplayable trait
///
/// # 参数
/// - `$entity`: 实体类型
/// - `$title_field`: 作为标题的字段名
/// - `$subtitle_field`: 作为副标题的字段名
///
/// # 示例
/// ```ignore
/// impl_crud_list_displayable!(FondClassification, code, name);
/// ```
#[macro_export]
macro_rules! impl_crud_list_displayable {
    ($entity:ty, $title_field:ident, $subtitle_field:ident) => {
        impl $crate::core::CrudListDisplayable for $entity {
            fn display_title(&self) -> String {
                self.$title_field.clone()
            }
            
            fn display_subtitle(&self) -> String {
                self.$subtitle_field.clone()
            }
        }
    };
}

// ============================================================================
// UI 回调辅助
// ============================================================================

/// CRUD 列表 UI 回调消息
#[derive(Clone, Debug)]
pub enum CrudListMessage {
    /// 项目被点击
    ItemClicked(i32),
    /// 项目被激活（双击或回车）
    ItemActivated(i32),
    /// 添加按钮点击
    AddClicked,
    /// 删除按钮点击
    DeleteClicked,
    /// 激活项目
    Activate(i32),
    /// 停用项目
    Deactivate(i32),
    /// 刷新列表
    Refresh,
}

/// CRUD 列表操作回调 trait
/// 
/// 用于处理 UI 事件并执行相应操作
pub trait CrudListCallback<E: Creatable + CrudListItemConvertible> {
    /// 处理消息
    fn handle_message(&mut self, message: CrudListMessage) -> CrudResult<()>;
    
    /// 获取列表状态
    fn get_state(&self) -> &CrudListState<E>;
    
    /// 获取可变列表状态
    fn get_state_mut(&mut self) -> &mut CrudListState<E>;
}
