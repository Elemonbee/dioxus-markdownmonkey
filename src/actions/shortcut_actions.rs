//! 快捷键动作 / Shortcut Actions
//!
//! 遵循 PAL 架构：处理快捷键业务逻辑
//!
//! 注意：部分功能为预留功能，暂未使用
//! Note: Some functions are reserved for future use, not yet used

use crate::actions::{AppActions, FileActions};
use crate::state::AppState;
use dioxus::prelude::*;

/// 快捷键定义 / Shortcut Definition
pub struct Shortcut {
    pub key: &'static str,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub action: ShortcutAction,
}

/// 快捷键动作 / Shortcut Actions
#[derive(Clone, Copy, Debug)]
pub enum ShortcutAction {
    NewFile,
    SaveFile,
    Undo,
    Redo,
    Bold,
    Italic,
    Code,
    Link,
    ToggleSidebar,
    TogglePreview,
    OpenSettings,
    ShowShortcuts,
    ToggleTheme,
    OpenAI,
    Search, // 搜索替换 / Search & Replace (Ctrl+F)
    GlobalSearch,
    Close,
}

/// Shortcut Actions - 快捷键业务逻辑
pub struct ShortcutActions;

impl ShortcutActions {
    /// 获取所有快捷键 / Get all shortcuts
    pub fn get_all() -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "n",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::NewFile,
            },
            Shortcut {
                key: "s",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::SaveFile,
            },
            Shortcut {
                key: "z",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::Undo,
            },
            Shortcut {
                key: "y",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::Redo,
            },
            Shortcut {
                key: "z",
                ctrl: true,
                shift: true,
                alt: false,
                action: ShortcutAction::Redo,
            },
            Shortcut {
                key: "b",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::Bold,
            },
            Shortcut {
                key: "i",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::Italic,
            },
            Shortcut {
                key: "`",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::Code,
            },
            Shortcut {
                key: "k",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::Link,
            },
            Shortcut {
                key: "\\",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::ToggleSidebar,
            },
            Shortcut {
                key: "p",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::TogglePreview,
            },
            Shortcut {
                key: ",",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::OpenSettings,
            },
            Shortcut {
                key: "/",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::ShowShortcuts,
            },
            Shortcut {
                key: "t",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::ToggleTheme,
            },
            Shortcut {
                key: "j",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::OpenAI,
            },
            Shortcut {
                key: "f",
                ctrl: true,
                shift: false,
                alt: false,
                action: ShortcutAction::Search,
            },
            Shortcut {
                key: "f",
                ctrl: true,
                shift: true,
                alt: false,
                action: ShortcutAction::GlobalSearch,
            },
            Shortcut {
                key: "Escape",
                ctrl: false,
                shift: false,
                alt: false,
                action: ShortcutAction::Close,
            },
        ]
    }

    /// 处理快捷键事件 / Handle shortcut event
    pub fn handle(state: &mut AppState, key: &str, ctrl: bool, shift: bool, alt: bool) -> bool {
        for shortcut in Self::get_all() {
            if key == shortcut.key
                && ctrl == shortcut.ctrl
                && shift == shortcut.shift
                && alt == shortcut.alt
            {
                Self::execute(state, shortcut.action);
                return true;
            }
        }
        false
    }

    /// 处理键盘事件 / Handle keyboard event
    pub fn handle_event(state: &mut AppState, event: &KeyboardEvent) -> bool {
        Self::handle(
            state,
            &event.key().to_string(),
            event.modifiers().ctrl(),
            event.modifiers().shift(),
            event.modifiers().alt(),
        )
    }

    /// 执行快捷键动作 / Execute shortcut action
    pub fn execute(state: &mut AppState, action: ShortcutAction) {
        match action {
            ShortcutAction::NewFile => {
                state.new_tab();
            }
            ShortcutAction::SaveFile => {
                if let Err(e) = FileActions::save_current_file(state) {
                    tracing::warn!("Save shortcut: {}", e);
                }
            }
            ShortcutAction::Undo => {
                state.undo();
            }
            ShortcutAction::Redo => {
                state.redo();
            }
            ShortcutAction::Bold => {
                state.insert_format_around_selection("**", "**");
            }
            ShortcutAction::Italic => {
                state.insert_format_around_selection("*", "*");
            }
            ShortcutAction::Code => {
                state.insert_format_around_selection("`", "`");
            }
            ShortcutAction::Link => {
                state.insert_format_around_selection("[", "](url)");
            }
            ShortcutAction::ToggleSidebar => {
                AppActions::toggle_sidebar(state);
            }
            ShortcutAction::TogglePreview => {
                AppActions::toggle_preview(state);
            }
            ShortcutAction::OpenSettings => {
                AppActions::show_settings(state);
            }
            ShortcutAction::ShowShortcuts => {
                AppActions::show_shortcuts(state);
            }
            ShortcutAction::ToggleTheme => {
                AppActions::toggle_theme(state);
            }
            ShortcutAction::OpenAI => {
                AppActions::show_ai_chat(state);
            }
            ShortcutAction::Search => {
                // Ctrl+F: 打开搜索替换弹窗 / Open search & replace modal
                *state.show_search.write() = true;
            }
            ShortcutAction::GlobalSearch => {
                *state.show_global_search.write() = true;
            }
            ShortcutAction::Close => {
                AppActions::close_overlays(state);
            }
        }
    }
}
