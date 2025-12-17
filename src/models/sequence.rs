use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

table! {
    sequences (id) {
        id -> Integer,
        prefix -> Text,
        next_value -> Integer,
        digits -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

/// Sequence 实体（序列生成器）
///
/// 用于生成各类业务编号的序列
///
/// 字段：
/// - `id`: 主键
/// - `prefix`: 序列前缀（如 "F01", "F001-001-" 等）
/// - `next_value`: 下一个序列值
/// - `digits`: 流水号位数，缺省为2
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = sequences)]
pub struct Sequence {
    pub id: i32,
    pub prefix: String,
    pub next_value: i32,
    pub digits: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
