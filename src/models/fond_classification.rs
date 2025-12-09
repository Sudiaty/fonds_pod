use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::{impl_creatable, impl_activeable, impl_sortable, impl_crud_list_displayable};

table! {
    fond_classifications (id) {
        id -> Integer,
        code -> Text,
        name -> Text,
        parent_id -> Nullable<Integer>,
        active -> Bool,
        sort_order -> Integer,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// FondClassification 实体
///
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `code`: 分类代码，唯一
/// - `name`: 分类名称
/// - `parent_id`: 父分类ID，可为空（引用fond_classifications.id）
/// - `active`: 是否激活状态
/// - `sort_order`: 排序顺序，数字越小越靠前
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
///
/// 使用示例：
/// ```ignore
/// repo.create(FondClassification {
///     code: "GA".into(),
///     name: "文化".into(),
///     parent_id: Some(1),
///     active: true,
///     sort_order: 1,
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = fond_classifications)]
pub struct FondClassification {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub parent_id: Option<i32>,
    pub active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(FondClassification);
impl_activeable!(FondClassification);
impl_sortable!(FondClassification);
impl_crud_list_displayable!(FondClassification, code, name);