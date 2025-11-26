# FondsPod 国际化指南 (Internationalization Guide)

本项目使用 Slint 的 `@tr()` 宏实现国际化。所有可翻译的字符串都集中在 `ui/translations.slint` 文件中。

## 目录结构

```
ui/
├── translations.slint      # 翻译全局变量定义（使用 @tr() 宏）
├── i18n/
│   ├── zh-CN.ftl          # 中文翻译文件
│   └── en-US.ftl          # 英文翻译文件
└── ...
```

## 使用 slint-tr-extractor

`slint-tr-extractor` 是 Slint 提供的工具，用于从 `.slint` 文件中提取所有 `@tr()` 宏的翻译字符串。

### 安装

```bash
cargo install slint-tr-extractor
```

### 提取翻译字符串

```bash
# 从主 UI 文件提取翻译
slint-tr-extractor ./ui/app-window.slint

# 将提取结果保存到文件
slint-tr-extractor ./ui/app-window.slint > ./ui/i18n/messages.pot
```

### 翻译文件格式 (.ftl)

翻译文件使用 Fluent 格式 (`.ftl`)：

```ftl
# 注释
nav-fonds = 📁 全宗
btn-add = 添加
label-name = 名称:
```

## 如何添加新的翻译字符串

1. 在 `ui/translations.slint` 中添加新的属性：
   ```slint
   out property <string> my_new_label: @tr("my-new-label" => "默认中文文本");
   ```

2. 在 `ui/i18n/zh-CN.ftl` 中添加中文翻译：
   ```ftl
   my-new-label = 中文翻译
   ```

3. 在 `ui/i18n/en-US.ftl` 中添加英文翻译：
   ```ftl
   my-new-label = English Translation
   ```

4. 在 UI 组件中使用：
   ```slint
   import { Translations } from "translations.slint";
   
   Text {
       text: Translations.my_new_label;
   }
   ```

## 运行时切换语言

在 Rust 代码中，可以通过调用 Slint 的 `select_translation_bundle` 或 `invoke_translations` 函数来切换语言。

### 示例代码

```rust
use slint::*;

fn main() {
    // 初始化翻译 - 加载中文翻译
    slint::init_translations(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/i18n/zh-CN.ftl"));
    
    // 或者在运行时切换语言
    // slint::init_translations(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/i18n/en-US.ftl"));
    
    let app = AppWindow::new().unwrap();
    app.run().unwrap();
}
```

## 翻译键命名规范

- 使用小写字母和连字符 `-`
- 前缀表示类型：
  - `nav-` 导航菜单
  - `btn-` 按钮文本
  - `label-` 标签文本
  - `dialog-` 对话框标题
  - `menu-` 右键菜单项
  - `msg-` 消息/提示文本

## 当前支持的语言

| 语言代码 | 语言名称 |
|---------|---------|
| zh-CN   | 简体中文 |
| en-US   | 英语(美国) |

## 添加新语言

1. 在 `ui/i18n/` 目录创建新的 `.ftl` 文件（如 `ja-JP.ftl`）
2. 复制 `zh-CN.ftl` 内容并翻译所有条目
3. 在应用初始化时加载对应的翻译文件
