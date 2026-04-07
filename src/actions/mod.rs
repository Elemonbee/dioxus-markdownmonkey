//! Actions 模块 - 业务逻辑处理器
//! Actions Module - Business Logic Handlers
//!
//! 遵循 PAL 架构原则，将业务逻辑与 UI 渲染分离
//! Following PAL architecture, separate business logic from UI rendering

mod app_actions;
mod editor_actions;
mod file_actions;
pub mod shortcut_actions;

#[cfg(test)]
mod tests;

pub use app_actions::*;
pub use editor_actions::*;
pub use file_actions::*;
// shortcut_actions 导出在需要时取消注释 / Uncomment when needed
// pub use shortcut_actions::*;
