use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::impl_creatable;
use crate::core::Sortable;

table! {
    fond_schemas (id) {
        id -> Integer,
        fond_id -> Integer,
        schema_id -> Integer,
        schema_item_id -> Nullable<Integer>,
        sort_order -> Integer,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// FondSchema 实体（全宗方案关联）
///
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `fond_id`: 全宗ID
/// - `schema_id`: 方案ID
/// - `schema_item_id`: 方案项ID，对于动态schema（如Year）可以为NULL
/// - `sort_order`: 排序顺序，数字越小越靠前
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
///
/// 使用示例：
/// ```ignore
/// repo.create(FondSchema {
///     fond_id: 1,
///     schema_id: 1,
///     schema_item_id: Some(1),  // or None for dynamic schemas
///     sort_order: 1,
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = fond_schemas)]
pub struct FondSchema {
    pub id: i32,
    pub fond_id: i32,
    pub schema_id: i32,
    pub schema_item_id: Option<i32>,
    pub sort_order: i32,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(FondSchema);

impl Sortable for FondSchema {
    fn sort_order(&self) -> i32 {
        self.sort_order
    }

    fn set_sort_order(&mut self, sort_order: i32) {
        self.sort_order = sort_order;
    }
}
