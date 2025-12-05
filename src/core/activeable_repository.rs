use std::error::Error;
use crate::core::{Creatable, GenericRepository};

// ============================================================================
// Core Trait - Activeable（约定实体有 active 字段）
// ============================================================================

/// Activeable trait - 约定实体必须有 active 字段
///
/// 约定：
/// - `active`: bool 类型，表示激活状态（true=激活，false=停用）
pub trait Activeable {
    /// 获取 active 状态
    fn active(&self) -> bool;

    /// 设置 active 状态
    fn set_active(&mut self, active: bool);
}

// ============================================================================
// Repository Trait - 可激活仓储接口
// ============================================================================

/// 可激活仓储 trait
///
/// 泛型参数:
/// - E: 实体类型（实现 Creatable 和 Activeable）
pub trait ActiveableRepository<E: Creatable + Activeable>: GenericRepository<E> {
    /// 激活记录（根据 id）
    fn activate(&mut self, id: i32) -> Result<(), Box<dyn Error>>;

    /// 停用记录（根据 id）
    fn deactivate(&mut self, id: i32) -> Result<(), Box<dyn Error>>;

    /// 查找所有激活的记录
    fn find_active(&mut self) -> Result<Vec<E>, Box<dyn Error>> {
        self.find_by_predicate(|e| e.active())
    }

    /// 查找所有停用的记录
    fn find_inactive(&mut self) -> Result<Vec<E>, Box<dyn Error>> {
        self.find_by_predicate(|e| !e.active())
    }
}

// ============================================================================
// 宏：自动实现 Activeable
// ============================================================================

/// 为实体自动实现 Activeable trait
///
/// 约定实体必须有 `active: bool` 字段
///
/// # 示例
/// ```ignore
/// impl_activeable!(Schema);
/// ```
#[macro_export]
macro_rules! impl_activeable {
    ($entity:ty) => {
        impl crate::core::Activeable for $entity {
            fn active(&self) -> bool {
                self.active
            }

            fn set_active(&mut self, active: bool) {
                self.active = active;
            }
        }
    };
}

// ============================================================================
// 宏：自动生成可激活仓储实现
// ============================================================================

/// 为实体自动生成可激活仓储结构体和 ActiveableRepository 实现
///
/// 基于 impl_repository! 宏，扩展添加激活/停用功能
///
/// # 参数
/// - `$repo`: 仓储结构体名称
/// - `$entity`: 实体类型（同时用于查询和插入，必须实现 Creatable 和 Activeable）
/// - `$table`: Diesel 表模块
/// - `$insert_cols`: 插入时的列（排除 id）
/// - `$update_cols`: 更新时的列
///
/// # 示例
/// ```ignore
/// impl_activeable_repository!(
///     SchemaRepository,       // 仓储名
///     Schema,                 // 实体类型
///     schemas,                // 表模块
///     { schema_no, name, created_at, active },  // 插入列（排除 id）
///     { schema_no, name, active }               // 更新列
/// );
/// ```
#[macro_export]
macro_rules! impl_activeable_repository {
    (
        $repo:ident,
        $entity:ty,
        $table:ident,
        { $( $insert_col:ident ),* $(,)? },
        { $( $update_col:ident ),* $(,)? }
    ) => {
        crate::impl_repository!(
            $repo,
            $entity,
            $table,
            { $( $insert_col ),* },
            { $( $update_col ),* }
        );

        impl<'a> crate::core::ActiveableRepository<$entity> for $repo<'a> {
            fn activate(&mut self, id: i32) -> Result<(), Box<dyn std::error::Error>> {
                use diesel::prelude::*;
                diesel::update($table::table.filter($table::id.eq(id)))
                    .set($table::active.eq(true))
                    .execute(self.conn)?;
                Ok(())
            }

            fn deactivate(&mut self, id: i32) -> Result<(), Box<dyn std::error::Error>> {
                use diesel::prelude::*;
                diesel::update($table::table.filter($table::id.eq(id)))
                    .set($table::active.eq(false))
                    .execute(self.conn)?;
                Ok(())
            }
        }
    };
}

// ============================================================================
// 宏：自动生成可激活可排序仓储实现
// ============================================================================

/// 为实体自动生成可激活可排序仓储结构体和 ActiveableRepository + SortableRepository 实现
///
/// 基于 impl_activeable_repository! 宏，扩展添加排序功能
///
/// # 参数
/// - `$repo`: 仓储结构体名称
/// - `$entity`: 实体类型（同时用于查询和插入，必须实现 Creatable, Activeable 和 Sortable）
/// - `$table`: Diesel 表模块
/// - `$insert_cols`: 插入时的列（排除 id）
/// - `$update_cols`: 更新时的列
/// - `$sort_col`: 排序字段名（通常是 sort_order）
///
/// # 示例
/// ```ignore
/// impl_activeable_sortable_repository!(
///     FondClassificationsRepository,                        // 仓储名
///     FondClassification,                                   // 实体类型
///     fond_classifications,                                 // 表模块
///     { code, name, parent_id, active, sort_order, created_at, created_by, created_machine }, // 插入列（排除 id）
///     { code, name, parent_id, active, sort_order },         // 更新列
///     sort_order                                             // 排序字段
/// );
/// ```
#[macro_export]
macro_rules! impl_activeable_sortable_repository {
    (
        $repo:ident,
        $entity:ty,
        $table:ident,
        { $( $insert_col:ident ),* $(,)? },
        { $( $update_col:ident ),* $(,)? },
        $sort_col:ident
    ) => {
        crate::impl_activeable_repository!(
            $repo,
            $entity,
            $table,
            { $( $insert_col ),* },
            { $( $update_col ),* }
        );

        impl<'a> crate::core::SortableRepository<$entity> for $repo<'a> {
            fn update_sort_order(&mut self, id: i32, sort_order: i32) -> Result<(), Box<dyn std::error::Error>> {
                use diesel::prelude::*;
                diesel::update($table::table.filter($table::id.eq(id)))
                    .set($table::$sort_col.eq(sort_order))
                    .execute(self.conn)?;
                Ok(())
            }
        }
    };
}