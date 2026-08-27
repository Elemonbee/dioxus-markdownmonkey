//! Actions 模块 - 业务逻辑处理器
//! Actions Module - Business Logic Handlers
//!
//! PAL：组件只读 AppState，所有 AppState 写入经由此层。
//! PAL: components read AppState; all AppState writes go through this layer.

mod app_actions;
mod editor_actions;
mod file_actions;
mod search_actions;
mod settings_actions;
pub mod shortcut_actions;

#[cfg(test)]
mod tests;

pub use app_actions::*;
pub use editor_actions::*;
pub use file_actions::*;
pub use search_actions::*;
pub use settings_actions::*;
// shortcut_actions 以子模块形式公开：`use crate::actions::shortcut_actions::ShortcutActions`
// Public as a submodule: `use crate::actions::shortcut_actions::ShortcutActions`
