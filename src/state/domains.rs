//! 按领域拆分的状态视图 / Domain-sliced state views
//!
//! `AppState` 仍保留扁平 Signal 字段以兼容现有调用；
//! 这些视图持有同一批 Signal 句柄（Copy），便于按域阅读与后续迁移。
//! `AppState` keeps flat Signal fields for compatibility;
//! these views hold the same Signal handles (Copy) for domain-oriented access.

#[cfg(test)]
use super::types::History as DocumentHistory;
use super::types::{
    AIConfig, AiApplyContext, ChatTurn, CloseTabSnapshot, Language, OutlineItem, SaveStatus,
    SidebarTab, TabId, TabInfo, Theme,
};
use super::AppState;
use crate::utils::file_encoding::FileEncoding;
use dioxus::prelude::Signal;
use std::path::PathBuf;

/// 文档域状态视图 / Document-domain state view
#[derive(Clone, Copy)]
pub struct DocumentState {
    pub current_file: Signal<Option<PathBuf>>,
    pub content: Signal<String>,
    pub modified: Signal<bool>,
    #[cfg(test)]
    pub history: Signal<DocumentHistory>,
    pub save_status: Signal<SaveStatus>,
    pub last_saved: Signal<Option<std::time::Instant>>,
    pub tabs: Signal<Vec<TabInfo>>,
    pub current_tab_index: Signal<usize>,
    pub outline_items: Signal<Vec<OutlineItem>>,
    pub file_encoding: Signal<FileEncoding>,
    pub content_revision: Signal<u64>,
    pub tab_access_clock: Signal<u64>,
    pub file_external_modified: Signal<bool>,
    pub file_watch_refresh_seq: Signal<u64>,
    pub show_close_confirm: Signal<bool>,
    pub pending_close_tab_id: Signal<Option<TabId>>,
    pub pending_close_save_as: Signal<Option<CloseTabSnapshot>>,
    pub trigger_save_as: Signal<bool>,
    pub show_large_file_warning: Signal<bool>,
    pub file_size_bytes: Signal<usize>,
    pub pending_large_file: Signal<Option<PathBuf>>,
}

/// UI 域状态视图 / UI-domain state view
#[derive(Clone, Copy)]
pub struct UiState {
    pub theme: Signal<Theme>,
    pub language: Signal<Language>,
    pub sidebar_visible: Signal<bool>,
    pub show_preview: Signal<bool>,
    pub sidebar_width: Signal<u32>,
    pub sidebar_tab: Signal<SidebarTab>,
    pub show_settings: Signal<bool>,
    pub show_shortcuts: Signal<bool>,
    pub show_search: Signal<bool>,
    pub show_global_search: Signal<bool>,
    pub show_table_editor: Signal<bool>,
    pub search_query: Signal<String>,
    pub replace_query: Signal<String>,
    pub search_regex: Signal<bool>,
    pub search_case_insensitive: Signal<bool>,
    pub search_index: Signal<usize>,
    pub search_total: Signal<usize>,
    pub font_size: Signal<u32>,
    pub preview_font_size: Signal<u32>,
    pub word_wrap: Signal<bool>,
    pub line_numbers: Signal<bool>,
    pub sync_scroll: Signal<bool>,
    pub cursor_start: Signal<usize>,
    pub cursor_end: Signal<usize>,
    pub workspace_root: Signal<Option<PathBuf>>,
    pub file_list: Signal<Vec<PathBuf>>,
    pub auto_save_enabled: Signal<bool>,
    pub auto_save_interval: Signal<u32>,
    pub session_restore_enabled: Signal<bool>,
}

/// AI 域状态视图 / AI-domain state view
#[derive(Clone, Copy)]
pub struct AiState {
    pub ai_config: Signal<AIConfig>,
    pub ai_loading: Signal<bool>,
    pub ai_result: Signal<String>,
    pub ai_title: Signal<String>,
    pub ai_input: Signal<String>,
    pub ai_use_selection: Signal<bool>,
    pub ai_history: Signal<Vec<ChatTurn>>,
    pub ai_generation_id: Signal<u64>,
    pub ai_apply_context: Signal<Option<AiApplyContext>>,
    pub ai_translate_target: Signal<Language>,
    pub show_ai_chat: Signal<bool>,
    pub show_ai_result: Signal<bool>,
}

impl AppState {
    /// 获取文档域视图（与扁平字段共享同一批 Signal）
    /// Document-domain view sharing the same Signal handles
    pub fn document(self) -> DocumentState {
        DocumentState {
            current_file: self.current_file,
            content: self.content,
            modified: self.modified,
            #[cfg(test)]
            history: self.history,
            save_status: self.save_status,
            last_saved: self.last_saved,
            tabs: self.tabs,
            current_tab_index: self.current_tab_index,
            outline_items: self.outline_items,
            file_encoding: self.file_encoding,
            content_revision: self.content_revision,
            tab_access_clock: self.tab_access_clock,
            file_external_modified: self.file_external_modified,
            file_watch_refresh_seq: self.file_watch_refresh_seq,
            show_close_confirm: self.show_close_confirm,
            pending_close_tab_id: self.pending_close_tab_id,
            pending_close_save_as: self.pending_close_save_as,
            trigger_save_as: self.trigger_save_as,
            show_large_file_warning: self.show_large_file_warning,
            file_size_bytes: self.file_size_bytes,
            pending_large_file: self.pending_large_file,
        }
    }

    /// 获取 UI 域视图 / UI-domain view
    pub fn ui(self) -> UiState {
        UiState {
            theme: self.theme,
            language: self.language,
            sidebar_visible: self.sidebar_visible,
            show_preview: self.show_preview,
            sidebar_width: self.sidebar_width,
            sidebar_tab: self.sidebar_tab,
            show_settings: self.show_settings,
            show_shortcuts: self.show_shortcuts,
            show_search: self.show_search,
            show_global_search: self.show_global_search,
            show_table_editor: self.show_table_editor,
            search_query: self.search_query,
            replace_query: self.replace_query,
            search_regex: self.search_regex,
            search_case_insensitive: self.search_case_insensitive,
            search_index: self.search_index,
            search_total: self.search_total,
            font_size: self.font_size,
            preview_font_size: self.preview_font_size,
            word_wrap: self.word_wrap,
            line_numbers: self.line_numbers,
            sync_scroll: self.sync_scroll,
            cursor_start: self.cursor_start,
            cursor_end: self.cursor_end,
            workspace_root: self.workspace_root,
            file_list: self.file_list,
            auto_save_enabled: self.auto_save_enabled,
            auto_save_interval: self.auto_save_interval,
            session_restore_enabled: self.session_restore_enabled,
        }
    }

    /// 获取 AI 域视图 / AI-domain view
    pub fn ai(self) -> AiState {
        AiState {
            ai_config: self.ai_config,
            ai_loading: self.ai_loading,
            ai_result: self.ai_result,
            ai_title: self.ai_title,
            ai_input: self.ai_input,
            ai_use_selection: self.ai_use_selection,
            ai_history: self.ai_history,
            ai_generation_id: self.ai_generation_id,
            ai_apply_context: self.ai_apply_context,
            ai_translate_target: self.ai_translate_target,
            show_ai_chat: self.show_ai_chat,
            show_ai_result: self.show_ai_result,
        }
    }
}
