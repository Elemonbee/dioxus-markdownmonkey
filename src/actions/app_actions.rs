//! 应用级别 Actions / App-level Actions
//!
//! 处理主题切换、语言切换、侧边栏等全局操作

use crate::config::{SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::state::{AIProvider, AppState, Language, SidebarTab, Theme};
use dioxus::prelude::{ReadableExt, WritableExt};

/// 应用 Actions 处理器 / App Actions Handler
pub struct AppActions;

impl AppActions {
    /// 切换主题 / Toggle Theme
    pub fn toggle_theme(state: &mut AppState) {
        let new_theme = match *state.theme.read() {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::System,
            Theme::System => Theme::Dark,
        };
        *state.theme.write() = new_theme;
    }

    /// 设置主题 / Set Theme
    pub fn set_theme(state: &mut AppState, theme: Theme) {
        *state.theme.write() = theme;
    }

    /// 切换语言 / Toggle Language
    pub fn toggle_language(state: &mut AppState) {
        let new_lang = match *state.language.read() {
            Language::ZhCN => Language::EnUS,
            Language::EnUS => Language::ZhCN,
        };
        *state.language.write() = new_lang;
    }

    /// 设置语言 / Set Language
    pub fn set_language(state: &mut AppState, language: Language) {
        *state.language.write() = language;
    }

    /// 切换侧边栏 / Toggle Sidebar
    pub fn toggle_sidebar(state: &mut AppState) {
        let current = *state.sidebar_visible.read();
        *state.sidebar_visible.write() = !current;
    }

    /// 设置侧边栏可见性 / Set Sidebar Visibility
    pub fn set_sidebar_visible(state: &mut AppState, visible: bool) {
        *state.sidebar_visible.write() = visible;
    }

    /// 切换预览 / Toggle Preview
    pub fn toggle_preview(state: &mut AppState) {
        let current = *state.show_preview.read();
        *state.show_preview.write() = !current;
    }

    /// 切换侧边栏标签 / Toggle Sidebar Tab
    pub fn set_sidebar_tab(state: &mut AppState, tab: SidebarTab) {
        *state.sidebar_tab.write() = tab;
    }

    /// 设置侧边栏宽度 / Set Sidebar Width
    pub fn set_sidebar_width(state: &mut AppState, width: u32) {
        *state.sidebar_width.write() = width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
    }

    /// 显示设置弹窗 / Show Settings Modal
    pub fn show_settings(state: &mut AppState) {
        *state.show_settings.write() = true;
    }

    /// 隐藏设置弹窗 / Hide Settings Modal
    pub fn hide_settings(state: &mut AppState) {
        *state.show_settings.write() = false;
    }

    /// 显示快捷键弹窗 / Show Shortcuts Modal
    pub fn show_shortcuts(state: &mut AppState) {
        *state.show_shortcuts.write() = true;
    }

    /// 隐藏快捷键弹窗 / Hide Shortcuts Modal
    pub fn hide_shortcuts(state: &mut AppState) {
        *state.show_shortcuts.write() = false;
    }

    /// 显示 AI 聊天弹窗 / Show AI Chat Modal
    pub fn show_ai_chat(state: &mut AppState) {
        *state.show_ai_chat.write() = true;
    }

    /// 隐藏 AI 聊天弹窗 / Hide AI Chat Modal
    pub fn hide_ai_chat(state: &mut AppState) {
        *state.show_ai_chat.write() = false;
    }

    /// 显示 AI 结果弹窗 / Show AI Result Modal
    pub fn show_ai_result(state: &mut AppState) {
        *state.show_ai_result.write() = true;
    }

    /// 隐藏 AI 结果弹窗 / Hide AI Result Modal
    pub fn hide_ai_result(state: &mut AppState) {
        *state.show_ai_result.write() = false;
    }

    /// 关闭所有弹窗 / Close All Modals
    pub fn close_all_modals(state: &mut AppState) {
        *state.show_settings.write() = false;
        *state.show_shortcuts.write() = false;
        *state.show_ai_chat.write() = false;
        *state.show_ai_result.write() = false;
    }

    /// Close all modal-like overlays including search panels.
    pub fn close_overlays(state: &mut AppState) {
        Self::close_all_modals(state);
        *state.show_search.write() = false;
        *state.show_global_search.write() = false;
        *state.show_table_editor.write() = false;
    }

    /// Update AI provider and sync default endpoint/model values.
    pub fn set_ai_provider(state: &mut AppState, provider: AIProvider) {
        let mut config = state.ai_config.write();
        config.provider = provider.clone();
        config.base_url = crate::services::ai::AIService::default_base_url(&provider).to_string();
        config.model = crate::services::ai::AIService::default_model(&provider).to_string();
    }
}
