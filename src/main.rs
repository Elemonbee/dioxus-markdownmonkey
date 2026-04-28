//! MarkdownMonkey - 现代化的 Markdown 编辑器 / A Modern Markdown Editor
//!
//! 使用 Dioxus 框架构建 / Built with Dioxus Framework
//! 纯 Rust 实现，无 Tauri 依赖 / Pure Rust, No Tauri Dependency

// 允许非 snake_case 命名（Dioxus 组件使用 PascalCase）
// Allow non-snake_case (Dioxus components use PascalCase)
#![allow(non_snake_case)]

// 应用模块 / Application Modules
mod actions; // 业务逻辑 Actions / Business Logic Actions
mod app; // 主应用 / Main Application
mod components; // UI 组件 / UI Components
mod config; // 运行时配置 / Runtime Configuration

mod services; // 服务层 / Services Layer
mod state; // 状态管理 / State Management
mod utils; // 工具函数 / Utility Functions

fn main() {
    // 初始化 tracing：默认 info；可通过 RUST_LOG 覆盖（如 markdownmonkey=debug,hyper=info）
    // Initialize tracing: default info; override with RUST_LOG (e.g. markdownmonkey=debug)
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // 启动 Dioxus Desktop 应用 / Launch Dioxus Desktop Application
    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Markdown Monkey")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
                        .with_min_inner_size(dioxus::desktop::LogicalSize::new(600.0, 400.0))
                        // 不置顶窗口 / Don't keep window always on top
                        .with_always_on_top(false),
                )
                // 不禁用右键菜单（启用右键"检查"开发者工具）
                // Don't disable context menu (enable right-click "Inspect" devtools)
                .with_disable_context_menu(false),
        )
        .launch(app::App);
}
