# 项目架构概述

FondsPod 项目采用 MVVM (Model-View-ViewModel) 架构模式，该模式将应用程序分为三个主要层次：

- **Model**: 数据模型和业务逻辑层，负责数据存储、验证和业务规则
- **View**: 用户界面层，使用 Slint 框架构建，专注于展示和用户交互
- **ViewModel**: 视图模型层，连接 Model 和 View，处理用户输入和数据绑定

项目的目录结构层次划分：

- **`src/models/`**: Model 层，包含数据模型定义
  - `fond.rs`, `schema.rs`, `item.rs` 等实体模型

- **`src/persistence/`**: 数据访问层，实现仓储模式
  - `fond_repository.rs`, `schema_repository.rs` 等仓储实现
  - `schema.rs` 数据库连接和配置

- **`src/core/`**: 核心抽象层，提供通用功能
  - `generic_repository.rs`等通用仓储接口

- **`src/viewmodels/`**: ViewModel 层，处理 UI 逻辑
  - 连接 Model 和 View，管理数据绑定和用户交互

- **`src/services/`**: 服务层，提供业务逻辑和配置
  - `settings_service.rs` 配置管理
  - `runtime_translations.rs` 国际化服务

- **`ui/`**: View 层，用户界面实现
  - `app-window.slint` 主窗口
  - `components/`, `pages/`, `layout/` 等 UI 组件
  - `locale/` 国际化资源

- **`migrations/`**: 数据库迁移脚本，确保数据结构一致性

这种目录结构确保了代码的模块化，便于维护和扩展。

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

---

# 国际化 (i18n) 维护指南

FondsPod 使用 GNU gettext 标准实现国际化，所有 UI 文本通过 PO 文件管理。

## 📁 文件结构

```
ui/locale/
├── zh_CN/LC_MESSAGES/fonds-pod.po    # 中文翻译
├── zh_CN/LC_MESSAGES/fonds-pod.mo    # 编译后中文翻译
├── en_US/LC_MESSAGES/fonds-pod.po    # 英文翻译
└── en_US/LC_MESSAGES/fonds-pod.mo    # 编译后英文翻译
```

## 🛠️ 推荐工具

### Poedit (推荐)
- **下载**: https://poedit.net/
- **功能**: 图形化 PO 文件编辑器，支持实时翻译检查
- **优势**: 自动生成 MO 文件，翻译记忆，拼写检查

### 替代工具
- **VS Code**: 安装 "gettext" 扩展
- **命令行**: `msgfmt` (用于生成 MO 文件)

## 📝 日常维护流程

### 1. 添加新文本
```slint
// UI 代码中使用 @tr()
Text { text: @tr("save_button"); }
```

### 2. 更新翻译 (使用 Poedit)
1. 打开 `ui/locale/zh_CN/LC_MESSAGES/fonds-pod.po`
2. Poedit 会自动检测新增的翻译键
3. 填写中文翻译
4. 保存时自动生成 `.mo` 文件

### 3. 编译应用
```bash
cargo build
```

## 🌐 添加新语言

1. **创建目录**: `mkdir -p ui/locale/ja_JP/LC_MESSAGES`
2. **复制模板**: `cp ui/locale/zh_CN/LC_MESSAGES/fonds-pod.po ui/locale/ja_JP/LC_MESSAGES/`
3. **使用 Poedit 编辑**日语翻译
4. **更新应用设置**添加语言选项

## ⚙️ 高级配置

### 翻译键命名
- 小写字母 + 下划线: `save_button`, `cancel_action`
- 描述性名称: `navigation_fonds` (非 `nav_1`)
- 保持一致性

### 复数形式
```po
msgid "item"
msgid_plural "items"
msgstr[0] "%d 项"
msgstr[1] "%d 项"
```

## 🔧 故障排除

### 文本显示为键名
- 检查 PO 文件是否包含对应条目
- 确认 MO 文件已更新 (`msgfmt` 生成)
- 重新编译应用

### 某些文本未翻译
- 确认 UI 代码使用 `@tr()` 而非硬编码文本
- 检查 PO 文件条目是否存在且正确

### 动态文本处理
```slint
Text { text: @tr("count: {0}", number); }
```

## 📋 贡献清单

- ✅ UI 代码使用 `@tr()` 调用
- ✅ 新增翻译键同步更新所有 PO 文件
- ✅ 修改后重新生成 MO 文件
- ✅ 测试翻译功能正常

## 🔗 相关文件

- 运行时翻译: `src/services/runtime_translations.rs`
- 构建配置: `build.rs`
- UI 组件: `ui/` 目录下所有 `.slint` 文件
