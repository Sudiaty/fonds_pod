use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::{impl_creatable};

table! {
    files (id) {
        id -> Integer,
        file_no -> Text,
        series_no -> Text,
        series_id -> Integer,
        name -> Text,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// File 实体（文件）
///
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `file_no`: 文件号，唯一标识
/// - `series_no`: 所属案卷号
/// - `name`: 文件名称
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
///
/// 使用示例：
/// ```ignore
/// repo.create(File {
///     file_no: "W001".into(),
///     series_no: "J001".into(),
///     name: "某文件".into(),
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = files)]
pub struct File {
    pub id: i32,
    pub file_no: String,
    pub series_no: String,
    pub series_id: i32,
    pub name: String,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(File);
