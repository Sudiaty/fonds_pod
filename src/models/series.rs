use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::{impl_creatable};

table! {
    series (id) {
        id -> Integer,
        series_no -> Text,
        fond_no -> Text,
        name -> Text,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// Series 实体（案卷）
///
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `series_no`: 案卷号，唯一标识
/// - `fond_no`: 所属全宗号
/// - `name`: 案卷名称
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
///
/// 使用示例：
/// ```ignore
/// repo.create(Series {
///     series_no: "J001".into(),
///     fond_no: "F001".into(),
///     name: "某案卷".into(),
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = series)]
pub struct Series {
    pub id: i32,
    pub series_no: String,
    pub fond_no: String,
    pub name: String,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(Series);
