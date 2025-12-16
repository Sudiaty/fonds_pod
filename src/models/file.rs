use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::{impl_creatable};

table! {
    files (id) {
        id -> Integer,
        series_id -> Integer,
        name -> Text,
        file_no -> Text,
        path -> Nullable<Text>,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// File 实体（文件）
///
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `series_id`: 所属案卷的 id，外键引用
/// - `name`: 文件名称
/// - `file_no`: 文件编号
/// - `path`: 文件路径，可为空
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
///
/// 使用示例：
/// ```ignore
/// repo.create(File {
///     series_id: 1,
///     name: "某文件".into(),
///     file_no: "FILE001".into(),
///     path: Some("/path/to/file".into()),
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = files)]
pub struct File {
    pub id: i32,
    pub series_id: i32,
    pub name: String,
    pub file_no: String,
    pub path: Option<String>,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(File);
