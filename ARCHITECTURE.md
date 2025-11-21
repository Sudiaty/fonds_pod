# FondsPod 架构文档 (Architecture Documentation)

## 概述 (Overview)

FondsPod 采用**清洁架构 (Clean Architecture)** 模式重构，实现了合理的分层设计和低耦合。

## 架构层次 (Architecture Layers)

```
┌─────────────────────────────────────────────────────┐
│          Presentation Layer (表示层)                 │
│     UI Callbacks & Event Handlers                   │
│     (presentation/handlers.rs)                      │
└────────────────┬────────────────────────────────────┘
                 │ depends on
┌────────────────▼────────────────────────────────────┐
│         Application Layer (应用层)                   │
│     Business Logic Orchestration                    │
│     (services/schema_service.rs,                    │
│      services/archive_service.rs)                   │
└────────────────┬────────────────────────────────────┘
                 │ depends on
┌────────────────▼────────────────────────────────────┐
│            Domain Layer (领域层)                     │
│     Business Rules & Abstractions                   │
│     (domain/models.rs,                              │
│      domain/repositories.rs)                        │
└────────────────▲────────────────────────────────────┘
                 │ implemented by
┌────────────────┴────────────────────────────────────┐
│        Infrastructure Layer (基础设施层)             │
│     Database & External Dependencies                │
│     (infrastructure/database.rs,                    │
│      infrastructure/config.rs)                      │
└─────────────────────────────────────────────────────┘
```

## 各层职责 (Layer Responsibilities)

### 1. Presentation Layer (表示层)
**Location:** `src/presentation/`

**职责:**
- 处理 UI 事件和回调
- 将用户操作转换为服务调用
- 更新 UI 状态

**主要组件:**
- `SchemaHandler`: 处理 schema 相关的 UI 事件
- `ArchiveHandler`: 处理档案库相关的 UI 事件
- `SettingsHandler`: 处理设置相关的 UI 事件

**示例:**
```rust
pub struct SchemaHandler<CR: ConfigRepository> {
    archive_service: Rc<ArchiveService<CR>>,
}

impl<CR: ConfigRepository> SchemaHandler<CR> {
    pub fn setup_callbacks(&self, ui: &AppWindow) {
        self.setup_add_schema(ui);
        self.setup_delete_schema(ui);
        // ...
    }
}
```

### 2. Application Layer (应用层)
**Location:** `src/services/`

**职责:**
- 协调业务流程
- 执行业务规则验证
- 组合多个领域操作

**主要组件:**
- `SchemaService`: Schema 业务逻辑服务
  - 创建、删除、列出 schema
  - 管理 schema items
  - 执行 Year schema 保护规则
  
- `ArchiveService`: 档案库管理服务
  - 添加、删除档案库
  - 管理配置
  - 数据库路径管理

**示例:**
```rust
pub struct SchemaService<SR: SchemaRepository, SIR: SchemaItemRepository> {
    schema_repo: SR,
    item_repo: SIR,
}

impl SchemaService {
    pub fn delete_schema(&mut self, schema_no: String) -> Result<bool, Box<dyn Error>> {
        if !can_modify_schema(&schema_no) {
            return Err(format!("Schema '{}' cannot be deleted", schema_no).into());
        }
        self.schema_repo.delete(&schema_no)
    }
}
```

### 3. Domain Layer (领域层)
**Location:** `src/domain/`

**职责:**
- 定义核心业务实体
- 定义数据访问接口 (Repository Traits)
- 实现业务规则函数

**主要组件:**
- `models.rs`: 领域模型
  - `Schema`: Schema 实体
  - `SchemaItem`: Schema 项实体
  - `ArchiveLibrary`: 档案库实体
  - `AppSettings`: 应用配置实体
  - 业务规则: `can_modify_schema()`, `can_modify_schema_items()`

- `repositories.rs`: Repository 接口
  - `SchemaRepository`: Schema 数据访问接口
  - `SchemaItemRepository`: SchemaItem 数据访问接口
  - `ConfigRepository`: 配置数据访问接口

**业务规则示例:**
```rust
pub const PROTECTED_SCHEMA: &str = "Year";

pub fn can_modify_schema(schema_no: &str) -> bool {
    schema_no != PROTECTED_SCHEMA
}

pub fn can_modify_schema_items(schema_no: &str) -> bool {
    schema_no != PROTECTED_SCHEMA
}
```

### 4. Infrastructure Layer (基础设施层)
**Location:** `src/infrastructure/`

**职责:**
- 实现 Repository 接口
- 处理数据库连接
- 处理文件系统操作
- 管理外部依赖

**主要组件:**
- `persistence/`: 数据持久化层
  - `models.rs`: 数据库实体模型
  - `queries.rs`: Diesel ORM 查询实现
  - `schema.rs`: 数据库 schema 定义和初始化
  - `mod.rs`: 提供 `establish_connection` 函数
  
- `database.rs`: Repository 实现
  - `DatabaseSchemaRepository`: Schema 数据库访问实现
  - `DatabaseSchemaItemRepository`: SchemaItem 数据库访问实现
  
- `config.rs`: 配置存储实现
  - `FileConfigRepository`: 文件配置存储实现

**示例:**
```rust
pub struct DatabaseSchemaRepository {
    db_path: PathBuf,
}

impl SchemaRepository for DatabaseSchemaRepository {
    fn create(&mut self, schema: &Schema) -> Result<(), Box<dyn Error>> {
        let mut conn = self.get_connection()?;
        let db_schema = Self::to_db_schema(schema);
        queries::create_schema(&mut conn, &db_schema)
    }
    
    fn list(&mut self) -> Result<Vec<Schema>, Box<dyn Error>> {
        let mut conn = self.get_connection()?;
        queries::list_schemas(&mut conn)
            .map(|schemas| schemas.into_iter().map(Self::from_db_schema).collect())
    }
}
```

## 依赖关系 (Dependencies)

```
main.rs
  │
  ├─> presentation::handlers (表示层)
  │     │
  │     └─> services (应用层)
  │           │
  │           └─> domain (领域层)
  │                 ▲
  │                 │ implemented by
  │                 │
  └───> infrastructure (基础设施层)
```

## 主要优势 (Key Benefits)

### 1. 低耦合 (Low Coupling)
- 各层通过接口 (trait) 交互
- 业务逻辑不依赖具体的数据库实现
- UI 层不直接访问数据库

### 2. 高内聚 (High Cohesion)
- 每层职责单一明确
- 相关功能集中在同一模块

### 3. 可测试性 (Testability)
- 可以为 Repository trait 创建 mock 实现
- 业务逻辑独立于外部依赖
- 可以单独测试每一层

### 4. 可维护性 (Maintainability)
- 清晰的代码组织结构
- 易于定位和修改功能
- 减少修改影响范围

### 5. 可扩展性 (Extensibility)
- 易于添加新的数据源 (实现 Repository trait)
- 易于添加新的业务功能 (添加新的 Service)
- 易于添加新的 UI 功能 (添加新的 Handler)

## 数据流示例 (Data Flow Example)

### 删除 Schema 的流程:

```
1. User clicks delete button
   │
   ▼
2. SchemaHandler.on_delete_schema() (Presentation)
   │ - Get selected schema
   │ - Get database path from ArchiveService
   │
   ▼
3. SchemaService.delete_schema() (Application)
   │ - Validate: can_modify_schema() (Domain Rule)
   │ - Call repository
   │
   ▼
4. DatabaseSchemaRepository.delete() (Infrastructure)
   │ - Establish database connection
   │ - Execute SQL via Diesel
   │
   ▼
5. Database updated
   │
   ▼
6. SchemaHandler.reload_schemas() (Presentation)
   │ - Refresh UI with updated list
   │
   ▼
7. UI updated
```

## Year Schema 保护机制 (Year Schema Protection)

保护机制实现在多个层次:

1. **Domain Layer** (领域层):
   ```rust
   pub const PROTECTED_SCHEMA: &str = "Year";
   
   pub fn can_modify_schema(schema_no: &str) -> bool {
       schema_no != PROTECTED_SCHEMA
   }
   ```

2. **Application Layer** (应用层):
   ```rust
   pub fn delete_schema(&mut self, schema_no: String) -> Result<bool> {
       if !can_modify_schema(&schema_no) {
           return Err("Schema cannot be deleted".into());
       }
       // ...
   }
   ```

3. **Infrastructure Layer** (基础设施层):
   ```rust
   // db/queries.rs
   pub fn delete_schema(conn: &mut SqliteConnection, schema_no: &str) -> Result<bool> {
       if schema_no == "Year" {
           return Ok(false);
       }
       // ...
   }
   ```

## 遗留代码 (Legacy Code)

以下模块仍在使用旧架构，待后续迁移:

- `archive_library.rs`
- `fonds_manager.rs`
- `file_manager.rs`
- `series_manager.rs`
- `fond_classification_manager.rs`
- `number_generator.rs`
- `non_functional.rs`
- `config.rs` (将被 `infrastructure/config.rs` 替代)
- `schema_manager.rs` (将被 `services/schema_service.rs` 替代)

## 最佳实践 (Best Practices)

### 1. 依赖注入 (Dependency Injection)
```rust
// main.rs
let config_repo = FileConfigRepository::new();
let archive_service = Rc::new(ArchiveService::new(config_repo));
let schema_handler = SchemaHandler::new(archive_service.clone());
```

### 2. 接口隔离 (Interface Segregation)
每个 Repository trait 只包含相关的方法，不混合不同实体的操作。

### 3. 单一职责 (Single Responsibility)
每个 Service 只处理一个领域的业务逻辑。

### 4. 依赖倒置 (Dependency Inversion)
高层模块依赖抽象 (trait)，不依赖具体实现。

## 未来改进 (Future Improvements)

1. **Error Handling**: 实现统一的错误处理策略
2. **Logging**: 添加结构化日志
3. **Testing**: 为每层添加单元测试
4. **Migration**: 将遗留代码迁移到新架构
5. **Documentation**: 为每个模块添加详细文档
6. **Validation**: 在 Domain Layer 添加数据验证
7. **Event System**: 实现事件驱动架构用于模块间通信

## 编译和运行 (Build & Run)

```bash
# 编译
cargo build

# 运行
cargo run

# 运行测试 (待实现)
cargo test
```

## 总结 (Summary)

通过采用清洁架构，FondsPod 实现了:
- ✅ 清晰的分层结构
- ✅ 低耦合高内聚
- ✅ 易于测试和维护
- ✅ 良好的可扩展性
- ✅ 业务规则的集中管理

这为项目的长期发展奠定了坚实的基础。
