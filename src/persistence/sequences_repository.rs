use crate::models::sequence::{sequences, Sequence};
use std::cell::RefCell;
use std::rc::Rc;

/// SequencesRepository - 序列仓储
/// 
/// 注意：Sequence 不实现 Creatable trait，因为 sequences 表没有标准的时间戳字段
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

    /// 根据前缀查找序列
    pub fn find_by_prefix(&mut self, prefix: &str) -> Result<Option<Sequence>, Box<dyn std::error::Error>> {
        use diesel::prelude::*;
        let result = sequences::table
            .filter(sequences::prefix.eq(prefix))
            .first::<Sequence>(&mut *self.conn.borrow_mut())
            .optional()?;
        Ok(result)
    }

    /// 插入新序列
    pub fn insert(&mut self, sequence: &Sequence) -> Result<(), Box<dyn std::error::Error>> {
        use diesel::prelude::*;
        diesel::insert_into(sequences::table)
            .values(sequence)
            .execute(&mut *self.conn.borrow_mut())?;
        Ok(())
    }

    /// 更新序列
    pub fn update(&mut self, sequence: &Sequence) -> Result<(), Box<dyn std::error::Error>> {
        use diesel::prelude::*;
        diesel::update(sequences::table.filter(sequences::prefix.eq(&sequence.prefix)))
            .set(sequences::current_value.eq(sequence.current_value))
            .execute(&mut *self.conn.borrow_mut())?;
        Ok(())
    }
}