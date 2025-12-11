use crate::{Creatable, core::GenericRepository, CrudListItem};
use slint::{ModelRc, Model, VecModel};
use std::rc::Rc;
use std::cell::RefCell;
use std::marker::PhantomData;

/// Trait to convert an entity to a CrudListItem
pub trait ToCrudListItem {
    fn to_crud_list_item(&self) -> CrudListItem;
}

/// CrudViewModelBase trait - 为所有CRUD ViewModel提供通用接口
///
/// 此trait定义了所有CRUD ViewModel应实现的公用操作。
/// 结合宏 `impl_crud_vm!` 可以自动生成标准的实现逻辑。
pub trait CrudViewModelBase {
    /// 获取ViewModel的名称，用于日志输出
    fn vm_name() -> &'static str;

    /// 加载数据
    fn load(&self);

    /// 获取列表项
    fn get_items(&self) -> slint::ModelRc<crate::CrudListItem>;

    /// 添加新项
    fn add(&self);

    /// 删除指定索引的项
    fn delete(&self, index: i32);

    /// 刷新数据（默认实现调用load）
    fn refresh(&self) {
        self.load();
    }
}

pub struct CrudViewModel<T, R>
where
    T: Creatable + Clone + Default + ToCrudListItem + 'static,
    R: GenericRepository<T> + 'static,
{
    pub items: Rc<VecModel<CrudListItem>>,
    repo: Rc<RefCell<R>>,
    _phantom: PhantomData<T>,
}

impl<T, R> Clone for CrudViewModel<T, R>
where
    T: Creatable + Clone + Default + ToCrudListItem + 'static,
    R: GenericRepository<T> + 'static,
{
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            repo: self.repo.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T, R> CrudViewModel<T, R>
where
    T: Creatable + Clone + Default + ToCrudListItem + 'static,
    R: GenericRepository<T> + 'static,
{
    pub fn new(repo: Rc<RefCell<R>>) -> Self {
        Self {
            items: Rc::new(VecModel::default()),
            repo,
            _phantom: PhantomData,
        }
    }

    pub fn load(&self) {
        let items = self.repo.borrow_mut().find_all().unwrap_or_default();
        let crud_items: Vec<CrudListItem> = items.iter().map(|item| item.to_crud_list_item()).collect();
        self.items.set_vec(crud_items);
    }

    pub fn add(&self, mut item: T) {
        let mut repo = self.repo.borrow_mut();
        if let Ok(id) = repo.create(item.clone()) {
            item.set_id(id);
            self.items.push(item.to_crud_list_item());
        }
    }

    pub fn delete(&self, index: usize) {
        if let Some(item) = self.items.row_data(index) {
             let mut repo = self.repo.borrow_mut();
             if repo.delete(item.id).is_ok() {
                 self.items.remove(index);
             }
        }
    }
    
    pub fn get_items(&self) -> ModelRc<CrudListItem> {
        ModelRc::from(self.items.clone())
    }
}

/// 为CrudViewModel<T, R>自动生成标准的load/add/delete/get_items实现
///
/// 此宏需要：
/// - 一个包含 `inner: CrudViewModel<T, R>` 字段的结构体
/// - 一个实现了 `ToCrudListItem` 的实体类型 `T`
/// - 必须提供 `create_default()` 方法来生成新实体
///
/// # 示例
/// ```ignore
/// pub struct FondViewModel {
///     inner: CrudViewModel<Fond, FondsRepository>,
/// }
///
/// impl FondViewModel {
///     fn create_default() -> Fond {
///         Fond {
///             id: 0,
///             fond_no: format!("F{:03}", chrono::Local::now().timestamp() % 1000),
///             ..Default::default()
///         }
///     }
/// }
///
/// impl_crud_vm_base!(FondViewModel, "FondViewModel", Fond);
/// ```
#[macro_export]
macro_rules! impl_crud_vm_base {
    ($vm_type:ty, $vm_name:expr, $entity_type:ty) => {
        impl $crate::core::CrudViewModelBase for $vm_type {
            fn vm_name() -> &'static str {
                $vm_name
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
                let new_item = <$vm_type>::create_default();
                self.inner.add(new_item);
                log::info!(
                    "{}: Added item, total count: {}",
                    Self::vm_name(),
                    self.inner.items.row_count()
                );
            }

            fn delete(&self, index: i32) {
                log::info!("{}: Deleting item at index {}", Self::vm_name(), index);
                if index >= 0 {
                    self.inner.delete(index as usize);
                    log::info!(
                        "{}: Deleted item, remaining count: {}",
                        Self::vm_name(),
                        self.inner.items.row_count()
                    );
                }
            }
        }
    };
}

/// 为CrudViewModel生成setup_callbacks的标准实现
///
/// 此宏自动生成处理UI回调的代码。你需要：
/// 1. ViewModel实现 `CrudViewModelBase` trait
/// 2. AppWindow中定义对应的回调：`on_{vm_callback_name}_add` 和 `on_{vm_callback_name}_delete`
/// 3. AppWindow中定义对应的属性设置器：`set_{vm_callback_name}_items`
///
/// # 示例
/// ```ignore
/// // 在 AppWindow 中应该有：
/// // callback fond_add();
/// // callback fond_delete(int);
/// // property fond_items: [CrudListItem];
///
/// // 此宏生成 setup_callbacks 实现
/// impl_crud_vm_base_impl!(FondViewModel, "FondViewModel");
/// ```
#[macro_export]
macro_rules! impl_crud_vm_base_impl {
    ($vm_type:ty, $log_prefix:expr, $on_add:path, $on_delete:path, $set_items:path) => {
        impl $vm_type {
            /// 为UI设置CRUD回调
            pub fn setup_callbacks(
                vm: std::rc::Rc<std::cell::RefCell<Self>>,
                ui_handle: &$crate::AppWindow,
            ) {
                // Add callback
                let vm_clone = vm.clone();
                let ui_weak = ui_handle.as_weak();

                $on_add(ui_handle, move || {
                    log::info!("{}::setup_callbacks: add triggered", $log_prefix);
                    if let Some(ui) = ui_weak.upgrade() {
                        vm_clone.borrow().add();
                        let items = vm_clone.borrow().get_items();
                        log::info!(
                            "{}::setup_callbacks: Setting {} items to UI",
                            $log_prefix,
                            items.row_count()
                        );
                        $set_items(&ui, items);
                    }
                });

                // Delete callback
                let vm_clone = vm.clone();
                let ui_weak = ui_handle.as_weak();

                $on_delete(ui_handle, move |idx| {
                    log::info!(
                        "{}::setup_callbacks: delete triggered for index {}",
                        $log_prefix,
                        idx
                    );
                    if let Some(ui) = ui_weak.upgrade() {
                        vm_clone.borrow().delete(idx);
                        let items = vm_clone.borrow().get_items();
                        log::info!(
                            "{}::setup_callbacks: Setting {} items to UI",
                            $log_prefix,
                            items.row_count()
                        );
                        $set_items(&ui, items);
                    }
                });
            }
        }
    };
}

/// 简化版本：一个宏同时生成CrudViewModelBase和callbacks
///
/// # 示例
/// ```ignore
/// impl_full_crud_vm!(FondViewModel, Fond, "FondViewModel");
/// ```
#[macro_export]
macro_rules! impl_full_crud_vm {
    ($vm_type:ty, $entity_type:ty, $log_prefix:expr) => {
        impl_crud_vm_base!($vm_type, $log_prefix, $entity_type);
    };
}