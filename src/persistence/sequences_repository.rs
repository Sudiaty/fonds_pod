use crate::models::sequence::{sequences, Sequence};
use std::cell::RefCell;
use std::rc::Rc;
use diesel::prelude::*;
use chrono::Utc;

/// SequencesRepository - 序列仓储
///
/// 用于生成各类业务编号的序列
pub struct SequencesRepository {
    conn: Rc<RefCell<diesel::SqliteConnection>>,
}

impl SequencesRepository {
    pub fn new(conn: Rc<RefCell<diesel::SqliteConnection>>) -> Self {
        SequencesRepository { conn }
    }

    pub fn update_connection(&mut self, new_conn: Rc<RefCell<diesel::SqliteConnection>>) {
        self.conn = new_conn;
    }

    pub fn clone_with_conn(&self, conn: Rc<RefCell<diesel::SqliteConnection>>) -> Self {
        SequencesRepository { conn }
    }

    /// 根据前缀查找序列
    pub fn find_by_prefix(&mut self, prefix: &str) -> Result<Option<Sequence>, Box<dyn std::error::Error>> {
        let result = sequences::table
            .filter(sequences::prefix.eq(prefix))
            .first::<Sequence>(&mut *self.conn.borrow_mut())
            .optional()?;
        Ok(result)
    }

    /// 插入新序列
    pub fn insert(&mut self, sequence: &Sequence) -> Result<i32, Box<dyn std::error::Error>> {
        let result = diesel::insert_into(sequences::table)
            .values(sequence)
            .execute(&mut *self.conn.borrow_mut())?;
        Ok(result as i32)
    }

    /// 更新序列
    pub fn update(&mut self, sequence: &Sequence) -> Result<(), Box<dyn std::error::Error>> {
        diesel::update(sequences::table.filter(sequences::id.eq(sequence.id)))
            .set((
                sequences::next_value.eq(sequence.next_value),
                sequences::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut *self.conn.borrow_mut())?;
        Ok(())
    }

    /// 获取下一个编号
    /// 如果序列不存在，会自动创建
    pub fn get_next_number(&mut self, prefix: &str, digits: Option<i32>) -> Result<String, Box<dyn std::error::Error>> {
        let digits = digits.unwrap_or(2);
        
        // 尝试查找现有序列
        if let Some(mut seq) = self.find_by_prefix(prefix)? {
            // 获取当前值并递增
            let current_value = seq.next_value;
            seq.next_value += 1;
            self.update(&seq)?;
            
            // 格式化编号
            Ok(format!("{}{:0width$}", prefix, current_value, width = digits as usize))
        } else {
            // 创建新序列
            let now = Utc::now().naive_utc();
            let new_seq = Sequence {
                id: 0, // 会被自动设置
                prefix: prefix.to_string(),
                next_value: 2, // 下次使用2
                digits,
                created_at: now,
                updated_at: now,
            };
            
            self.insert(&new_seq)?;
            
            // 返回第一个编号
            Ok(format!("{}{:0width$}", prefix, 1, width = digits as usize))
        }
    }

    /// 重置序列（用于测试或初始化）
    pub fn reset_sequence(&mut self, prefix: &str, start_value: i32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut seq) = self.find_by_prefix(prefix)? {
            seq.next_value = start_value;
            self.update(&seq)?;
        } else {
            let now = Utc::now().naive_utc();
            let new_seq = Sequence {
                id: 0,
                prefix: prefix.to_string(),
                next_value: start_value,
                digits: 2,
                created_at: now,
                updated_at: now,
            };
            self.insert(&new_seq)?;
        }
        Ok(())
    }
}