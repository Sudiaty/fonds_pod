use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::impl_creatable;
use crate::core::Sortable;

table! {
    fond_schemas (id) {
        id -> Integer,
        fond_no -> Text,
        schema_no -> Text,
        order_no -> Integer,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// FondSchema 实体（全宗方案关联）
///
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `fond_no`: 全宗号
/// - `schema_no`: 方案号
/// - `order_no`: 排序顺序，数字越小越靠前（等同于 sort_order）
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
///
/// 使用示例：
/// ```ignore
/// repo.create(FondSchema {
///     fond_no: "F001".into(),
///     schema_no: "S001".into(),
///     order_no: 1,
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = fond_schemas)]
pub struct FondSchema {
    pub id: i32,
    pub fond_no: String,
    pub schema_no: String,
    pub order_no: i32,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(FondSchema);

// 手动实现 Sortable，因为字段名是 order_no 而非 sort_order
impl Sortable for FondSchema {
    fn sort_order(&self) -> i32 {
        self.order_no
    }

    fn set_sort_order(&mut self, sort_order: i32) {
        self.order_no = sort_order;
    }
}
