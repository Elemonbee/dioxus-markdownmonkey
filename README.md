# MarkdownMonkey
<img width="3840" height="2024" alt="image" src="https://github.com/user-attachments/assets/08b034d0-bc88-4551-8116-e8bf62e91ea2" />



**[English](./README_EN.md)** | 中文

一个使用 [Dioxus](https://dioxuslabs.com/) 框架构建的现代 Markdown 编辑器。
> 本项目为 Vibe Coding 项目，所有代码由 AI 生成。

## ✨ 特性

- 📝 **Markdown 编辑** - 实时预览、语法高亮、Mermaid 图表、数学公式 (KaTeX)
- 📁 **文件管理** - 新建、打开、保存 Markdown 文件，文件树浏览，多编码支持 (UTF-8/GBK/UTF-16)
- 🗂️ **多标签页** - 同时编辑多个文件，每个标签独立撤销/重做历史
- 📋 **大纲视图** - 自动提取标题生成目录，快速导航
- 🤖 **AI 助手** - 集成多个 AI 提供商（OpenAI、Claude、DeepSeek、Kimi、Ollama、OpenRouter）
- 🎨 **主题切换** - 深色/浅色/跟随系统
- 🌐 **国际化** - 简体中文 / 美式英语
- ⌨️ **快捷键** - 丰富的键盘快捷键支持
- 📤 **多格式导出** - HTML / PDF / DOCX (Word) / 纯文本
- 💾 **自动保存** - 可配置间隔的自动保存，外部修改检测
- 🔍 **搜索替换** - 支持大小写敏感、全局搜索
- 📊 **表格编辑器** - 可视化表格创建与编辑
- ✅ **拼写检查** - 英文拼写 + 中文检测
- 🔐 **安全存储** - API Key 通过系统密钥环安全存储
- 🖼️ **图片支持** - 粘贴/拖放图片，自动插入 Markdown 语法

## 🛠️ 技术栈

版本列为当前 `Cargo.lock` 解析结果，执行 `cargo update` 后可能微调。

| 类别 | 技术 | 版本 |
|------|------|------|
| **UI 框架** | Dioxus (desktop) | 0.7.6 |
| **语言** | Rust | Edition 2021 |
| **Markdown 解析** | pulldown-cmark | 0.12.2 |
| **HTML 安全** | ammonia | 4.1.2 |
| **代码高亮** | syntect | 5.3.0 |
| **HTTP** | reqwest (rustls-tls) | 0.12.28 |
| **异步运行时** | tokio | 1.50.0 |
| **密钥存储** | keyring | 3.6.3 |
| **序列化** | serde + serde_json | 1.x |
| **文件对话框** | rfd | 0.15.4 |
| **用户目录** | dirs | 6.0.0 |
| **日志** | tracing + tracing-subscriber (env-filter) | 0.1 / 0.3.23 |
| **PDF 导出** | printpdf | 0.7.0 |
| **DOCX 导出** | zip (OOXML) | 4.6.1 |
| **文件监控** | notify | 7.0.0 |
| **剪贴板** | arboard | 3.6.1 |

## 🏗️ 架构

项目以 **组件 + Actions + Services/State** 的分层方式组织代码，采用 PAL (Presentation-Actions-Logic) 启发式架构：部分组件仍会直接读写 `AppState`，Actions 层以轻量封装为主。

```
┌─────────────────────────────────────────────────┐
│  Presentation (展示层)                           │
│  components/ — UI 组件，只负责渲染                 │
│  ├── editor.rs, preview.rs, sidebar.rs          │
│  ├── toolbar.rs, tabbar.rs, statusbar.rs        │
│  └── *_modal.rs (各种弹窗)                       │
├─────────────────────────────────────────────────┤
│  Actions (动作层)                                │
│  actions/ — 业务逻辑处理器                        │
│  ├── app_actions.rs — 应用级操作                  │
│  ├── editor_actions.rs — 编辑器操作               │
│  ├── file_actions.rs — 文件操作                   │
│  └── shortcut_actions.rs — 快捷键分发             │
├─────────────────────────────────────────────────┤
│  Logic (逻辑层)                                  │
│  state/ — 全局状态 (AppState, Dioxus Signal)     │
│  services/ — 纯逻辑服务 (可独立测试)              │
│  utils/ — 工具函数 (i18n 等)                     │
└─────────────────────────────────────────────────┘
```

### 核心原则

1. 所有 Hooks 在组件顶部无条件调用
2. 始终渲染所有子组件，用 CSS 控制显示
3. 优先通过 Actions 复用交互逻辑，但允许组件直接操作状态以保持实现简单
4. 状态管理使用 Dioxus Signal 响应式模式

## 📁 项目结构

```
src/
├── main.rs              # 应用入口，启动 Dioxus desktop
├── app.rs               # 主应用组件（布局、初始化、自动保存、文件监控）
│
├── state/
│   └── app_state.rs     # 全局状态 (AppState, 40+ Signal)
│
├── components/          # UI 组件 (17 个)
│   ├── editor.rs        # Markdown 编辑器 (textarea + 虚拟行号)
│   ├── preview.rs       # 实时预览面板 (防抖渲染)
│   ├── sidebar.rs       # 侧边栏 (大纲 + 文件树)
│   ├── toolbar.rs       # 格式化工具栏
│   ├── tabbar.rs        # 多标签页栏
│   ├── statusbar.rs     # 状态栏 (字数/行数/保存状态)
│   ├── file_tree.rs     # 文件树浏览
│   ├── table_editor_modal.rs  # 表格编辑器弹窗
│   ├── ai_chat_modal.rs       # AI 聊天弹窗
│   ├── ai_result_modal.rs     # AI 结果弹窗
│   ├── settings_modal.rs      # 设置弹窗
│   ├── shortcuts_modal.rs     # 快捷键弹窗
│   ├── search_modal.rs        # 搜索替换弹窗
│   ├── global_search_modal.rs # 全局搜索弹窗
│   ├── file_modified_modal.rs # 文件修改提示弹窗
│   ├── large_file_warning_modal.rs # 大文件警告弹窗
│   ├── close_confirm_modal.rs # 关闭未保存确认弹窗
│   └── icons.rs         # SVG 图标组件
│
├── actions/             # 交互逻辑层
│   ├── app_actions.rs   # 应用级 Actions (主题、弹窗、侧边栏)
│   ├── editor_actions.rs # 编辑器 Actions (格式化、文本操作)
│   ├── file_actions.rs  # 文件操作 Actions (打开、保存、编码检测)
│   └── shortcut_actions.rs # 快捷键分发
│
├── services/            # 服务层 (纯逻辑，可独立测试)
│   ├── markdown.rs      # Markdown 渲染 (pulldown-cmark + 代码高亮)
│   ├── ai.rs            # AI API 调用 (多提供商，流式/非流式)
│   ├── export.rs        # 导出服务 (HTML/PDF/DOCX/TXT)
│   ├── auto_save.rs     # 自动保存服务
│   ├── image.rs         # 图片处理 (Base64)
│   ├── settings.rs      # 设置持久化
│   ├── recent_files.rs  # 最近文件记录
│   ├── file_watcher.rs  # 文件外部修改检测
│   ├── spellcheck.rs    # 拼写检查
│   ├── syntax_highlight.rs # 语法高亮
│   └── keyring_service.rs  # 密钥管理
│
├── utils/
│   ├── i18n.rs          # 国际化 (中/英)
│   └── file_utils.rs    # 文件扫描工具 (带深度/数量限制)
│
└── styles/              # CSS 样式
    ├── variables.css    # CSS 变量 (主题色)
    ├── base.css         # 基础样式
    ├── editor.css       # 编辑器样式
    ├── toolbar.css      # 工具栏样式
    ├── sidebar.css      # 侧边栏样式
    └── modals.css       # 弹窗样式
```

## 🚀 开发

### 环境要求

- Rust 1.80+
- Cargo

### 构建

```bash
# 开发模式
cargo build

# 发布模式 (优化体积)
cargo build --release
```

### 运行

```bash
cargo run
```

调试时可设置日志级别，例如：

```bash
# Windows PowerShell
$env:RUST_LOG="markdownmonkey=debug,info"; cargo run
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test services::
cargo test actions::

# 格式检查
cargo fmt --all -- --check

# Clippy 检查
cargo clippy
```

## ⌨️ 快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+N | 新建文件 |
| Ctrl+Z | 撤销 |
| Ctrl+Y / Ctrl+Shift+Z | 重做 |
| Ctrl+B | 粗体 |
| Ctrl+I | 斜体 |
| Ctrl+` | 行内代码 |
| Ctrl+K | 插入链接 |
| Ctrl+F | 搜索替换 |
| Ctrl+Shift+F | 全局搜索 |
| Ctrl+\\ | 切换侧边栏 |
| Ctrl+P | 切换预览 |
| Ctrl+T | 切换主题 |
| Ctrl+, | 打开设置 |
| Ctrl+/ | 显示快捷键 |
| Ctrl+J | AI 助手 |
| Escape | 关闭弹窗 |

## 📄 许可证

MIT License
