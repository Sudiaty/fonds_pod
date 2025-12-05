use crate::models::file::{files, File};
use crate::impl_repository;

// 使用宏自动生成 FilesRepository 和 GenericRepository 实现
impl_repository!(
    FilesRepository,                                      // 仓储名
    File,                                                  // 实体类型
    files,                                                 // 表模块
    { file_no, series_no, name, created_at, created_by, created_machine }, // 插入列（排除 id）
    { file_no, series_no, name }                           // 更新列
);
