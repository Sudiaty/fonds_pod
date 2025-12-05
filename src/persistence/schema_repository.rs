use crate::models::schema::{schemas, Schema};
use crate::impl_repository;


// 使用宏自动生成 SchemaRepository结构体和 GenericRepository 实现
impl_repository!(
    SchemaRepository,                                     // 仓储名
    Schema,                                                   // 实体类型
    schemas,                                                  // 表模块
    { schema_no, name, created_at, created_by, created_machine }, // 插入列（排除 id）
    { schema_no, name }                                       // 更新列
);
