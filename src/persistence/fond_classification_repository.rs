use crate::models::fond_classification::{fond_classifications, FondClassification};
use crate::impl_activeable_sortable_repository;

// 使用宏自动生成 FondClassificationsRepository 和 ActiveableRepository + SortableRepository 实现
impl_activeable_sortable_repository!(
    FondClassificationsRepository,                        // 仓储名
    FondClassification,                                   // 实体类型
    fond_classifications,                                 // 表模块
    { code, name, parent_id, active, sort_order, created_at, created_by, created_machine }, // 插入列（排除 id）
    { code, name, parent_id, active, sort_order },         // 更新列
    sort_order                                             // 排序字段
);

// 自定义方法实现
impl FondClassificationsRepository {
    /// 根据父级ID查找子分类
    pub fn find_by_parent_id(&mut self, parent_id_param: Option<i32>) -> Result<Vec<FondClassification>, Box<dyn std::error::Error>> {
        use diesel::prelude::*;
        use crate::models::fond_classification::fond_classifications::dsl::*;

        let mut query = fond_classifications.into_boxed();

        if let Some(pid) = parent_id_param {
            query = query.filter(parent_id.eq(pid));
        } else {
            query = query.filter(parent_id.is_null());
        }

        query.order(sort_order.asc()).load::<FondClassification>(&mut *self.conn.borrow_mut())
            .map_err(|e| e.into())
    }

    /// 删除所有分类数据
    pub fn delete_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use diesel::prelude::*;
        use crate::models::fond_classification::fond_classifications::dsl::*;

        diesel::delete(fond_classifications).execute(&mut *self.conn.borrow_mut())?;
        Ok(())
    }
}