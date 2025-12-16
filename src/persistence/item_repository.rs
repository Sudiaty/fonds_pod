use crate::models::item::{items, Item};
use crate::impl_repository;

// 使用宏自动生成 ItemsRepository 和 GenericRepository 实现
impl_repository!(
    ItemsRepository,                                      // 仓储名
    Item,                                                  // 实体类型
    items,                                                 // 表模块
    { item_no, file_no, file_id, name, path, created_at, created_by, created_machine }, // 插入列（排除 id）
    { item_no, file_no, file_id, name, path }              // 更新列
);
