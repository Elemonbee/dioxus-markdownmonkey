# MarkdownMonkey

一个使用 [Dioxus](https://dioxuslabs.com/) 框架构建的现代 Markdown 编辑器。

A modern Markdown editor built with [Dioxus](https://dioxuslabs.com/) framework.

## 特性 / Features

- 📝 **Markdown 编辑** - 实时预览、语法高亮、Mermaid 图表、数学公式 (KaTeX)
- 📁 **文件管理** - 新建、打开、保存 Markdown 文件，文件树浏览
- 🗂️ **多标签页** - 同时编辑多个文件，每个标签独立撤销/重做历史
- 📋 **大纲视图** - 自动提取标题生成目录，快速导航
- 🤖 **AI 助手** - 集成多个 AI 提供商（OpenAI、Claude、DeepSeek、Kimi、Ollama、OpenRouter）
- 🎨 **主题切换** - 深色/浅色/跟随系统
- 🌐 **国际化** - 简体中文 / 美式英语
- ⌨️ **快捷键** - 丰富的键盘快捷键支持
- 📤 **多格式导出** - HTML / PDF / DOCX (Word) / 纯文本
- 💾 **自动保存** - 可配置间隔的自动保存，外部修改检测
- 🔍 **搜索替换** - 支持正则、大小写敏感
- 📊 **表格编辑器** - 可视化表格创建与编辑
- ✅ **拼写检查** - 英文拼写 + 中文检测
- 🔐 **安全存储** - API Key 通过系统密钥环安全存储

## 技术栈 / Tech Stack

| 类别 | 技术 | 版本 |
|------|------|------|
| **UI 框架** | Dioxus (desktop) | 0.7 |
| **语言** | Rust | Edition 2021 |
| **Markdown 解析** | pulldown-cmark | 0.10 |
| **HTML 安全** | ammonia | 4 |
| **代码高亮** | syntect | 5 |
| **HTTP** | reqwest (rustls-tls) | 0.12 |
| **异步运行时** | tokio | 1 |
| **密钥存储** | keyring | 3 |
| **序列化** | serde + serde_json | 1 |
| **文件对话框** | rfd | 0.15 |
| **PDF 导出** | printpdf | 0.7 |
| **DOCX 导出** | zip (OOXML) | 2.2 |
| **文件监控** | notify | 6 |
| **剪贴板** | arboard | 3 |

## 架构 / Architecture

项目以 **组件 + Actions + Services/State** 的分层方式组织代码。当前实现接近 PAL (Presentation-Actions-Logic)，但并不是严格的纯 PAL：部分组件仍会直接读写 `AppState`，Actions 层也以轻量封装为主。

The codebase uses a **components + actions + services/state** layered structure. It is PAL-inspired rather than a strict PAL implementation: some components still read/write `AppState` directly, and the Actions layer is intentionally lightweight.

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
│  └── shortcut_actions.rs — 快捷键操作             │
├─────────────────────────────────────────────────┤
│  Logic (逻辑层)                                  │
│  state/ — 全局状态 (AppState, Dioxus Signal)     │
│  services/ — 纯逻辑服务 (可独立测试)              │
│  utils/ — 工具函数 (i18n 等)                     │
└─────────────────────────────────────────────────┘
```

### 核心原则 / Core Principles

1. **所有 Hooks 在组件顶部无条件调用** / All hooks called unconditionally at component top
2. **始终渲染所有子组件，用 CSS 控制显示** / Always render all sub-components, control visibility with CSS
3. **优先通过 Actions 复用交互逻辑，但允许组件直接操作状态以保持实现简单** / Prefer Actions for reusable interaction logic, while allowing direct state access where it keeps the implementation simpler
4. **状态管理使用 Dioxus Signal 响应式模式** / State management uses Dioxus Signal reactive pattern

## 项目结构 / Project Structure

```
src/
├── main.rs              # 应用入口，启动 Dioxus desktop
├── app.rs               # 主应用组件（布局、初始化）
│
├── state/
│   └── app_state.rs     # 全局状态 (AppState, 40+ Signal)
│
├── components/          # UI 组件 (17 个)
│   ├── editor.rs        # Markdown 编辑器 (textarea + 虚拟行号)
│   ├── preview.rs       # 实时预览面板
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
│   └── icons.rs         # SVG 图标组件
│
├── actions/             # 交互逻辑层（轻量封装）
│   ├── app_actions.rs   # 应用级 Actions (主题、弹窗、侧边栏)
│   ├── editor_actions.rs # 编辑器 Actions (格式化、文本操作)
│   ├── file_actions.rs  # 文件操作 Actions (打开、保存、扫描)
│   └── shortcut_actions.rs # 快捷键分发
│
├── services/            # 服务层 (纯逻辑，可独立测试)
│   ├── markdown.rs      # Markdown 渲染 (pulldown-cmark + 代码高亮)
│   ├── ai.rs            # AI API 调用 (多提供商)
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
│   └── i18n.rs          # 国际化 (中/英)
│
└── styles/              # CSS 样式
    ├── variables.css    # CSS 变量 (主题色)
    ├── base.css         # 基础样式
    ├── editor.css       # 编辑器样式
    ├── toolbar.css      # 工具栏样式
    ├── sidebar.css      # 侧边栏样式
    └── modals.css       # 弹窗样式
```

## 开发 / Development

### 环境要求 / Requirements

- Rust 1.80+
- Cargo

### 构建 / Build

```bash
# 开发模式 / Development
cargo build

# 发布模式 (优化体积) / Release (optimized)
cargo build --release
```

### 运行 / Run

```bash
cargo run
```

### 测试 / Test

```bash
# 运行所有测试 / Run all tests
cargo test

# 运行特定模块测试 / Run specific module tests
cargo test services::
cargo test actions::
```

## 快捷键 / Keyboard Shortcuts

| 快捷键 / Shortcut | 功能 / Function |
|--------|------|
| Ctrl+N | 新建文件 / New File |
| Ctrl+O | 打开文件 / Open File |
| Ctrl+S | 保存文件 / Save File |
| Ctrl+Shift+S | 另存为 / Save As |
| Ctrl+Z | 撤销 / Undo |
| Ctrl+Y / Ctrl+Shift+Z | 重做 / Redo |
| Ctrl+B | 粗体 / Bold |
| Ctrl+I | 斜体 / Italic |
| Ctrl+K | 插入链接 / Insert Link |
| Ctrl+\ | 切换侧边栏 / Toggle Sidebar |
| Ctrl+P | 切换预览 / Toggle Preview |
| Ctrl+, | 打开设置 / Open Settings |
| Ctrl+/ | 显示快捷键 / Show Shortcuts |
| Ctrl+J | AI 助手 / AI Assistant |
| Ctrl+F | 搜索替换 / Search & Replace |
| Ctrl++ / Ctrl+- | 字体缩放 / Font Zoom |
| Escape | 关闭弹窗 / Close Modal |

## 许可证 / License

MIT License
