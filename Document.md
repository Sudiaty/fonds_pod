# Core 包使用说明

Core 包提供了 FondsPod 项目的核心抽象和通用功能，主要包括：

- **Creatable Trait**: 定义具有审计字段的实体接口
- **GenericRepository Trait**: 通用仓储接口，提供 CRUD 操作
- **宏**: 自动实现 trait 和仓储的代码生成工具

## 核心组件

### 1. Creatable Trait

`Creatable` trait 定义了具有审计功能的实体必须实现的接口。

#### 字段约定

实体必须包含以下字段：
- `id: i32` - 自增主键
- `created_by: String` - 创建者用户名
- `created_machine: String` - 创建机器名
- `created_at: NaiveDateTime` - 创建时间

#### 使用示例

```rust
use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct MyEntity {
    pub id: i32,
    pub name: String,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

// 使用宏自动实现 Creatable trait
impl_creatable!(MyEntity);
```

### 2. GenericRepository Trait

`GenericRepository<E>` 提供了通用的数据访问接口。

#### 主要方法

- `create(entity: E) -> Result<i32, Error>` - 创建记录，自动设置审计字段
- `insert(entity: &E) -> Result<i32, Error>` - 插入记录（不设置审计字段）
- `find_by_id(id: i32) -> Result<Option<E>, Error>` - 根据 ID 查找
- `find_all() -> Result<Vec<E>, Error>` - 查找所有记录
- `find_by_predicate<P>(predicate: P) -> Result<Vec<E>, Error>` - 根据条件过滤
- `update(entity: &E) -> Result<(), Error>` - 更新记录
- `delete(id: i32) -> Result<(), Error>` - 删除记录

#### 辅助方法

- `get_current_user() -> Result<String, Error>` - 获取当前用户名
- `get_current_machine() -> Result<String, Error>` - 获取当前机器名

## 使用 GenericRepository

### 1. 定义实体

```rust
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Queryable, Default)]
#[diesel(table_name = my_entities)]
pub struct MyEntity {
    pub id: i32,
    pub name: String,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

// 实现 Creatable trait
impl_creatable!(MyEntity);

// 定义表结构
table! {
    my_entities (id) {
        id -> Integer,
        name -> Text,
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}
```

### 2. 创建仓储

```rust
use diesel::prelude::*;

// 使用宏生成仓储
impl_repository!(
    MyEntityRepository,           // 仓储名称
    MyEntity,                     // 实体类型
    my_entities,                  // 表模块
    { name, created_at, created_by, created_machine }, // 插入列
    { name }                      // 更新列
);
```

### 3. 使用仓储

```rust
use fonds_pod::core::{GenericRepository, Creatable};

// 创建仓储实例
let mut conn = establish_connection()?;
let mut repo = MyEntityRepository::new(&mut conn);

// 创建记录（自动设置审计字段）
let entity_id = repo.create(MyEntity {
    name: "Test Entity".into(),
    ..Default::default()
})?;

// 查找记录
if let Some(entity) = repo.find_by_id(entity_id)? {
    println!("Found: {}", entity.name);
}

// 查找所有记录
let all_entities = repo.find_all()?;

// 根据条件过滤
let filtered = repo.find_by_predicate(|e| e.name.contains("Test"))?;

// 更新记录
let mut entity = all_entities[0].clone();
entity.set_name("Updated Name".into());
repo.update(&entity)?;

// 删除记录
repo.delete(entity_id)?;
```

## 高级用法

### 自定义仓储方法

```rust
impl<'a> MyEntityRepository<'a> {
    // 自定义方法
    pub fn find_by_name(&mut self, name: &str) -> Result<Vec<MyEntity>, Box<dyn Error>> {
        self.find_by_predicate(|e| e.name == name)
    }

    pub fn find_active(&mut self) -> Result<Vec<MyEntity>, Box<dyn Error>> {
        // 这里可以实现更复杂的逻辑
        self.find_all() // 暂时返回所有记录
    }
}
```

### 组合条件查询

```rust
// 使用 find_by_predicate 进行复杂条件过滤
let results = repo.find_by_predicate(|entity| {
    entity.name.starts_with("Test") &&
    entity.created_at > some_date &&
    entity.id > 100
})?;
```

## 设计原则

### 审计跟踪

所有通过 `create()` 方法创建的记录都会自动设置：
- `created_at`: 当前本地时间（毫秒精度）
- `created_by`: 当前系统用户名
- `created_machine`: 当前机器名

### 内存过滤 vs 数据库过滤

- `find_by_predicate`: 在内存中过滤，适合数据量小或需要复杂逻辑的场景
- 自定义方法: 可以实现数据库级别的过滤以提高性能

### 类型安全

使用泛型确保编译时类型检查，避免运行时错误。

## 依赖

- `chrono`: 时间处理
- `diesel`: ORM 框架
- `serde`: 序列化支持

## 注意事项

1. 实体必须实现 `Creatable` trait
2. 数据库表必须有相应的审计字段
3. 插入时会自动排除 `id` 字段（由数据库自增）
4. 更新时会根据 `id` 字段进行匹配

## 示例项目

参考 `src/models/schema.rs` 和 `src/persistence/schema_repository.rs` 查看完整实现示例。
