use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::{impl_creatable};

table! {
    items (id) {
        id -> Integer,
        file_id -> Integer,
        item_no -> Text,
        name -> Text,
        path -> Nullable<Text>,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// Item 实体（档案项）
///
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `file_id`: 所属文件的 id，外键引用
/// - `item_no`: 档案项号，唯一标识
/// - `name`: 档案项名称
/// - `path`: 文件路径，可为空
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
///
/// 使用示例：
/// ```ignore
/// repo.create(Item {
///     file_id: 1,
///     item_no: "I001".into(),
///     name: "某档案项".into(),
///     path: Some("/path/to/file".into()),
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = items)]
pub struct Item {
    pub id: i32,
    pub file_id: i32,
    pub item_no: String,
    pub name: String,
    pub path: Option<String>,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(Item);
