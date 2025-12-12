use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::impl_creatable;
use crate::core::ToCrudListItem;
use crate::CrudListItem;

table! {
    schema_items (id) {
        id -> Integer,
        schema_id -> Integer,
        item_no -> Text,
        item_name -> Text,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// SchemaItem 实体
/// 
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `schema_id`: 关联的 Schema id
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
/// 
/// 使用示例：
/// ```ignore
/// repo.create(SchemaItem { schema_id: 1, item_no: "01".into(), item_name: "Name".into(), ..Default::default() });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = schema_items)]
pub struct SchemaItem {
    pub id: i32,
    pub schema_id: i32,
    pub item_no: String,
    pub item_name: String,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(SchemaItem);

impl ToCrudListItem for SchemaItem {
    fn to_crud_list_item(&self) -> CrudListItem {
        CrudListItem {
            id: self.id,
            title: self.item_name.clone().into(),
            subtitle: self.item_no.clone().into(),
            active: true,
        }
    }
}
