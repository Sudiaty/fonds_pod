use chrono::{NaiveDateTime, Timelike};
use std::error::Error;

// ============================================================================
// Core Trait - Creatable（约定实体有 id 和 created_at 字段）
// ============================================================================

/// Creatable trait - 约定实体必须有 id、created_by、created_machine 和 created_at 字段
///
/// 约定：
/// - `id`: i32 类型，作为自增主键
/// - `created_by`: String 类型，记录创建者用户名
/// - `created_machine`: String 类型，记录创建机器名
/// - `created_at`: NaiveDateTime 类型，记录创建时间
pub trait Creatable {
    /// 获取 id
    fn id(&self) -> i32;

    /// 设置 id
    fn set_id(&mut self, id: i32);

    /// 获取 created_by 创建者
    fn created_by(&self) -> &str;

    /// 设置 created_by 创建者
    fn set_created_by(&mut self, created_by: String);

    /// 获取 created_machine 机器名
    fn created_machine(&self) -> &str;

    /// 设置 created_machine 机器名
    fn set_created_machine(&mut self, created_machine: String);

    /// 获取 created_at 时间
    fn created_at(&self) -> NaiveDateTime;

    /// 设置 created_at 时间
    fn set_created_at(&mut self, dt: NaiveDateTime);

    /// 初始化为当前本地时间（默认实现），精度到毫秒
    fn init_timestamp(&mut self) {
        let now = chrono::Local::now().naive_local();
        // 截断到毫秒精度（保留3位小数）
        let now_ms = now.with_nanosecond((now.nanosecond() / 1_000_000) * 1_000_000).unwrap();
        self.set_created_at(now_ms);
    }
}

// ============================================================================
// Repository Trait - 通用仓储接口
// ============================================================================

/// 通用仓储 trait
///
/// 泛型参数:
/// - E: 实体类型（实现 Creatable，包含 id、created_at、created_by 和 created_machine）
pub trait GenericRepository<E: Creatable> {
    /// 获取当前用户名（默认实现返回 Windows 用户名）
    fn get_current_user() -> Result<String, Box<dyn Error>> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("USERNAME").map_err(|e| Box::new(e) as Box<dyn Error>)
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::var("USER").or_else(|_| std::env::var("LOGNAME")).map_err(|e| Box::new(e) as Box<dyn Error>)
        }
    }

    /// 获取当前机器名（默认实现返回计算机名）
    fn get_current_machine() -> Result<String, Box<dyn Error>> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("COMPUTERNAME").map_err(|e| Box::new(e) as Box<dyn Error>)
        }
        #[cfg(not(target_os = "windows"))]
        {
            use std::process::Command;
            let output = Command::new("hostname").output().map_err(|e| Box::new(e) as Box<dyn Error>)?;
            String::from_utf8(output.stdout).map(|s| s.trim().to_string()).map_err(|e| Box::new(e) as Box<dyn Error>)
        }
    }

    /// 插入记录（不修改时间戳），返回插入后的 id
    fn insert(&mut self, entity: &E) -> Result<i32, Box<dyn Error>>;

    /// 创建记录（自动设置 created_at、created_by 和 created_machine），返回插入后的 id
    fn create(&mut self, mut entity: E) -> Result<i32, Box<dyn Error>> {
        entity.init_timestamp();

        // 设置创建者
        if let Ok(user) = Self::get_current_user() {
            entity.set_created_by(user);
        }

        // 设置机器名
        if let Ok(machine) = Self::get_current_machine() {
            entity.set_created_machine(machine);
        }

        self.insert(&entity)
    }

    /// 根据 id 查找记录
    fn find_by_id(&mut self, id: i32) -> Result<Option<E>, Box<dyn Error>>;

    /// 查找所有记录
    fn find_all(&mut self) -> Result<Vec<E>, Box<dyn Error>>;

    /// 根据过滤条件查找记录（在内存中过滤）
    /// 
    /// # 参数
    /// - `predicate`: 过滤谓词，返回 true 表示保留该记录
    /// 
    /// # 示例
    /// ```ignore
    /// let results = repo.find_by_predicate(|s| s.name.contains("Test"))?;
    /// ```
    fn find_by_predicate<P>(&mut self, predicate: P) -> Result<Vec<E>, Box<dyn Error>>
    where
        P: Fn(&E) -> bool,
    {
        let all = self.find_all()?;
        Ok(all.into_iter().filter(predicate).collect())
    }

    /// 更新记录（根据 id）
    fn update(&mut self, entity: &E) -> Result<(), Box<dyn Error>>;

    /// 根据 id 删除记录
    fn delete(&mut self, id: i32) -> Result<(), Box<dyn Error>>;
}

// ============================================================================
// 宏：自动实现 Creatable
// ============================================================================

/// 为实体自动实现 Creatable trait
///
/// 约定实体必须有 `id: i32`、`created_by: String`、`created_machine: String` 和 `created_at: NaiveDateTime` 字段
///
/// # 示例
/// ```ignore
/// impl_creatable!(Schema);
/// ```
#[macro_export]
macro_rules! impl_creatable {
    ($entity:ty) => {
        impl crate::core::Creatable for $entity {
            fn id(&self) -> i32 {
                self.id
            }

            fn set_id(&mut self, id: i32) {
                self.id = id;
            }

            fn created_by(&self) -> &str {
                &self.created_by
            }

            fn set_created_by(&mut self, created_by: String) {
                self.created_by = created_by;
            }

            fn created_machine(&self) -> &str {
                &self.created_machine
            }

            fn set_created_machine(&mut self, created_machine: String) {
                self.created_machine = created_machine;
            }

            fn created_at(&self) -> chrono::NaiveDateTime {
                self.created_at
            }

            fn set_created_at(&mut self, dt: chrono::NaiveDateTime) {
                self.created_at = dt;
            }
        }
    };
}

// ============================================================================
// 宏：自动生成仓储实现
// ============================================================================

/// 为实体自动生成仓储结构体和 GenericRepository 实现
///
/// 约定：
/// - 表必须有 `id` 列作为自增主键
/// - 实体类型必须实现 Creatable trait
/// - 插入时自动排除 id 列（由数据库自动生成）
///
/// # 参数
/// - `$repo`: 仓储结构体名称
/// - `$entity`: 实体类型（同时用于查询和插入）
/// - `$table`: Diesel 表模块
/// - `$insert_cols`: 插入时的列（排除 id）
/// - `$update_cols`: 更新时的列
///
/// # 示例
/// ```ignore
/// impl_repository!(
///     SchemaRepository,       // 仓储名
///     Schema,                 // 实体类型
///     schemas,                // 表模块
///     { schema_no, name, created_at },  // 插入列（排除 id）
///     { schema_no, name }               // 更新列
/// );
/// ```
#[macro_export]
macro_rules! impl_repository {
    (
        $repo:ident,
        $entity:ty,
        $table:ident,
        { $( $insert_col:ident ),* $(,)? },
        { $( $update_col:ident ),* $(,)? }
    ) => {
        use ::std::rc::Rc;
        use ::std::cell::RefCell;

        pub struct $repo {
            conn: Rc<RefCell<diesel::SqliteConnection>>,
        }

        impl $repo {
            pub fn new(conn: Rc<RefCell<diesel::SqliteConnection>>) -> Self {
                $repo { conn }
            }

            pub fn update_connection(&mut self, new_conn: Rc<RefCell<diesel::SqliteConnection>>) {
                self.conn = new_conn;
            }
        }

        impl crate::core::GenericRepository<$entity> for $repo {
            fn insert(&mut self, entity: &$entity) -> Result<i32, Box<dyn std::error::Error>> {
                use diesel::prelude::*;
                // 插入时只插入指定列，排除 id（由数据库自增）
                diesel::insert_into($table::table)
                    .values(( $( $table::$insert_col.eq(&entity.$insert_col), )* ))
                    .execute(&mut *self.conn.borrow_mut())?;
                // 获取最后插入的 id
                let last_id: i32 = diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
                    .get_result(&mut *self.conn.borrow_mut())?;
                Ok(last_id)
            }

            fn find_by_id(&mut self, id: i32) -> Result<Option<$entity>, Box<dyn std::error::Error>> {
                use diesel::prelude::*;
                let result = $table::table
                    .filter($table::id.eq(id))
                    .first::<$entity>(&mut *self.conn.borrow_mut())
                    .optional()?;
                Ok(result)
            }

            fn find_all(&mut self) -> Result<Vec<$entity>, Box<dyn std::error::Error>> {
                use diesel::prelude::*;
                let results = $table::table.load::<$entity>(&mut *self.conn.borrow_mut())?;
                Ok(results)
            }

            fn update(&mut self, entity: &$entity) -> Result<(), Box<dyn std::error::Error>> {
                use diesel::prelude::*;
                use crate::core::Creatable;
                diesel::update($table::table.filter($table::id.eq(entity.id())))
                    .set(( $( $table::$update_col.eq(&entity.$update_col), )* ))
                    .execute(&mut *self.conn.borrow_mut())?;
                Ok(())
            }

            fn delete(&mut self, id: i32) -> Result<(), Box<dyn std::error::Error>> {
                use diesel::prelude::*;
                diesel::delete($table::table.filter($table::id.eq(id)))
                    .execute(&mut *self.conn.borrow_mut())?;
                Ok(())
            }
        }
    };
}
