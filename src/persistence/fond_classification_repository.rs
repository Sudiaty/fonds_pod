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