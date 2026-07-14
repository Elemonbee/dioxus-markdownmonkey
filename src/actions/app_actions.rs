//! 应用级别 Actions / App-level Actions
//!
//! 处理主题切换、语言切换、侧边栏等全局操作

use crate::config::{SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::state::{AIProvider, AppState, Language, SidebarTab, Theme};
use dioxus::prelude::{ReadableExt, WritableExt};
use std::sync::Mutex;

/// 当前 AI 流取消发送端 / Cancel sender for the active AI stream
static AI_CANCEL_TX: Mutex<Option<tokio::sync::watch::Sender<bool>>> = Mutex::new(None);

/// 应用 Actions 处理器 / App Actions Handler
pub struct AppActions;

impl AppActions {
    /// 切换主题 / Toggle Theme
    pub fn toggle_theme(state: &mut AppState) {
        let mut ui = state.ui();
        let new_theme = match *ui.theme.read() {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::System,
            Theme::System => Theme::Dark,
        };
        *ui.theme.write() = new_theme;
    }

    /// 设置主题 / Set Theme
    pub fn set_theme(state: &mut AppState, theme: Theme) {
        *state.ui().theme.write() = theme;
    }

    /// 切换语言 / Toggle Language
    pub fn toggle_language(state: &mut AppState) {
        let mut ui = state.ui();
        let new_lang = match *ui.language.read() {
            Language::ZhCN => Language::EnUS,
            Language::EnUS => Language::ZhCN,
        };
        *ui.language.write() = new_lang;
    }

    /// 设置语言 / Set Language
    pub fn set_language(state: &mut AppState, language: Language) {
        *state.ui().language.write() = language;
    }

    /// 切换侧边栏 / Toggle Sidebar
    pub fn toggle_sidebar(state: &mut AppState) {
        let mut ui = state.ui();
        let current = *ui.sidebar_visible.read();
        *ui.sidebar_visible.write() = !current;
    }

    /// 设置侧边栏可见性 / Set Sidebar Visibility
    pub fn set_sidebar_visible(state: &mut AppState, visible: bool) {
        *state.ui().sidebar_visible.write() = visible;
    }

    /// 切换预览 / Toggle Preview
    pub fn toggle_preview(state: &mut AppState) {
        let mut ui = state.ui();
        let current = *ui.show_preview.read();
        *ui.show_preview.write() = !current;
    }

    /// 切换侧边栏标签 / Toggle Sidebar Tab
    pub fn set_sidebar_tab(state: &mut AppState, tab: SidebarTab) {
        *state.ui().sidebar_tab.write() = tab;
    }

    /// 设置侧边栏宽度 / Set Sidebar Width
    pub fn set_sidebar_width(state: &mut AppState, width: u32) {
        *state.ui().sidebar_width.write() = width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
    }

    /// 显示设置弹窗 / Show Settings Modal
    pub fn show_settings(state: &mut AppState) {
        *state.ui().show_settings.write() = true;
    }

    /// 隐藏设置弹窗 / Hide Settings Modal
    pub fn hide_settings(state: &mut AppState) {
        *state.ui().show_settings.write() = false;
    }

    /// 显示快捷键弹窗 / Show Shortcuts Modal
    pub fn show_shortcuts(state: &mut AppState) {
        *state.ui().show_shortcuts.write() = true;
    }

    /// 隐藏快捷键弹窗 / Hide Shortcuts Modal
    pub fn hide_shortcuts(state: &mut AppState) {
        *state.ui().show_shortcuts.write() = false;
    }

    /// 显示 AI 聊天弹窗 / Show AI Chat Modal
    pub fn show_ai_chat(state: &mut AppState) {
        let doc = state.document();
        let ui = state.ui();
        let mut ai = state.ai();
        let content = doc.content.read();
        let has_selection = crate::services::ai::selected_ai_context(
            &content,
            *ui.cursor_start.read(),
            *ui.cursor_end.read(),
        )
        .is_some();
        *ai.ai_use_selection.write() = has_selection;
        *ai.show_ai_chat.write() = true;
    }

    /// 隐藏 AI 聊天弹窗 / Hide AI Chat Modal
    pub fn hide_ai_chat(state: &mut AppState) {
        *state.ai().show_ai_chat.write() = false;
    }

    /// 显示 AI 结果弹窗 / Show AI Result Modal
    pub fn show_ai_result(state: &mut AppState) {
        *state.ai().show_ai_result.write() = true;
    }

    /// 隐藏 AI 结果弹窗 / Hide AI Result Modal
    pub fn hide_ai_result(state: &mut AppState) {
        *state.ai().show_ai_result.write() = false;
    }

    /// 开始一轮 AI 生成，返回世代号与取消接收端
    /// Start an AI generation; returns generation epoch and cancel receiver
    pub fn start_ai_generation(
        state: &mut AppState,
    ) -> (u64, tokio::sync::watch::Receiver<bool>) {
        let (tx, rx) = tokio::sync::watch::channel(false);
        if let Ok(mut guard) = AI_CANCEL_TX.lock() {
            *guard = Some(tx);
        }
        let mut ai = state.ai();
        let next = *ai.ai_generation_id.read() + 1;
        *ai.ai_generation_id.write() = next;
        *ai.ai_loading.write() = true;
        (next, rx)
    }

    /// 取消当前 AI 生成（抬高世代号、发取消信号并结束 loading）
    /// Cancel current AI generation (bump epoch, signal cancel, clear loading)
    pub fn cancel_ai_generation(state: &mut AppState) {
        if let Ok(mut guard) = AI_CANCEL_TX.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(true);
            }
        }
        let mut ai = state.ai();
        let next = *ai.ai_generation_id.read() + 1;
        *ai.ai_generation_id.write() = next;
        *ai.ai_loading.write() = false;
    }

    /// 从结果弹窗继续提问：关闭结果、打开聊天并预填上次用户问题
    /// Follow up from result modal: hide result, show chat, prefill last user turn
    pub fn follow_up_ai_chat(state: &mut AppState) {
        let mut ai = state.ai();
        let prefill = ai
            .ai_history
            .read()
            .iter()
            .rev()
            .find(|t| t.role == "user")
            .map(|t| t.content.clone())
            .unwrap_or_default();
        *ai.ai_input.write() = prefill;
        *ai.show_ai_result.write() = false;
        Self::show_ai_chat(state);
    }

    /// 截取 transcript 时间窗口（正序，最近 max_msgs 条）
    /// Take chronological transcript window (oldest→newest, last max_msgs)
    pub fn transcript_window(
        history: &[crate::state::ChatTurn],
        max_msgs: usize,
    ) -> Vec<&crate::state::ChatTurn> {
        let skip = history.len().saturating_sub(max_msgs);
        history[skip..].iter().collect()
    }

    /// 清空 AI 会话历史 / Clear AI conversation history
    pub fn clear_ai_history(state: &mut AppState) {
        state.ai().ai_history.write().clear();
        let key = state.current_ai_session_key();
        if let Err(e) = crate::services::settings::save_ai_history_for_key(&key, &[]) {
            tracing::warn!("Failed to persist cleared AI history: {}", e);
        }
    }

    /// 追加一轮 AI 会话 / Append one AI conversation turn pair
    pub fn push_ai_turn(state: &mut AppState, user: String, assistant: String) {
        if user.is_empty() && assistant.is_empty() {
            return;
        }
        {
            let mut ai = state.ai();
            let mut hist = ai.ai_history.write();
            if !user.is_empty() {
                hist.push(crate::state::ChatTurn::user(user));
            }
            if !assistant.is_empty() {
                hist.push(crate::state::ChatTurn::assistant(assistant));
            }
            // 上限 10 轮（20 条）/ Cap at 10 turns (20 messages)
            const MAX: usize = crate::services::settings::AI_HISTORY_MAX_MESSAGES;
            if hist.len() > MAX {
                let excess = hist.len() - MAX;
                hist.drain(0..excess);
            }
        }
        let key = state.current_ai_session_key();
        let snapshot = state.ai().ai_history.read().clone();
        if let Err(e) = crate::services::settings::save_ai_history_for_key(&key, &snapshot) {
            tracing::warn!("Failed to persist AI history: {}", e);
        }
    }

    /// 关闭所有弹窗 / Close All Modals
    pub fn close_all_modals(state: &mut AppState) {
        let mut ui = state.ui();
        let mut ai = state.ai();
        *ui.show_settings.write() = false;
        *ui.show_shortcuts.write() = false;
        *ai.show_ai_chat.write() = false;
        *ai.show_ai_result.write() = false;
    }

    /// Close all modal-like overlays including search panels.
    pub fn close_overlays(state: &mut AppState) {
        Self::close_all_modals(state);
        let mut ui = state.ui();
        *ui.show_search.write() = false;
        *ui.show_global_search.write() = false;
        *ui.show_table_editor.write() = false;
    }

    /// Update AI provider: persist current key, then load target provider key from keyring.
    /// 切换 AI 提供商：先保存当前密钥，再从密钥环加载目标提供商密钥
    pub fn set_ai_provider(state: &mut AppState, provider: AIProvider) {
        let mut ai = state.ai();
        // 切换前把当前内存中的 key 写回旧提供商条目 / Persist in-memory key under the old provider first
        {
            let config = ai.ai_config.read();
            let old_id = config.provider.as_str();
            if !config.api_key.is_empty() {
                if let Err(e) =
                    crate::services::keyring_service::store_api_key(old_id, &config.api_key)
                {
                    tracing::warn!("Failed to store API key before provider switch: {}", e);
                }
            }
        }

        let provider_id = provider.as_str().to_string();
        let api_key =
            crate::services::keyring_service::get_api_key(&provider_id).unwrap_or_default();

        let mut config = ai.ai_config.write();
        config.provider = provider.clone();
        config.base_url = crate::services::ai::AIService::default_base_url(&provider).to_string();
        config.model = crate::services::ai::AIService::default_model(&provider).to_string();
        config.api_key = api_key;
    }

    /// 显示表格编辑器 / Show table editor modal
    pub fn show_table_editor(state: &mut AppState) {
        *state.ui().show_table_editor.write() = true;
    }

    /// 隐藏表格编辑器 / Hide table editor modal
    pub fn hide_table_editor(state: &mut AppState) {
        *state.ui().show_table_editor.write() = false;
    }

    /// 切换自动保存 / Toggle auto-save
    pub fn toggle_auto_save(state: &mut AppState) {
        let mut ui = state.ui();
        let enabled = !*ui.auto_save_enabled.read();
        *ui.auto_save_enabled.write() = enabled;
    }

    /// 设置 AI 是否使用选区 / Set whether AI uses selection context
    pub fn set_ai_use_selection(state: &mut AppState, use_selection: bool) {
        *state.ai().ai_use_selection.write() = use_selection;
    }

    /// 设置 AI 输入框内容 / Set AI input text
    pub fn set_ai_input(state: &mut AppState, input: String) {
        *state.ai().ai_input.write() = input;
    }

    /// 清空 AI 输入框 / Clear AI input text
    pub fn clear_ai_input(state: &mut AppState) {
        *state.ai().ai_input.write() = String::new();
    }

    /// 设置自动保存间隔（秒）/ Set auto-save interval in seconds
    pub fn set_auto_save_interval(state: &mut AppState, secs: u32) {
        use crate::config::{AUTO_SAVE_INTERVAL_MAX_SECS, AUTO_SAVE_INTERVAL_MIN_SECS};
        *state.ui().auto_save_interval.write() =
            secs.clamp(AUTO_SAVE_INTERVAL_MIN_SECS, AUTO_SAVE_INTERVAL_MAX_SECS);
    }

    /// 忽略外部文件修改提示 / Dismiss external file-modified prompt
    pub fn dismiss_file_external_modified(state: &mut AppState) {
        state.refresh_file_watch();
        *state.document().file_external_modified.write() = false;
    }
}
