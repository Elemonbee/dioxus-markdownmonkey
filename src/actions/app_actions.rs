//! 应用级别 Actions / App-level Actions
//!
//! 处理主题切换、语言切换、侧边栏等全局操作

use crate::actions::EditorActions;
use crate::config::{clamp_chat_context_chars, SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::services::ai::{
    clip_chat_document_context, format_ai_error, selected_ai_context, truncate_ai_context,
    AIService, AITask,
};
use crate::state::{AIProvider, AiApplyContext, AppState, Language, SidebarTab, Theme};
use crate::utils::i18n::t;
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

    /// 显示 AI 聊天弹窗；结果已关却仍 loading 时复位，避免空白窗
    /// Show the AI chat modal; reset a stuck loading flag if the result modal is already closed
    pub fn show_ai_chat(state: &mut AppState) {
        let has_selection = {
            let doc = state.document();
            let ui = state.ui();
            let content = doc.content.read();
            let start = *ui.cursor_start.read();
            let end = *ui.cursor_end.read();
            crate::services::ai::selected_ai_context(&content, start, end).is_some()
        };
        Self::set_ai_use_selection(state, has_selection);
        let stuck_loading = *state.ai().ai_loading.read()
            && !*state.ai().show_ai_result.read()
            && !*state.ai().show_ai_chat.read();
        if stuck_loading {
            Self::cancel_ai_generation(state);
        }
        *state.ai().show_ai_chat.write() = true;
    }

    /// 隐藏 AI 聊天弹窗；若正在聊则取消生成
    /// Hide the AI chat modal; cancel an in-flight chat generation
    pub fn hide_ai_chat(state: &mut AppState) {
        let chat_streaming = *state.ai().ai_loading.read() && !*state.ai().show_ai_result.read();
        if chat_streaming {
            Self::cancel_ai_generation(state);
        }
        *state.ai().show_ai_chat.write() = false;
    }

    /// 显示 AI 结果弹窗 / Show AI Result Modal
    pub fn show_ai_result(state: &mut AppState) {
        *state.ai().show_ai_result.write() = true;
    }

    /// 隐藏 AI 结果弹窗；生成中则先取消
    /// Hide the AI result modal; cancel an in-flight generation first
    pub fn hide_ai_result(state: &mut AppState) {
        if *state.ai().ai_loading.read() {
            Self::cancel_ai_generation(state);
        }
        *state.ai().show_ai_result.write() = false;
    }

    /// 准备 AI 结果弹窗（清空正文、设标题并显示）
    /// Prepare the AI result modal (clear body, set title, then show)
    pub fn prepare_ai_result(state: &mut AppState, title: String) {
        {
            let mut ai = state.ai();
            *ai.ai_result.write() = String::new();
            *ai.ai_title.write() = title;
        }
        Self::show_ai_result(state);
    }

    /// 当前世代仍有效时追加流式片段 / Append a stream chunk if this generation is still current
    pub fn append_ai_chunk(state: &mut AppState, generation_id: u64, chunk: &str) {
        let mut ai = state.ai();
        if *ai.ai_generation_id.read() == generation_id {
            ai.ai_result.write().push_str(chunk);
        }
    }

    /// 当前世代是否仍是活动生成 / Whether generation_id is still the active generation
    pub fn is_ai_generation_current(state: &AppState, generation_id: u64) -> bool {
        *state.ai().ai_generation_id.read() == generation_id
    }

    /// 读取当前 AI 结果正文 / Read the current AI result text
    pub fn ai_result_text(state: &AppState) -> String {
        state.ai().ai_result.read().clone()
    }

    /// 结束当前世代的 loading；世代过期则返回 false
    /// Finish loading for this generation; returns false if the generation is stale
    pub fn finish_ai_generation(state: &mut AppState, generation_id: u64) -> bool {
        let mut ai = state.ai();
        if *ai.ai_generation_id.read() != generation_id {
            return false;
        }
        *ai.ai_loading.write() = false;
        true
    }

    /// 把错误写入结果弹窗 / Write an error into the result modal
    pub fn set_ai_error_result(state: &mut AppState, message: String, title: String) {
        {
            let mut ai = state.ai();
            *ai.ai_result.write() = message;
            *ai.ai_title.write() = title;
        }
        Self::show_ai_result(state);
    }

    /// 开始一轮 AI 生成，返回世代号与取消接收端
    /// Start an AI generation; returns generation epoch and cancel receiver
    pub fn start_ai_generation(state: &mut AppState) -> (u64, tokio::sync::watch::Receiver<bool>) {
        let (tx, rx) = tokio::sync::watch::channel(false);
        if let Ok(mut guard) = AI_CANCEL_TX.lock() {
            if let Some(old) = guard.take() {
                let _ = old.send(true);
            }
            *guard = Some(tx);
        }
        let mut ai = state.ai();
        let next = *ai.ai_generation_id.read() + 1;
        *ai.ai_generation_id.write() = next;
        *ai.ai_loading.write() = true;
        if let Some(ctx) = ai.ai_apply_context.write().as_mut() {
            ctx.is_error = false;
        }
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

    /// 从结果弹窗打开聊天，不预填任务提示词
    /// Open chat from the result modal without prefilling task prompts
    pub fn follow_up_ai_chat(state: &mut AppState) {
        *state.ai().ai_input.write() = String::new();
        *state.ai().show_ai_result.write() = false;
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
        *state.ui().show_settings.write() = false;
        *state.ui().show_shortcuts.write() = false;
        Self::hide_ai_chat(state);
        Self::hide_ai_result(state);
    }

    /// Close all modal-like overlays including search panels.
    pub fn close_overlays(state: &mut AppState) {
        if *state.ai().ai_loading.read() {
            Self::cancel_ai_generation(state);
        }
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

    /// 插入表格 Markdown 并关闭表格编辑器 / Insert table markdown and close the table editor
    pub async fn insert_table_and_close(state: &mut AppState, markdown: String) {
        crate::actions::EditorActions::insert_text_from_dom(state, &markdown).await;
        Self::hide_table_editor(state);
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

    /// 设置翻译任务的目标语言 / Set the translate-task target language
    pub fn set_ai_translate_target(state: &mut AppState, target: Language) {
        *state.ai().ai_translate_target.write() = target;
    }

    /// 超长上下文：警告确认，超硬上限则截断；取消返回 None
    /// Warn on long context; hard-truncate above the cap; None if the user cancels
    pub fn apply_ai_context_limit(content: String, lang: Language) -> Option<String> {
        use crate::config::{AI_CONTEXT_HARD_MAX_CHARS, AI_CONTEXT_WARN_CHARS};
        let chars = content.chars().count();
        if chars > AI_CONTEXT_HARD_MAX_CHARS {
            rfd::MessageDialog::new()
                .set_title(t("ai_context_truncated_title", lang))
                .set_description(t("ai_context_truncated_msg", lang))
                .set_buttons(rfd::MessageButtons::Ok)
                .set_level(rfd::MessageLevel::Warning)
                .show();
            return Some(truncate_ai_context(&content, AI_CONTEXT_HARD_MAX_CHARS));
        }
        if chars > AI_CONTEXT_WARN_CHARS {
            let confirmed = rfd::MessageDialog::new()
                .set_title(t("ai_context_warn_title", lang))
                .set_description(t("ai_context_warn_msg", lang))
                .set_buttons(rfd::MessageButtons::OkCancel)
                .set_level(rfd::MessageLevel::Warning)
                .show();
            if confirmed != rfd::MessageDialogResult::Ok {
                return None;
            }
        }
        Some(content)
    }

    /// 标记最近一次 AI 请求是否以错误结束
    /// Mark whether the latest AI request ended in error
    pub fn mark_ai_apply_error(state: &mut AppState, is_error: bool) {
        if let Some(ctx) = state.ai().ai_apply_context.write().as_mut() {
            ctx.is_error = is_error;
        }
    }

    /// 从聊天弹窗发起一轮 AI 任务（flush、选区快照、限流、流式请求）
    /// Start an AI task from the chat modal (flush, selection snapshot, limits, stream)
    pub async fn run_ai_task(
        state: &mut AppState,
        task_id: String,
        title_error: String,
        error_prefix: String,
    ) {
        EditorActions::flush_from_dom(state).await;

        let lang = *state.ui().language.read();
        let body = state.document().content.read().clone();
        let start = *state.ui().cursor_start.read();
        let end = *state.ui().cursor_end.read();
        let selected = selected_ai_context(&body, start, end);
        let task = AITask::from_str_id(&task_id);
        if task.requires_selection() && selected.is_none() {
            rfd::MessageDialog::new()
                .set_title(t("ai_assistant", lang))
                .set_description(t("ai_need_selection", lang))
                .set_buttons(rfd::MessageButtons::Ok)
                .set_level(rfd::MessageLevel::Warning)
                .show();
            return;
        }
        let input = state.ai().ai_input.read().clone();
        if task == AITask::Custom && input.trim().is_empty() {
            return;
        }
        // 聊天不按选区写回；预设任务才替换/插在选区
        // Chat never writes back by selection; only presets replace/insert against it
        let used_selection = task != AITask::Custom && selected.is_some();
        let (source_start, source_end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let source_text = if used_selection {
            selected.clone().unwrap_or_default()
        } else {
            String::new()
        };
        let raw_content = if task == AITask::Custom {
            let max_chars =
                clamp_chat_context_chars(state.ai().ai_config.read().chat_context_chars);
            clip_chat_document_context(&body, selected.as_deref(), start, max_chars)
        } else if used_selection {
            selected.unwrap_or_else(|| body.clone())
        } else if body.is_empty() {
            String::new()
        } else {
            body
        };
        let content = if task == AITask::Custom {
            raw_content
        } else {
            let Some(limited) = Self::apply_ai_context_limit(raw_content, lang) else {
                return;
            };
            limited
        };

        *state.ai().ai_apply_context.write() = Some(AiApplyContext {
            task_id,
            used_selection,
            source_start,
            source_end,
            source_text,
            request_content: content.clone(),
            request_input: input.clone(),
            is_error: false,
        });

        Self::dispatch_ai_request(state, lang, title_error, error_prefix).await;
    }

    /// 从编辑器右键菜单启动预设任务（清空聊天输入，避免大纲误用）
    /// Launch a preset AI task from the editor context menu (clear chat input so Outline is not hijacked)
    pub async fn run_editor_ai_preset(
        state: &mut AppState,
        task_id: String,
        translate_target: Option<Language>,
        title_error: String,
        error_prefix: String,
    ) {
        if let Some(target) = translate_target {
            Self::set_ai_translate_target(state, target);
        }
        Self::clear_ai_input(state);
        Self::run_ai_task(state, task_id, title_error, error_prefix).await;
    }

    /// 重试最近一次失败的 AI 请求 / Retry the last failed AI request
    pub async fn retry_ai_task(state: &mut AppState, title_error: String, error_prefix: String) {
        let Some(ctx) = state.ai().ai_apply_context.read().clone() else {
            return;
        };
        if !ctx.is_error {
            return;
        }
        let lang = *state.ui().language.read();
        Self::dispatch_ai_request(state, lang, title_error, error_prefix).await;
    }

    /// 发送已快照的 AI 请求并写入结果弹窗
    /// Dispatch a snapshotted AI request and write into the result modal
    async fn dispatch_ai_request(
        state: &mut AppState,
        lang: Language,
        title_error: String,
        error_prefix: String,
    ) {
        let Some(ctx) = state.ai().ai_apply_context.read().clone() else {
            return;
        };
        let translate_target = *state.ai().ai_translate_target.read();
        let config = state.ai().ai_config.read().clone();
        let task = AITask::from_str_id(&ctx.task_id);
        let (history, global_system) = if task.uses_chat_session() {
            (
                state.ai().ai_history.read().clone(),
                config.system_prompt.clone(),
            )
        } else {
            (Vec::new(), String::new())
        };
        let (generation_id, cancel_rx) = Self::start_ai_generation(state);
        if task.uses_chat_session() {
            *state.ai().ai_result.write() = String::new();
            Self::clear_ai_input(state);
        } else {
            Self::hide_ai_chat(state);
            let result_title = t(task.title_i18n_key(), lang);
            Self::prepare_ai_result(state, result_title);
        }

        let service = AIService::with_temperature(
            config.api_key,
            Some(config.base_url),
            Some(config.model),
            config.temperature,
        );
        let messages = task.build_messages_localized(
            &ctx.request_content,
            &ctx.request_input,
            &history,
            &global_system,
            lang,
            translate_target,
        );
        let user_summary = task.history_user_summary_localized(
            &ctx.request_content,
            &ctx.request_input,
            lang,
            translate_target,
        );
        let mut stream_state = *state;
        let check_state = *state;

        let result = service
            .chat_stream_cancellable(
                messages,
                |chunk| {
                    AppActions::append_ai_chunk(&mut stream_state, generation_id, chunk);
                },
                || AppActions::is_ai_generation_current(&check_state, generation_id),
                cancel_rx,
            )
            .await;

        if !Self::finish_ai_generation(state, generation_id) {
            return;
        }

        match result {
            Ok(full) => {
                Self::mark_ai_apply_error(state, false);
                let assistant = if full.is_empty() {
                    Self::ai_result_text(state)
                } else {
                    full
                };
                if task.uses_chat_session() {
                    if !assistant.is_empty() {
                        Self::push_ai_turn(state, user_summary, assistant);
                    }
                    *state.ai().ai_result.write() = String::new();
                }
            }
            Err(crate::services::ai::AIError::Cancelled) => {
                Self::mark_ai_apply_error(state, false);
                let partial = Self::ai_result_text(state);
                if task.uses_chat_session() && !partial.is_empty() {
                    Self::push_ai_turn(state, user_summary, partial);
                    *state.ai().ai_result.write() = String::new();
                }
            }
            Err(e) => {
                Self::mark_ai_apply_error(state, true);
                if Self::ai_result_text(state).is_empty() {
                    let message = format_ai_error(&e, &error_prefix);
                    if task.uses_chat_session() {
                        *state.ai().ai_result.write() = message;
                    } else {
                        Self::set_ai_error_result(state, message, title_error);
                    }
                }
            }
        }
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
