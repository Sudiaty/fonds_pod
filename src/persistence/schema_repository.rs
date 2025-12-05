use crate::models::schema::{schemas, Schema};
use crate::impl_sortable_repository;

#[allow(unused_imports)]
use crate::core::SortableRepository;


// 使用宏自动生成 SchemaRepository结构体和 SortableRepository 实现
impl_sortable_repository!(
    SchemaRepository,                                     // 仓储名
    Schema,                                                   // 实体类型
    schemas,                                                  // 表模块
    { schema_no, name, sort_order, created_at, created_by, created_machine }, // 插入列（排除 id）
    { schema_no, name, sort_order }                                       // 更新列
);
