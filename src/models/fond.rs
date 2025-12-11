use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::{impl_creatable};

table! {
    fonds (id) {
        id -> Integer,
        fond_no -> Text,
        fond_classification_code -> Text,
        name -> Text,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// Fond 实体（全宗）
///
/// 约定：
/// - `id`: 自增主键，创建时设为 0（由数据库自动生成）
/// - `fond_no`: 全宗号，唯一标识
/// - `fond_classification_code`: 关联的全宗分类代码
/// - `name`: 全宗名称
/// - `created_at`: 创建时间，由仓储自动设置
/// - `created_by`: 创建者，由仓储自动设置
/// - `created_machine`: 创建机器，由仓储自动设置
///
/// 使用示例：
/// ```ignore
/// repo.create(Fond {
///     fond_no: "F001".into(),
///     fond_classification_code: "GA".into(),
///     name: "某单位全宗".into(),
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = fonds)]
pub struct Fond {
    pub id: i32,
    pub fond_no: String,
    pub fond_classification_code: String,
    pub name: String,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

impl_creatable!(Fond);

use crate::core::ToCrudListItem;
use crate::CrudListItem;

impl ToCrudListItem for Fond {
    fn to_crud_list_item(&self) -> CrudListItem {
        CrudListItem {
            id: self.id,
            title: self.name.clone().into(),
            subtitle: self.fond_no.clone().into(),
            active: true,
        }
    }
}

