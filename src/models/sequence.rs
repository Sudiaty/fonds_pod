use diesel::prelude::*;
use serde::{Deserialize, Serialize};

table! {
    sequences (prefix) {
        prefix -> Text,
        current_value -> Integer,
    }
}

/// Sequence 实体（序列生成器）
///
/// 用于生成各类业务编号的序列，不实现 Creatable trait
///
/// 约定：
/// - `prefix`: 序列前缀，作为主键（如 "FOND", "SERIES", "FILE" 等）
/// - `current_value`: 当前序列值
///
/// 使用示例：
/// ```ignore
/// // 查询
/// let seq = sequences::table
///     .find("FOND")
///     .first::<Sequence>(conn)?;
///
/// // 插入或更新
/// diesel::insert_into(sequences::table)
///     .values(&Sequence {
///         prefix: "FOND".into(),
///         current_value: 0,
///     })
///     .execute(conn)?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = sequences)]
pub struct Sequence {
    pub prefix: String,
    pub current_value: i32,
}
