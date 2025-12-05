use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::{impl_creatable, impl_sortable};

table! {
    schemas (id) {
        id -> Integer,
        schema_no -> Text,
        name -> Text,
        sort_order -> Integer,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// Schema 实体
/// 
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `sort_order`: 排序顺序，数字越小越靠前
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
/// 
/// 使用示例：
/// ```ignore
/// repo.create(Schema { schema_no: "S001".into(), name: "Test".into(), sort_order: 1, ..Default::default() });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = schemas)]
pub struct Schema {
    pub id: i32,
    pub schema_no: String,
    pub name: String,
    pub sort_order: i32,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(Schema);
impl_sortable!(Schema);
