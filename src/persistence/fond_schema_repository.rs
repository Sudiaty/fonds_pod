use crate::models::fond_schema::{fond_schemas, FondSchema};
use crate::impl_repository;

// 使用宏自动生成 FondSchemasRepository 和 GenericRepository 实现
impl_repository!(
    FondSchemasRepository,                                // 仓储名
    FondSchema,                                            // 实体类型
    fond_schemas,                                          // 表模块
    { fond_id, schema_id, schema_item_id, sort_order, created_at, created_by, created_machine }, // 插入列（排除 id）
    { fond_id, schema_id, schema_item_id, sort_order }                       // 更新列
);

// 额外实现 SortableRepository
impl crate::core::SortableRepository<FondSchema> for FondSchemasRepository {
    fn update_sort_order(&mut self, id: i32, sort_order: i32) -> Result<(), Box<dyn std::error::Error>> {
        use diesel::prelude::*;
        diesel::update(fond_schemas::table.filter(fond_schemas::id.eq(id)))
            .set(fond_schemas::sort_order.eq(sort_order))
            .execute(&mut *self.conn.borrow_mut())?;
        Ok(())
    }
}
