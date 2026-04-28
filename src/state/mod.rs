//! 状态管理模块 / State Management Module
//!
//! 使用 Dioxus Signal 管理全局状态 / Use Dioxus Signal for Global State Management

mod app_state;
mod app_state_ops;
mod types;

#[cfg(test)]
mod app_state_tests;

// 重新导出 / Re-exports
pub use app_state::AppState;
pub use types::*;
