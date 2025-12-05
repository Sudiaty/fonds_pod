use crate::models::fond_schema::{fond_schemas, FondSchema};
use crate::impl_repository;

// 使用宏自动生成 FondSchemasRepository 和 GenericRepository 实现
impl_repository!(
    FondSchemasRepository,                                // 仓储名
    FondSchema,                                            // 实体类型
    fond_schemas,                                          // 表模块
    { fond_no, schema_no, order_no, created_at, created_by, created_machine }, // 插入列（排除 id）
    { fond_no, schema_no, order_no }                       // 更新列
);

// 额外实现 SortableRepository
impl<'a> crate::core::SortableRepository<FondSchema> for FondSchemasRepository<'a> {
    fn update_sort_order(&mut self, id: i32, sort_order: i32) -> Result<(), Box<dyn std::error::Error>> {
        use diesel::prelude::*;
        diesel::update(fond_schemas::table.filter(fond_schemas::id.eq(id)))
            .set(fond_schemas::order_no.eq(sort_order))
            .execute(self.conn)?;
        Ok(())
    }
}
