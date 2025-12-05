use std::error::Error;
use crate::core::{Creatable, GenericRepository};

// ============================================================================
// Core Trait - Sortable（约定实体有 sort_order 字段）
// ============================================================================

/// Sortable trait - 约定实体必须有 sort_order 字段
///
/// 约定：
/// - `sort_order`: i32 类型，表示排序顺序（数字越小越靠前）
pub trait Sortable {
    /// 获取 sort_order
    fn sort_order(&self) -> i32;

    /// 设置 sort_order
    fn set_sort_order(&mut self, sort_order: i32);
}

// ============================================================================
// Repository Trait - 可排序仓储接口
// ============================================================================

/// 可排序仓储 trait
///
/// 泛型参数:
/// - E: 实体类型（实现 Creatable 和 Sortable）
pub trait SortableRepository<E: Creatable + Sortable>: GenericRepository<E> {
    /// 查找所有记录并按 sort_order 升序排序
    fn find_sorted(&mut self) -> Result<Vec<E>, Box<dyn Error>> {
        let mut all = self.find_all()?;
        all.sort_by_key(|e| e.sort_order());
        Ok(all)
    }

    /// 更新记录的排序顺序
    fn update_sort_order(&mut self, id: i32, sort_order: i32) -> Result<(), Box<dyn Error>>;

    /// 重新排序所有记录（根据当前顺序设置 sort_order）
    fn reorder_all(&mut self) -> Result<(), Box<dyn Error>> {
        let mut entities = self.find_all()?;
        entities.sort_by_key(|e| e.sort_order());

        for (index, entity) in entities.into_iter().enumerate() {
            let mut entity = entity;
            entity.set_sort_order(index as i32 + 1);
            self.update(&entity)?;
        }
        Ok(())
    }

    /// 移动记录到新位置（调整其他记录的 sort_order）
    fn move_to_position(&mut self, id: i32, new_position: i32) -> Result<(), Box<dyn Error>> {
        // 获取当前实体
        let mut entity = self.find_by_id(id)?.ok_or("Entity not found")?;
        let current_order = entity.sort_order();

        if current_order == new_position {
            return Ok(());
        }

        // 获取所有记录
        let mut all = self.find_all()?;
        all.sort_by_key(|e| e.sort_order());

        // 移除当前实体
        let entity_index = all.iter().position(|e| e.id() == id).unwrap();
        all.remove(entity_index);

        // 插入到新位置
        let insert_index = if new_position < current_order {
            (new_position - 1) as usize
        } else {
            (new_position - 1) as usize
        };

        entity.set_sort_order(new_position);
        all.insert(insert_index, entity);

        // 重新设置所有 sort_order
        for (index, entity) in all.into_iter().enumerate() {
            let mut entity = entity;
            entity.set_sort_order(index as i32 + 1);
            self.update(&entity)?;
        }

        Ok(())
    }
}

// ============================================================================
// 宏：自动实现 Sortable
// ============================================================================

/// 为实体自动实现 Sortable trait
///
/// 约定实体必须有 `sort_order: i32` 字段
///
/// # 示例
/// ```ignore
/// impl_sortable!(Schema);
/// ```
#[macro_export]
macro_rules! impl_sortable {
    ($entity:ty) => {
        impl crate::core::Sortable for $entity {
            fn sort_order(&self) -> i32 {
                self.sort_order
            }

            fn set_sort_order(&mut self, sort_order: i32) {
                self.sort_order = sort_order;
            }
        }
    };
}

// ============================================================================
// 宏：自动生成可排序仓储实现
// ============================================================================

/// 为实体自动生成可排序仓储结构体和 SortableRepository 实现
///
/// 基于 impl_repository! 宏，扩展添加排序功能
///
/// # 参数
/// - `$repo`: 仓储结构体名称
/// - `$entity`: 实体类型（同时用于查询和插入，必须实现 Creatable 和 Sortable）
/// - `$table`: Diesel 表模块
/// - `$insert_cols`: 插入时的列（排除 id）
/// - `$update_cols`: 更新时的列
///
/// # 示例
/// ```ignore
/// impl_sortable_repository!(
///     SchemaRepository,       // 仓储名
///     Schema,                 // 实体类型
///     schemas,                // 表模块
///     { schema_no, name, sort_order, created_at, created_by, created_machine },  // 插入列（排除 id）
///     { schema_no, name, sort_order }               // 更新列
/// );
/// ```
#[macro_export]
macro_rules! impl_sortable_repository {
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

        impl<'a> crate::core::SortableRepository<$entity> for $repo<'a> {
            fn update_sort_order(&mut self, id: i32, sort_order: i32) -> Result<(), Box<dyn std::error::Error>> {
                use diesel::prelude::*;
                diesel::update($table::table.filter($table::id.eq(id)))
                    .set($table::sort_order.eq(sort_order))
                    .execute(self.conn)?;
                Ok(())
            }
        }
    };
}