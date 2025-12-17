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

## 注意事项

1. 实体必须实现 `Creatable` trait
2. 数据库表必须有相应的审计字段
3. 插入时会自动排除 `id` 字段（由数据库自增）
4. 更新时会根据 `id` 字段进行匹配

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

### 命令行工具 (推荐)
- **slint-tr-extractor**: 提取 Slint 文件中的 @tr 字符串到 pot 文件
- **msgmerge**: 从 pot 文件更新 po 文件
- **msgfmt**: 从 po 文件生成 mo 文件
- **优势**: 自动化、可脚本化、版本控制友好

### 可选工具
- **Poedit**: 图形化 PO 文件编辑器，支持实时翻译检查
  - **下载**: https://poedit.net/
  - **优势**: 自动生成 MO 文件，翻译记忆，拼写检查
- **VS Code**: 安装 "gettext" 扩展

## 📝 日常维护流程

### 1. 添加新文本
```slint
// UI 代码中使用 @tr()
Text { text: @tr("save_button"); }
```

### 2. 提取翻译字符串 (生成 pot 文件)
```bash
# Linux bash 提取所有 Slint 文件中的 @tr 到 pot 文件
find -name \*.slint | xargs slint-tr-extractor -o MY_PROJECT.pot
```

### 3. 更新翻译文件 (从 pot 更新 po)
```bash
# 更新中文 po 文件
msgmerge --update ui/locale/zh_CN/LC_MESSAGES/fonds-pod.po ui/locale/fonds_pod.pot

# 更新英文 po 文件
msgmerge --update ui/locale/en_US/LC_MESSAGES/fonds-pod.po ui/locale/fonds_pod.pot
```

### 4. 生成二进制翻译文件 (po 到 mo)
```bash
# 生成中文 mo 文件
msgfmt --output-file=ui/locale/zh_CN/LC_MESSAGES/fonds-pod.mo ui/locale/zh_CN/LC_MESSAGES/fonds-pod.po

# 生成英文 mo 文件
msgfmt --output-file=ui/locale/en_US/LC_MESSAGES/fonds-pod.mo ui/locale/en_US/LC_MESSAGES/fonds-pod.po
```

### 5. 编译应用
```bash
cargo build
```

### 可选方式：使用 Poedit 图形化工具
1. 打开 `ui/locale/zh_CN/LC_MESSAGES/fonds-pod.po`
2. Poedit 会自动检测新增的翻译键
3. 填写中文翻译
4. 保存时自动生成 `.mo` 文件

## 🌐 添加新语言

### 推荐方式：命令行工具
1. **创建目录**: `mkdir -p ui/locale/ja_JP/LC_MESSAGES`
2. **复制模板**: `cp ui/locale/zh_CN/LC_MESSAGES/fonds-pod.po ui/locale/ja_JP/LC_MESSAGES/`
3. **编辑翻译**: 使用文本编辑器或命令行工具填写日语翻译
4. **生成 mo 文件**: `msgfmt --output-file=ui/locale/ja_JP/LC_MESSAGES/fonds-pod.mo ui/locale/ja_JP/LC_MESSAGES/fonds-pod.po`
5. **更新应用设置**添加语言选项

### 可选方式：使用 Poedit
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
---

# 新增模块开发指南 - 以Fonds（全宗）为例

本章节以 Fonds 模块的开发为例，详细讲解如何在 FondsPod 项目中遵循 MVVM 架构和最佳实践，快速开发新的数据管理模块。

## 📋 概述

Fonds（全宗）是档案学中的核心概念，代表由某个机构或个人积累的档案集合。本章通过开发 Fonds 管理模块，展示了：

- ✅ 如何复用 `CrudViewModel` 和 `GenericRepository` 框架
- ✅ 如何正确处理数据库约束和外键关系
- ✅ 如何分离关注点，保持 App 结构简洁
- ✅ 如何使用日志进行调试和问题诊断

## 🏗️ 架构设计

### 分层设计
:::mermaid
graph TD
    A["UI Layer (Slint)
    FondPage (fond-page.slint)
    - 显示全宗列表
    - 处理用户交互事件"] --> B["ViewModel Layer (Rust)
    FondViewModel
    - 管理业务逻辑
    - 处理UI回调
    - 数据绑定"]
    B --> C["Repository Layer
    FondsRepository (GenericRepository)
    - 数据持久化
    - 数据库操作 CRUD"]
    C --> D["Model Layer
    Fond (数据模型)
    - 实现 Creatable Trait
    - 实现 ToCrudListItem Trait"]
:::

## 📝 Step-by-Step 开发指南

### Step 1: 定义数据模型 (Model)

**文件**: `src/models/fond.rs`

```rust
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::{impl_creatable};

// 定义数据库表
table! {
    fonds (id) {
        id -> Integer,
        fond_no -> Text,                    // 全宗号
        fond_classification_code -> Text,   // 分类代码
        name -> Text,                        // 名称
        created_by -> Text,
        created_machine -> Text,
        created_at -> Timestamp,
    }
}

/// Fond 实体定义
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Default)]
#[diesel(table_name = fonds)]
pub struct Fond {
    pub id: i32,
    pub fond_no: String,
    pub fond_classification_code: String,
    pub name: String,
    pub created_by: String,
    pub created_machine: String,
    pub created_at: NaiveDateTime,
}

// 自动实现 Creatable trait
impl_creatable!(Fond);

// 实现 ToCrudListItem 以支持列表展示
impl ToCrudListItem for Fond {
    fn to_crud_list_item(&self) -> CrudListItem {
        CrudListItem {
            id: self.id,
            title: self.name.clone().into(),
            subtitle: self.fond_no.clone().into(),
            active: true,
        }
    }
}
```

**关键点**:
- ✅ 定义表结构，包含审计字段（`created_by`, `created_machine`, `created_at`）
- ✅ 使用 `#[diesel(table_name)]` 映射到表
- ✅ 实现 `Creatable` trait（通过宏）
- ✅ 实现 `ToCrudListItem` trait 用于UI展示

---

### Step 2: 创建数据仓储 (Repository)

**文件**: `src/persistence/fond_repository.rs`

```rust
use crate::models::fond::{fonds, Fond};
use crate::impl_repository;

// 使用宏自动生成仓储
impl_repository!(
    FondsRepository,                  // 仓储类名
    Fond,                             // 实体类型
    fonds,                            // 表模块
    { fond_no, fond_classification_code, name, created_at, created_by, created_machine },
    { fond_no, fond_classification_code, name }
);
```

**说明**:
- ✅ 使用 `impl_repository!` 宏自动生成CRUD操作
- ✅ 指定插入列（排除自增ID）
- ✅ 指定更新列（仅可修改的字段）

**在 `src/persistence/mod.rs` 中导出**:

```rust
pub mod fond_repository;
pub use fond_repository::FondsRepository;
```

---

### Step 3: 初始化数据库架构 (Database Schema)

**文件**: `src/persistence/schema.rs`

在 `init_schema()` 函数中添加表创建SQL：

```rust
pub fn init_schema(conn: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
    // ... 其他表 ...

    // Create fonds table
    sql_query(
        r#"
        CREATE TABLE IF NOT EXISTS fonds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fond_no TEXT NOT NULL UNIQUE,
            fond_classification_code TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            created_by TEXT NOT NULL,
            created_machine TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(conn)?;

    Ok(())
}
```

**⚠️ 注意事项**:
- ❌ **不要添加外键约束**到空值字段（会导致插入失败）
- ✅ 使用 `DEFAULT ''` 提供默认值
- ✅ 确保审计字段存在

---

### Step 4: 创建 ViewModel

**文件**: `src/viewmodels/fond_vm.rs`

```rust
use crate::core::CrudViewModel;
use crate::models::Fond;
use crate::persistence::FondsRepository;
use std::rc::Rc;
use std::cell::RefCell;
use slint::{Model, ComponentHandle};
use crate::{CrudListItem, AppWindow};

/// Fond（全宗）管理ViewModel
/// 
/// 此ViewModel通过复用CrudViewModelBase trait和宏，提供了极简的实现。
/// 只需实现 `create_default()` 方法来定义新项的默认值。
pub struct FondViewModel {
    inner: CrudViewModel<Fond, FondsRepository>,
}

impl FondViewModel {
    /// 创建新的 FondViewModel 实例
    pub fn new(repo: Rc<RefCell<FondsRepository>>) -> Self {
        let inner = CrudViewModel::new(repo);
        Self { inner }
    }

    /// 创建默认的Fond实例 - 由 `impl_crud_vm_base!` 宏使用
    fn create_default() -> Fond {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        
        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        Fond {
            id: 0,
            fond_no: format!("F{:03}", count),
            fond_classification_code: String::new(),
            name: "新全宗".to_string(),
            ..Default::default()
        }
    }

    /// 根据索引获取全宗项
    pub fn get_by_index(&self, index: usize) -> Option<CrudListItem> {
        self.inner.items.row_data(index)
    }

    /// 为UI设置CRUD回调 - 标准实现在这里
    pub fn setup_callbacks(
        vm: Rc<RefCell<Self>>,
        ui_handle: &AppWindow,
    ) {
        use crate::core::CrudViewModelBase;
        
        // Add callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_fond_add(move || {
            log::info!("FondViewModel::setup_callbacks: add triggered");
            if let Some(ui) = ui_weak.upgrade() {
                vm_clone.borrow().add();
                let items = vm_clone.borrow().get_items();
                log::info!(
                    "FondViewModel::setup_callbacks: Setting {} items to UI",
                    items.row_count()
                );
                ui.set_fond_items(items);
            }
        });

        // Delete callback
        let vm_clone = vm.clone();
        let ui_weak = ui_handle.as_weak();
        ui_handle.on_fond_delete(move |idx| {
            log::info!("FondViewModel::setup_callbacks: delete triggered for index {}", idx);
            if let Some(ui) = ui_weak.upgrade() {
                vm_clone.borrow().delete(idx);
                let items = vm_clone.borrow().get_items();
                log::info!(
                    "FondViewModel::setup_callbacks: Setting {} items to UI",
                    items.row_count()
                );
                ui.set_fond_items(items);
            }
        });
    }
}

// 使用宏自动生成 CrudViewModelBase trait 实现
crate::impl_crud_vm_base!(FondViewModel, "FondViewModel", Fond);
```

**关键设计**:
- ✅ 包装 `CrudViewModel` 获得通用CRUD逻辑
- ✅ 只需实现 `create_default()` 方法定义新实体
- ✅ 使用 `impl_crud_vm_base!` 宏自动生成所有CRUD方法
- ✅ 添加详细的日志用于调试
- ✅ `setup_callbacks()` 处理所有UI交互
- ✅ 各个方法专注单一职责

**宏自动生成的CRUD方法**:
- ✅ `vm_name()` - 返回 "FondViewModel"
- ✅ `load()` - 加载数据并记录日志
- ✅ `get_items()` - 返回UI模型
- ✅ `add()` - 创建新项并添加到数据库/UI
- ✅ `delete()` - 删除指定索引的项
- ✅ `refresh()` - 默认实现调用load()

**在 `src/viewmodels/mod.rs` 中导出**:

```rust
mod fond_vm;
pub use fond_vm::FondViewModel;
```

---

### Step 5: 创建UI界面

**文件**: `ui/pages/fond-page.slint`

```slint
import { CrudList, CrudListItem } from "../components/crud-list.slint";

export component FondPage inherits Rectangle {
    in property <[CrudListItem]> items: [];
    callback add-clicked();
    callback delete-clicked(int);

    CrudList {
        title: @tr("fond_page_title");
        items: root.items;
        add-clicked => { root.add-clicked(); }
        delete-clicked => { root.delete-clicked(self.active-index); }
    }
}
```

**说明**:
- ✅ 复用 `CrudList` 组件
- ✅ 暴露数据绑定属性 `items`
- ✅ 暴露回调 `add-clicked`, `delete-clicked`
- ✅ 使用 `@tr()` 进行国际化

**在主窗口中集成 (`ui/app-window.slint`)**:

```slint
if root.current_page == "fonds" : FondPage {
    width: parent.width;
    height: parent.height;
    items: root.fond_items;
    add-clicked => { root.fond_add(); }
    delete-clicked(idx) => { root.fond_delete(idx); }
}
```

---

### Step 6: 在App中协调初始化

**文件**: `src/app.rs`

```rust
impl App {
    pub fn initialize(ui_handle: &AppWindow) -> Self {
        let settings_service = Rc::new(SettingsService::new());
        
        // 初始化FondViewModel
        let fond_vm = Rc::new(RefCell::new(
            Self::initialize_fond_vm(&settings_service)
        ));
        fond_vm.borrow().load();

        // ... 初始化其他ViewModel ...

        App {
            settings_vm,
            about_vm,
            home_vm,
            fond_vm,
        }
    }

    /// 初始化Fond相关的数据库连接
    fn initialize_fond_vm(settings_service: &SettingsService) -> FondViewModel {
        let db_path = if let Ok(Some(path)) = settings_service.get_last_opened_library() {
            let db = std::path::PathBuf::from(&path).join(".fondspod.db");
            log::info!("App: Using database at: {:?}", db);
            db
        } else {
            log::warn!("App: No last_opened_library found, using in-memory database");
            std::path::PathBuf::from(":memory:")
        };
        
        let conn = fonds_pod_lib::persistence::establish_connection(&db_path)
            .unwrap_or_else(|_| {
                fonds_pod_lib::persistence::establish_connection(
                    &std::path::PathBuf::from(":memory:")
                ).unwrap()
            });

        let repo = Rc::new(RefCell::new(
            fonds_pod_lib::persistence::FondsRepository::new(conn)
        ));
        FondViewModel::new(repo)
    }

    pub fn setup_ui_callbacks(&self, ui_handle: &AppWindow) {
        // ... 其他设置 ...

        // 设置FondViewModel回调
        FondViewModel::setup_callbacks(Rc::clone(&self.fond_vm), ui_handle);

        // 初始化UI数据
        let items = self.fond_vm.borrow().get_items();
        ui_handle.set_fond_items(items);

        // ... 其他回调 ...
    }
}
```

**设计原则**:
- ✅ App 只负责协调，具体逻辑在ViewModel
- ✅ 数据库初始化与ViewModel分离
- ✅ 回调设置委托给各个ViewModel

---

## 🐛 常见问题与解决方案

### 问题1: 外键约束违反导致数据无法插入

**现象**: `fond_classification_code` 为空时，数据库INSERT失败

**根本原因**: Schema中添加了外键约束：
```sql
FOREIGN KEY (fond_classification_code) REFERENCES fond_classifications(code)
```

**解决方案**:
1. 删除外键约束
2. 添加默认值：`fond_classification_code TEXT NOT NULL DEFAULT ''`
3. 重新生成数据库（删除 `.fondspod.db` 文件）

```bash
# 删除旧数据库强制重新创建
Remove-Item "C:\__mig__\.fondspod.db" -ErrorAction SilentlyContinue
cargo run
```

---

### 问题2: UI中显示的数据列表为空

**排查步骤**:

1. **检查日志输出**
   ```bash
   # 应该看到如下日志
   FondViewModel: Loading fonds data
   FondViewModel: Loaded 0 fonds
   Initial setup: Setting 0 fond items to UI
   ```

2. **验证数据库连接**
   ```rust
   log::info!("Using database at: {:?}", db_path);
   ```

3. **检查回调是否触发**
   ```rust
   log::info!("FondViewModel::setup_callbacks: fond_add triggered");
   ```

4. **验证UI绑定**
   - 检查 `ui/app-window.slint` 中的属性绑定
   - 确认 `fond_items` 属性正确传递

---

### 问题3: 点击添加按钮无反应

**检查清单**:

- ✅ `FondViewModel::setup_callbacks()` 是否被调用
- ✅ UI 中的 `on_fond_add` 回调是否注册
- ✅ 数据库连接是否成功建立
- ✅ 是否有权限写入数据库文件

**调试方法**:

```rust
ui_handle.on_fond_add(move || {
    eprintln!("DEBUG: fond_add callback triggered!");  // 加入调试输出
    // ... 业务逻辑 ...
});
```

---

## 🔧 调试技巧

### 1. 启用详细日志

```bash
# 运行时指定日志级别
RUST_LOG=debug cargo run
```

### 2. 在关键位置添加日志

```rust
log::info!("FondViewModel: Loading fonds data");
log::debug!("FondViewModel: Repository found {} items", items.len());
log::warn!("FondViewModel: Database connection warning");
log::error!("FondViewModel: Failed to add fond: {}", error);
```

### 3. 检查数据库状态

```bash
# 使用 SQLite 命令行工具检查数据
sqlite3 "C:\__mig__\.fondspod.db" "SELECT COUNT(*) FROM fonds;"
```

### 4. 添加跟踪点

```rust
pub fn add(&self) {
    eprintln!("TRACE: add() called");
    let new_fond = Fond { /* ... */ };
    eprintln!("TRACE: created fond: {:?}", new_fond);
    self.inner.add(new_fond);
    eprintln!("TRACE: added, count = {}", self.inner.items.row_count());
}
```

---

## ✅ 测试验证检单

开发完成后，按以下步骤进行测试：

- [ ] **编译通过**: `cargo build` 无错误
- [ ] **数据库创建**: `.fondspod.db` 文件正确生成
- [ ] **数据加载**: 启动时日志显示 "Loaded X fonds"
- [ ] **添加功能**: 点击 "+" 按钮，新项目出现在列表中
- [ ] **删除功能**: 选择项目并删除，列表更新
- [ ] **数据持久化**: 重启应用，数据仍然存在
- [ ] **日志完整**: 日志输出清晰，便于问题诊断
- [ ] **UI响应**: 所有操作反馈及时，无卡顿

---

## 📚 相关参考

- **Core 抽象层**: `src/core/` - `CrudViewModel`, `GenericRepository`
- **数据模型示例**: `src/models/schema.rs`
- **仓储实现**: `src/persistence/schema_repository.rs`
- **UI 组件**: `ui/components/crud-list.slint`
- **应用协调**: `src/app.rs`

---

## 日志级别设置

项目使用 `simple_logger` 进行日志记录，默认从环境变量 `RUST_LOG` 读取日志级别。

### 在 PowerShell 中设置日志级别并运行：

```powershell
# 设置为 debug 级别（显示所有日志）
$env:RUST_LOG = "debug"; cargo run

# 设置为 info 级别（默认）
$env:RUST_LOG = "info"; cargo run

# 设置为 warn 级别
$env:RUST_LOG = "warn"; cargo run
```

### 在 Linux/macOS 中：

```bash
# 设置为 debug 级别
RUST_LOG=debug cargo run
```

这对于调试应用程序行为非常有用，特别是查看 ViewModel 中的数据绑定和状态变化。
