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

impl SchemaRepository {
    /// 自定义删除方法，防止删除code为Year的Schema
    pub fn delete(&mut self, id: i32) -> Result<(), Box<dyn std::error::Error>> {
        // 首先检查是否是code为Year的Schema
        use diesel::prelude::*;
        let schema = schemas::table
            .filter(schemas::id.eq(id))
            .first::<Schema>(&mut *self.conn.borrow_mut())?;
        
        if schema.schema_no == "Year" {
            return Err("Cannot delete the schema with code 'Year'".into());
        }
        
        // 调用父类的delete
        <Self as crate::core::GenericRepository<Schema>>::delete(self, id)
    }
}
