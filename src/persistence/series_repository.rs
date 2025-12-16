use crate::models::series::{series, Series};
use crate::impl_repository;

// 使用宏自动生成 SeriesRepository 和 GenericRepository 实现
impl_repository!(
    SeriesRepository,                                     // 仓储名
    Series,                                                // 实体类型
    series,                                                // 表模块
    { fond_id, series_no, name, created_at, created_by, created_machine }, // 插入列（排除 id）
    { fond_id, series_no, name }                       // 更新列
);
