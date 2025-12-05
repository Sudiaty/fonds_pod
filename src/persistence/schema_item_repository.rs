use std::error::Error;
use crate::models::schema_item::{schema_items, SchemaItem};
use crate::impl_repository;
use crate::GenericRepository;

// 使用宏自动生成 SchemaItemRepository 和 GenericRepository 实现
impl_repository!(
    SchemaItemRepository,                                              // 仓储名
    SchemaItem,                                                        // 实体类型
    schema_items,                                                      // 表模块
    { schema_id, item_no, item_name, created_at, created_by, created_machine }, // 插入列（排除 id）
    { schema_id, item_no, item_name }                                  // 更新列
);

// 额外的自定义方法
impl<'a> SchemaItemRepository<'a> {
    /// Find all items belonging to a specific schema
    pub fn find_by_schema_id(&mut self, schema_id_val: i32) -> Result<Vec<SchemaItem>, Box<dyn Error>> {
        self.find_by_predicate(|item| item.schema_id == schema_id_val)
    }
}