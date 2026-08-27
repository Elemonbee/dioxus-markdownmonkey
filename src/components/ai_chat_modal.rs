//! AI 聊天弹窗组件 / AI Chat Modal Component

use crate::actions::{AppActions, EditorActions};
use crate::components::icons::{
    CloseIcon, ContinueIcon, GrammarIcon, ImproveIcon, OutlineIcon, TranslateIcon,
};
use crate::services::ai::{format_ai_error, selected_ai_context, AIService, AITask};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// AI 聊天弹窗 / AI Chat Modal
#[component]
pub fn AiChatModal() -> Element {
    let mut state = use_context::<AppState>();
    let ai = state.ai();
    let ui = state.ui();
    let doc = state.document();
    let show = *ai.show_ai_chat.read();
    let lang = *ui.language.read();

    // i18n
    let ai_title_t = t("ai_assistant", lang);
    let ai_not_enabled_t = t("ai_not_enabled", lang);
    let ai_configure_t = t("ai_configure", lang);
    let open_settings_t = t("open_settings_btn", lang);
    let ai_thinking_t = t("ai_thinking", lang);
    let continue_t = t("ai_continue", lang);
    let improve_t = t("ai_improve", lang);
    let outline_t = t("ai_outline", lang);
    let translate_t = t("ai_translate", lang);
    let grammar_t = t("ai_fix_grammar", lang);
    let placeholder_t = t("custom_input_placeholder", lang);
    let clear_t = t("clear", lang);
    let clear_history_t = t("ai_clear_history", lang);
    let clear_confirm_t = t("ai_clear_history_confirm", lang);
    let history_turns_t = t("ai_history_turns", lang);
    let transcript_user_t = t("ai_transcript_user", lang);
    let transcript_assistant_t = t("ai_transcript_assistant", lang);
    let send_t = t("send", lang);
    let ai_context_t = t("ai_context", lang);
    let ai_context_full_t = t("ai_context_full", lang);
    let ai_context_selection_t = t("ai_context_selection", lang);
    let ai_context_full_hint_t = t("ai_context_full_hint", lang);
    let ai_context_selection_hint_t = t("ai_context_selection_hint", lang);
    let chars_abbr_t = t("chars_abbr", lang);
    let aria_ai_transcript_t = t("aria_ai_transcript", lang);
    let transcript_copy_t = t("ai_transcript_copy", lang);
    let transcript_copied_t = t("ai_transcript_copied", lang);

    let display_class = if show { "" } else { "hidden" };
    let history_len = ai.ai_history.read().len();
    let history_turns = history_len / 2;
    let history_badge = if history_turns > 0 {
        history_turns_t.replace("{n}", &history_turns.to_string())
    } else {
        String::new()
    };
    let mut expanded_turn = use_signal(|| Option::<usize>::None);
    let mut copy_flash = use_signal(|| Option::<usize>::None);
    let history_snapshot: Vec<(usize, String, String, String, String)> = {
        let hist = ai.ai_history.read();
        AppActions::transcript_window(&hist, 20)
            .into_iter()
            .enumerate()
            .map(|(i, turn)| {
                let role_class = if turn.role == "assistant" {
                    "assistant".to_string()
                } else {
                    "user".to_string()
                };
                let role = if turn.role == "assistant" {
                    transcript_assistant_t.clone()
                } else {
                    transcript_user_t.clone()
                };
                let full = turn.content.clone();
                let preview: String = turn.content.chars().take(120).collect();
                let preview = if turn.content.chars().count() > 120 {
                    format!("{preview}…")
                } else {
                    preview
                };
                (i, role_class, role, preview, full)
            })
            .collect()
    };

    // 历史变化时滚到底部 / Scroll transcript to bottom when history changes
    use_effect(move || {
        let _ = history_len;
        if show && history_len > 0 {
            let _ = document::eval(
                r#"
                (function() {
                    const el = document.querySelector('.ai-transcript');
                    if (el) el.scrollTop = el.scrollHeight;
                })();
                "#,
            );
        }
    });

    // 打开弹窗时 flush、同步选区，并聚焦输入框 / Flush, sync selection, focus input on open
    let mut state_for_sel = state;
    use_effect(move || {
        if show {
            spawn(async move {
                EditorActions::flush_from_dom(&mut state_for_sel).await;
                let _ = document::eval(
                    r#"
                    (function() {
                        const input = document.querySelector('.ai-input');
                        if (input) { input.focus(); input.setSelectionRange(input.value.length, input.value.length); }
                    })();
                    "#,
                );
            });
        }
    });

    let ai_config = ai.ai_config.read().clone();
    let ai_enabled = ai_config.enabled;
    let ai_loading = *ai.ai_loading.read();
    let input_text = ai.ai_input.read().clone();
    let content = doc.content.read().clone();
    let cursor_start = *ui.cursor_start.read();
    let cursor_end = *ui.cursor_end.read();
    let selection_content = selected_ai_context(&content, cursor_start, cursor_end);
    let has_selection = selection_content.is_some();
    let use_selection = *ai.ai_use_selection.read() && has_selection;
    let context_content = if use_selection {
        selection_content.clone().unwrap_or_else(|| content.clone())
    } else {
        content.clone()
    };
    let context_hint = if use_selection {
        format!(
            "{} · {} {}",
            ai_context_selection_hint_t,
            context_content.chars().count(),
            chars_abbr_t
        )
    } else {
        format!(
            "{} · {} {}",
            ai_context_full_hint_t,
            context_content.chars().count(),
            chars_abbr_t
        )
    };

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                AppActions::hide_ai_chat(&mut state);
            },

            div {
                class: "modal ai-chat-modal",
                role: "dialog",
                "aria-modal": "true",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 {
                        "{ai_title_t}"
                        if !history_badge.is_empty() {
                            span { class: "ai-history-badge", " · {history_badge}" }
                        }
                    }
                    div { class: "modal-header-actions", style: "display: flex; gap: 8px; align-items: center;",
                        if history_len > 0 {
                            button {
                                class: "btn-secondary",
                                title: "{clear_history_t}",
                                onclick: move |_| {
                                    let confirmed = rfd::MessageDialog::new()
                                        .set_title(&clear_history_t)
                                        .set_description(&clear_confirm_t)
                                        .set_buttons(rfd::MessageButtons::OkCancel)
                                        .set_level(rfd::MessageLevel::Warning)
                                        .show();
                                    if confirmed == rfd::MessageDialogResult::Ok {
                                        AppActions::clear_ai_history(&mut state);
                                    }
                                },
                                "{clear_history_t}"
                            }
                        }
                        button {
                            class: "modal-close",
                            onclick: move |_| {
                                AppActions::hide_ai_chat(&mut state);
                            },
                            CloseIcon { size: 20 }
                        }
                    }
                }

                div { class: "modal-body",
                    // AI 未启用提示
                    div {
                        class: if !ai_enabled { "ai-empty visible" } else { "ai-empty hidden" },
                        style: if !ai_enabled { "" } else { "display: none;" },
                        p { "{ai_not_enabled_t}" }
                        p { "{ai_configure_t}" }
                        button {
                            class: "btn-primary",
                            onclick: move |_| {
                                AppActions::hide_ai_chat(&mut state);
                                AppActions::show_settings(&mut state);
                            },
                            "{open_settings_t}"
                        }
                    }

                    // 加载中状态
                    div {
                        class: if ai_enabled && ai_loading { "ai-loading visible" } else { "ai-loading hidden" },
                        style: if ai_enabled && ai_loading { "" } else { "display: none;" },
                        div { class: "spinner" }
                        p { "{ai_thinking_t}" }
                    }

                    // AI 功能区域
                    div {
                        class: if ai_enabled && !ai_loading { "ai-content visible" } else { "ai-content hidden" },
                        style: if ai_enabled && !ai_loading { "" } else { "display: none;" },

                        div { class: "ai-context",
                            span { class: "ai-context-label", "{ai_context_t}" }
                            div { class: "ai-context-toggle",
                                button {
                                    class: if !use_selection { "ai-context-option active" } else { "ai-context-option" },
                                    onclick: move |_| {
                                        AppActions::set_ai_use_selection(&mut state, false);
                                    },
                                    "{ai_context_full_t}"
                                }
                                button {
                                    class: if use_selection { "ai-context-option active" } else { "ai-context-option" },
                                    disabled: !has_selection,
                                    onclick: move |_| {
                                        if has_selection {
                                            AppActions::set_ai_use_selection(&mut state, true);
                                        }
                                    },
                                    "{ai_context_selection_t}"
                                }
                            }
                            span { class: "ai-context-hint", "{context_hint}" }
                        }

                        if !history_snapshot.is_empty() {
                            div { class: "ai-transcript", "aria-label": "{aria_ai_transcript_t}",
                                for (idx, role_class, role, preview, full) in history_snapshot {
                                    {
                                        let is_expanded = *expanded_turn.read() == Some(idx);
                                        let turn_class = if is_expanded {
                                            format!("ai-transcript-turn {role_class} expanded")
                                        } else {
                                            format!("ai-transcript-turn {role_class}")
                                        };
                                        let display_text = if is_expanded {
                                            full.clone()
                                        } else {
                                            preview.clone()
                                        };
                                        let copied = *copy_flash.read() == Some(idx);
                                        let copy_label = if copied {
                                            transcript_copied_t.clone()
                                        } else {
                                            transcript_copy_t.clone()
                                        };
                                        rsx! {
                                            div {
                                                class: "{turn_class}",
                                                title: "{full}",
                                                onclick: move |_| {
                                                    let cur = *expanded_turn.read();
                                                    expanded_turn.set(if cur == Some(idx) { None } else { Some(idx) });
                                                },
                                                div { class: "ai-transcript-turn-header",
                                                    span { class: "ai-transcript-role", "{role}" }
                                                    button {
                                                        class: "ai-transcript-copy",
                                                        title: "{copy_label}",
                                                        onclick: move |e| {
                                                            e.stop_propagation();
                                                            crate::utils::clipboard::copy_text(&full);
                                                                copy_flash.set(Some(idx));
                                                        },
                                                        "{copy_label}"
                                                    }
                                                }
                                                span { class: "ai-transcript-text", "{display_text}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "ai-actions",
                            AiActionBtn {
                                label: continue_t.clone(),
                                icon: Some("continue".to_string()),
                                task_type: "continue",
                                config: ai_config.clone(),
                                content: context_content.clone(),
                                input: input_text.clone(),
                            }
                            AiActionBtn {
                                label: improve_t.clone(),
                                icon: Some("improve".to_string()),
                                task_type: "improve",
                                config: ai_config.clone(),
                                content: context_content.clone(),
                                input: input_text.clone(),
                            }
                            AiActionBtn {
                                label: outline_t.clone(),
                                icon: Some("outline".to_string()),
                                task_type: "outline",
                                config: ai_config.clone(),
                                content: context_content.clone(),
                                input: input_text.clone(),
                            }
                            AiActionBtn {
                                label: translate_t.clone(),
                                icon: Some("translate".to_string()),
                                task_type: "translate",
                                config: ai_config.clone(),
                                content: context_content.clone(),
                                input: input_text.clone(),
                            }
                            AiActionBtn {
                                label: grammar_t.clone(),
                                icon: Some("grammar".to_string()),
                                task_type: "fix_grammar",
                                config: ai_config.clone(),
                                content: context_content.clone(),
                                input: input_text.clone(),
                            }
                        }

                        div { class: "ai-input-section",
                            textarea {
                                class: "ai-input",
                                placeholder: "{placeholder_t}",
                                value: "{input_text}",
                                oninput: move |e| {
                                    AppActions::set_ai_input(&mut state, e.value());
                                },
                            }

                            div { class: "ai-input-actions",
                                button {
                                    class: "btn-secondary",
                                    onclick: move |_| {
                                        AppActions::clear_ai_input(&mut state);
                                    },
                                    "{clear_t}"
                                }
                                AiActionBtn {
                                    label: send_t.clone(),
                                    task_type: "custom",
                                    config: ai_config.clone(),
                                    content: context_content.clone(),
                                    input: input_text.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// AI 操作按钮属性 / AI Action Button Props
#[derive(Props, Clone, PartialEq)]
struct AiActionBtnProps {
    label: String,
    #[props(default = None)]
    icon: Option<String>,
    task_type: String,
    config: crate::state::AIConfig,
    content: String,
    input: String,
}

/// AI 操作按钮 / AI Action Button
fn AiActionBtn(props: AiActionBtnProps) -> Element {
    let state = use_context::<AppState>();
    let ui = state.ui();
    let icon_type = props.icon.clone();
    let lang = *ui.language.read();

    // Pre-compute i18n strings for async use
    let title_error = t("ai_error", lang);
    let error_prefix = t("error", lang);

    rsx! {
        button {
            class: "ai-action-btn",
            onclick: move |_| {
                let task_type = props.task_type.clone();
                let api_key = props.config.api_key.clone();
                let base_url = props.config.base_url.clone();
                let model = props.config.model.clone();
                let temperature = props.config.temperature;
                let global_system = props.config.system_prompt.clone();
                let fallback_content = props.content.clone();
                let input = props.input.clone();

                let mut state = state;
                let te = title_error.clone();
                let ep = error_prefix.clone();

                spawn(async move {
                    let ui = state.ui();
                    let doc = state.document();
                    let ai = state.ai();

                    // 发送前 flush + 同步选区，避免非受控模式下上下文过期
                    // Flush + sync selection before send so uncontrolled context is fresh
                    EditorActions::flush_from_dom(&mut state).await;

                    let body = doc.content.read().clone();
                    let content = if *ai.ai_use_selection.read() {
                        selected_ai_context(
                            &body,
                            *ui.cursor_start.read(),
                            *ui.cursor_end.read(),
                        )
                        .unwrap_or(fallback_content)
                    } else if body.is_empty() {
                        fallback_content
                    } else {
                        body
                    };
                    let history = ai.ai_history.read().clone();

                    let (generation_id, cancel_rx) = AppActions::start_ai_generation(&mut state);
                    AppActions::hide_ai_chat(&mut state);

                    let mut ai_loading = ai.ai_loading;
                    let mut ai_result = ai.ai_result;
                    let mut ai_title = ai.ai_title;
                    let mut show_ai_result_signal = ai.show_ai_result;
                    let generation_signal = ai.ai_generation_id;

                    let task = AITask::from_str_id(&task_type);
                    let result_title = t(task.title_i18n_key(), lang);

                    *ai_result.write() = String::new();
                    *ai_title.write() = result_title;
                    AppActions::show_ai_result(&mut state);
                    *show_ai_result_signal.write() = true;

                    let service = AIService::with_temperature(
                        api_key,
                        Some(base_url),
                        Some(model),
                        temperature,
                    );

                    let messages = task.build_messages_with_history(
                        &content,
                        &input,
                        &history,
                        &global_system,
                    );
                    let user_summary = task.history_user_summary(&content, &input);

                    let result = service
                        .chat_stream_cancellable(
                            messages,
                            |chunk| {
                                if *generation_signal.read() == generation_id {
                                    ai_result.write().push_str(chunk);
                                }
                            },
                            || *generation_signal.read() == generation_id,
                            cancel_rx,
                        )
                        .await;

                    // 仅当前世代才收尾 / Only finish if this generation is still current
                    if *generation_signal.read() != generation_id {
                        return;
                    }
                    *ai_loading.write() = false;

                    match result {
                        Ok(full) => {
                            let assistant = if full.is_empty() {
                                ai_result.read().clone()
                            } else {
                                full
                            };
                            if !assistant.is_empty() {
                                AppActions::push_ai_turn(&mut state, user_summary, assistant);
                            }
                        }
                        Err(crate::services::ai::AIError::Cancelled) => {
                            let partial = ai_result.read().clone();
                            if !partial.is_empty() {
                                AppActions::push_ai_turn(&mut state, user_summary, partial);
                            }
                        }
                        Err(e) => {
                            let current_result = ai_result.read().clone();
                            if current_result.is_empty() {
                                *ai_result.write() = format_ai_error(&e, &ep);
                                *ai_title.write() = te;
                                *show_ai_result_signal.write() = true;
                            }
                        }
                    }
                });
            },
            if let Some(icon) = icon_type.as_ref() {
                match icon.as_str() {
                    "continue" => rsx! { ContinueIcon { size: 16 } },
                    "improve" => rsx! { ImproveIcon { size: 16 } },
                    "outline" => rsx! { OutlineIcon { size: 16 } },
                    "translate" => rsx! { TranslateIcon { size: 16 } },
                    "grammar" => rsx! { GrammarIcon { size: 16 } },
                    _ => rsx! {},
                }
            }
            span { "{props.label}" }
        }
    }
}
