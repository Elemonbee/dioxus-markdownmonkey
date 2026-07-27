# MarkdownMonkey

![MarkdownMonkey 主界面](docs/screenshots/main_zh.png)

**版本 0.5.0** · **[English](./README_EN.md)** | 中文

一个使用 [Dioxus](https://dioxuslabs.com/) 框架构建的现代 Markdown 编辑器。
> 本项目为 Vibe Coding 项目，所有代码由 AI 生成。

## ✨ 特性

- 📝 **Markdown 编辑** - 实时预览、语法高亮、Mermaid 图表、数学公式 (KaTeX)；预览优先使用内置离线脚本，必要时回退 CDN
- 📁 **文件管理** - 工作区文件夹、文件树筛选、最近打开、多编码 (UTF-8/GBK/UTF-16)；拖放 `.md` / `.txt` 打开
- 🗂️ **多标签页** - 同时编辑多个文件，每标签独立撤销/重做；关闭未保存文件时确认
- 📋 **大纲视图** - 自动提取标题生成目录，快速导航
- 💾 **会话恢复** - 启动时恢复标签、活动页、工作区与未保存草稿（可在设置中关闭）
- 🤖 **AI 助手** - OpenAI / Claude / DeepSeek / Kimi / Ollama / OpenRouter；流式生成可停止；**按文档独立会话历史**
- 📤 **多格式导出** - 工具栏下拉：HTML / PDF / DOCX / 纯文本
  - HTML：打包本地图片与离线 Mermaid/KaTeX 到 `{文件名}_files/`
  - PDF / DOCX：嵌入本地 PNG/JPEG（整行 `![alt](path)`）；PDF 自动探测系统中文字体，可在设置中指定字体路径
- 🔍 **搜索替换** - 文档内搜索（大小写 / 正则）；工作区全局搜索与批量替换（优先使用已打开标签缓冲）
- 🖼️ **图片支持** - 粘贴/拖放图片保存到工作区并插入 Markdown
- 🎨 **主题切换** - 深色 / 浅色 / 跟随系统；窗口尺寸持久化；工具栏可快速切换语言
- 🌐 **国际化** - 简体中文 / 美式英语
- ⌨️ **快捷键** - 见下方一览表
- 📊 **表格编辑器** - 可视化创建与编辑
- ✅ **拼写检查** - 英文拼写 + 中文检测
- 🔐 **安全存储** - API Key 存于系统密钥环
- 💾 **自动保存** - 可配置间隔；外部文件修改检测；大文件（默认 1 MB）打开前提示

## 🛠️ 技术栈

版本列为当前 `Cargo.lock` / `Cargo.toml` 解析结果，执行 `cargo update` 后可能微调。

| 类别 | 技术 | 版本 |
|------|------|------|
| **UI 框架** | Dioxus (desktop) | 0.7.9 |
| **语言** | Rust | Edition 2021 |
| **Markdown 解析** | pulldown-cmark | 0.13 |
| **HTML 安全** | ammonia | 4 |
| **代码高亮** | syntect | 5 |
| **HTTP** | reqwest (rustls) | 0.13 |
| **异步运行时** | tokio | 1 |
| **密钥存储** | keyring | 4 |
| **序列化** | serde + serde_json | 1.x |
| **文件对话框** | rfd | 0.17 |
| **用户目录** | dirs | 6 |
| **日志** | tracing + tracing-subscriber | 0.1 / 0.3 |
| **PDF 导出** | printpdf (png/jpeg) | 0.9 |
| **DOCX 导出** | zip (OOXML) | 8 |
| **文件监控** | notify | 8 |
| **剪贴板** | arboard | 3 |

## 🏗️ 架构

项目以 **组件 + Actions + Services/State** 分层，采用 PAL (Presentation-Actions-Logic) 启发式架构：部分组件仍会直接读写 `AppState`，Actions 层以轻量封装为主。

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
│  utils/ — 工具函数 (i18n、工作区搜索等)           │
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
├── main.rs                 # 应用入口
├── app.rs                  # 主布局、初始化、自动保存、会话恢复、文件监控
├── config.rs               # 应用配置常量
│
├── state/
│   ├── types.rs            # Theme、TabInfo、History 等类型
│   ├── domains.rs          # 按领域拆分的状态视图
│   ├── app_state.rs        # AppState 结构与初始化
│   ├── app_state_ops.rs    # 文档 / 标签 / 大纲业务逻辑
│   └── app_state_tests.rs  # 状态单元测试
│
├── components/             # UI 组件
│   ├── editor.rs / preview.rs / sidebar.rs / toolbar.rs
│   ├── tabbar.rs / statusbar.rs / file_tree.rs / icons.rs
│   └── *_modal.rs          # 设置、搜索、AI、表格、确认等弹窗
│
├── actions/                # 交互逻辑
│   ├── app_actions.rs / editor_actions.rs
│   ├── file_actions.rs / shortcut_actions.rs
│   └── tests.rs
│
├── services/
│   ├── markdown.rs / ai.rs / auto_save.rs / image.rs
│   ├── settings.rs / session.rs / recent_files.rs
│   ├── file_watcher.rs / spellcheck.rs / syntax_highlight.rs
│   ├── keyring_service.rs / theme_detector.rs
│   └── export/             # HTML / PDF / DOCX / TXT
│       ├── mod.rs / shared.rs
│       ├── html.rs / pdf.rs / docx.rs / text.rs
│
├── utils/
│   ├── i18n.rs / file_utils.rs
│   ├── workspace_search.rs # 工作区搜索（含打开标签缓冲）
│   └── replace.rs          # 替换工具
│
└── styles/                 # CSS（variables / base / editor / toolbar / sidebar / modals）

assets/
├── editor_enhance.js
├── dictionaries/
└── vendor/                 # 离线 Mermaid + KaTeX
```

配置与会话数据默认位于用户配置目录下的 `MarkdownMonkey/`（如 `settings.json`、`session.json`、`session_drafts/`、`ai_history/`）。

## 🚀 开发

### 环境要求

- Rust 1.80+
- Cargo

### 构建与运行

```bash
cargo build
cargo build --release
cargo run
```

调试日志示例（Windows PowerShell）：

```powershell
$env:RUST_LOG="markdownmonkey=debug,info"; cargo run
```

可选：通过环境变量 `MARKDOWNMONKEY_PDF_FONT` 指定 PDF 中文字体路径。

### 测试与检查

```bash
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

## 📦 发布

跨平台打包与打标签流程见 **[docs/RELEASE.md](docs/RELEASE.md)**（GitHub Actions：Windows / Linux / macOS）。

## ⌨️ 快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+N | 新建文件 |
| Ctrl+O | 打开文件 |
| Ctrl+S | 保存 |
| Ctrl+Z | 撤销 |
| Ctrl+Y / Ctrl+Shift+Z | 重做 |
| Ctrl+B | 粗体 |
| Ctrl+I | 斜体 |
| Ctrl+` | 行内代码 |
| Ctrl+K | 插入链接 |
| Ctrl+F | 文档内搜索替换 |
| Ctrl+Shift+F | 工作区全局搜索 / 替换 |
| Ctrl+\\ | 切换侧边栏 |
| Ctrl+P | 切换预览 |
| Ctrl+T | 切换主题 |
| Ctrl+, | 打开设置 |
| Ctrl+/ | 显示快捷键 |
| Ctrl+J | AI 助手 |
| Escape | 关闭弹窗 |

## 📄 许可证

MIT License
