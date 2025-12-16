use crate::models::sequence::Sequence;
use crate::persistence::SequencesRepository;
use std::cell::RefCell;
use std::rc::Rc;
use std::error::Error;

/// 编号生成服务
/// 
/// 负责生成各类业务编号，使用 sequences 表维护当前序列值
pub struct SequenceService {
    repo: Rc<RefCell<SequencesRepository>>,
}

impl SequenceService {
    /// 创建新的 SequenceService 实例
    pub fn new(repo: Rc<RefCell<SequencesRepository>>) -> Self {
        Self { repo }
    }

    /// 生成下一个编号
    /// 
    /// # 参数
    /// - `prefix`: 编号前缀（如 "F" 表示全宗，"S" 表示案卷等）
    /// - `digits`: 编号位数，默认 3 位
    /// 
    /// # 返回
    /// 生成的编号字符串，如 "F001", "S042"
    pub fn next_number(&mut self, prefix: &str, digits: usize) -> Result<String, Box<dyn Error>> {

        // 获取或创建序列
        let mut current_value = match self.repo.borrow_mut().find_by_prefix(prefix) {
            Ok(Some(seq)) => seq.current_value,
            Ok(None) => {
                // 创建新序列
                let new_seq = Sequence {
                    prefix: prefix.to_string(),
                    current_value: 0,
                };
                self.repo.borrow_mut().insert(&new_seq)?;
                0
            }
            Err(e) => return Err(e),
        };

        // 递增序列值
        current_value += 1;

        // 更新数据库
        let updated_seq = Sequence {
            prefix: prefix.to_string(),
            current_value,
        };
        self.repo.borrow_mut().update(&updated_seq)?;

        // 格式化编号
        Ok(format!("{}{:0width$}", prefix, current_value, width = digits))
    }

    /// 生成全宗编号 (F001, F002, ...)
    pub fn next_fond_number(&mut self) -> Result<String, Box<dyn Error>> {
        self.next_number("F", 3)
    }

    /// 生成案卷编号 (S001, S002, ...)
    pub fn next_series_number(&mut self) -> Result<String, Box<dyn Error>> {
        self.next_number("S", 3)
    }

    /// 生成文件编号 (格式: [FondNo]-[SeriesNo]-[Two-digit Sequence])
    /// 
    /// # 参数
    /// - `fond_no`: 全宗编号
    /// - `series_no`: 案卷编号
    pub fn next_file_number(&mut self, fond_no: &str, series_no: &str) -> Result<String, Box<dyn Error>> {
        let prefix = format!("{}-{}", fond_no, series_no);
        let number = self.next_number(&prefix, 2)?;
        Ok(number)
    }

    /// 生成档案项编号 (I001, I002, ...)
    pub fn next_item_number(&mut self) -> Result<String, Box<dyn Error>> {
        self.next_number("I", 3)
    }
}