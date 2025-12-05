use crate::models::fond::{fonds, Fond};
use crate::impl_repository;

// 使用宏自动生成 FondsRepository 和 GenericRepository 实现
impl_repository!(
    FondsRepository,                                      // 仓储名
    Fond,                                                  // 实体类型
    fonds,                                                 // 表模块
    { fond_no, fond_classification_code, name, created_at, created_by, created_machine }, // 插入列（排除 id）
    { fond_no, fond_classification_code, name }            // 更新列
);
