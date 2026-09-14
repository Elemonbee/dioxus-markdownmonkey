//! AI 聊天弹窗组件 / AI Chat Modal Component

use crate::actions::{AppActions, EditorActions};
use crate::components::icons::CloseIcon;
use crate::config::clamp_chat_context_chars;
use crate::services::ai::{clip_chat_document_context, selected_ai_context, AITask};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 发送一轮聊天 / Send one chat round
fn launch_chat_send(mut state: AppState) {
    spawn(async move {
        let lang = *state.ui().language.read();
        AppActions::run_ai_task(
            &mut state,
            "custom".into(),
            t("ai_error", lang),
            t("error", lang),
        )
        .await;
    });
}

/// 重试上一轮失败的聊天 / Retry the last failed chat round
fn launch_chat_retry(mut state: AppState) {
    spawn(async move {
        let lang = *state.ui().language.read();
        AppActions::retry_ai_task(&mut state, t("ai_error", lang), t("error", lang)).await;
    });
}

/// AI 聊天弹窗：实时上下文对话，不跳转结果窗
/// AI chat modal: live-context conversation, no jump to the result modal
#[component]
pub fn AiChatModal() -> Element {
    let mut state = use_context::<AppState>();
    let ai = state.ai();
    let ui = state.ui();
    let doc = state.document();
    let show = *ai.show_ai_chat.read();
    let lang = *ui.language.read();

    let ai_title_t = t("ai_assistant", lang);
    let ai_not_enabled_t = t("ai_not_enabled", lang);
    let ai_configure_t = t("ai_configure", lang);
    let open_settings_t = t("open_settings_btn", lang);
    let ai_thinking_t = t("ai_thinking", lang);
    let placeholder_t = t("custom_input_placeholder", lang);
    let clear_history_t = t("ai_clear_history", lang);
    let clear_confirm_t = t("ai_clear_history_confirm", lang);
    let history_turns_t = t("ai_history_turns", lang);
    let transcript_user_t = t("ai_transcript_user", lang);
    let transcript_assistant_t = t("ai_transcript_assistant", lang);
    let send_t = t("send", lang);
    let stop_t = t("ai_stop", lang);
    let retry_t = t("ai_retry", lang);
    let aria_ai_transcript_t = t("aria_ai_transcript", lang);
    let transcript_copy_t = t("ai_transcript_copy", lang);
    let transcript_copied_t = t("ai_transcript_copied", lang);
    let context_attached_t = t("ai_chat_context_attached", lang);
    let empty_hint_t = t("ai_chat_empty", lang);

    let display_class = if show { "" } else { "hidden" };
    let history_len = ai.ai_history.read().len();
    let history_turns = history_len / 2;
    let history_badge = if history_turns > 0 {
        history_turns_t.replace("{n}", &history_turns.to_string())
    } else {
        String::new()
    };
    let mut copy_flash = use_signal(|| Option::<usize>::None);
    let history_snapshot: Vec<(usize, String, String, String)> = {
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
                (i, role_class, role, turn.content.clone())
            })
            .collect()
    };

    use_effect(move || {
        let _ = history_len;
        let _ = *ai.ai_result.read();
        if show {
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
    let can_send = ai_enabled && !ai_loading && !input_text.trim().is_empty();
    let content = doc.content.read().clone();
    let cursor_start = *ui.cursor_start.read();
    let cursor_end = *ui.cursor_end.read();
    let selection_content = selected_ai_context(&content, cursor_start, cursor_end);
    let max_chars = clamp_chat_context_chars(ai_config.chat_context_chars);
    let attached = clip_chat_document_context(
        &content,
        selection_content.as_deref(),
        cursor_start,
        max_chars,
    );
    let context_hint = context_attached_t
        .replace("{n}", &attached.chars().count().to_string())
        .replace("{max}", &max_chars.to_string());

    let apply_ctx = ai.ai_apply_context.read().clone();
    let chat_pending = apply_ctx
        .as_ref()
        .map(|ctx| AITask::from_str_id(&ctx.task_id).uses_chat_session())
        .unwrap_or(false);
    let chat_error = apply_ctx.as_ref().map(|ctx| ctx.is_error).unwrap_or(false);
    let pending_user = apply_ctx
        .as_ref()
        .map(|ctx| ctx.request_input.clone())
        .unwrap_or_default();
    let live_reply = ai.ai_result.read().clone();
    let show_pending = chat_pending
        && !pending_user.is_empty()
        && (ai_loading || chat_error || !live_reply.is_empty());

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

                div { class: "modal-body ai-chat-body",
                    div {
                        class: if !ai_enabled { "ai-empty" } else { "ai-empty hidden" },
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

                    div {
                        class: if ai_enabled { "ai-content ai-chat-content" } else { "ai-content hidden" },

                        span { class: "ai-context-hint", "{context_hint}" }

                        div { class: "ai-transcript", "aria-label": "{aria_ai_transcript_t}",
                            if history_snapshot.is_empty() && !show_pending {
                                p { class: "ai-chat-empty-hint", "{empty_hint_t}" }
                            }
                            for (idx, role_class, role, full) in history_snapshot {
                                {
                                    let copied = *copy_flash.read() == Some(idx);
                                    let copy_label = if copied {
                                        transcript_copied_t.clone()
                                    } else {
                                        transcript_copy_t.clone()
                                    };
                                    rsx! {
                                        div {
                                            class: "ai-transcript-turn {role_class}",
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
                                            span { class: "ai-transcript-text", "{full}" }
                                        }
                                    }
                                }
                            }
                            if show_pending {
                                div { class: "ai-transcript-turn user",
                                    div { class: "ai-transcript-turn-header",
                                        span { class: "ai-transcript-role", "{transcript_user_t}" }
                                    }
                                    span { class: "ai-transcript-text", "{pending_user}" }
                                }
                                div { class: "ai-transcript-turn assistant",
                                    div { class: "ai-transcript-turn-header",
                                        span { class: "ai-transcript-role", "{transcript_assistant_t}" }
                                    }
                                    span { class: "ai-transcript-text",
                                        if live_reply.is_empty() {
                                            "{ai_thinking_t}"
                                        } else {
                                            "{live_reply}"
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "ai-input-section",
                            textarea {
                                class: "ai-input",
                                placeholder: "{placeholder_t}",
                                value: "{input_text}",
                                disabled: ai_loading,
                                oninput: move |e| {
                                    AppActions::set_ai_input(&mut state, e.value());
                                },
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter && !e.modifiers().shift() {
                                        e.prevent_default();
                                        if can_send {
                                            launch_chat_send(state);
                                        }
                                    }
                                },
                            }

                            div { class: "ai-input-actions",
                                if ai_loading {
                                    button {
                                        class: "btn-secondary",
                                        onclick: move |_| {
                                            AppActions::cancel_ai_generation(&mut state);
                                        },
                                        "{stop_t}"
                                    }
                                }
                                if chat_error && !ai_loading {
                                    button {
                                        class: "btn-secondary",
                                        onclick: move |_| {
                                            launch_chat_retry(state);
                                        },
                                        "{retry_t}"
                                    }
                                }
                                button {
                                    class: "ai-action-btn",
                                    disabled: !can_send,
                                    onclick: move |_| {
                                        if can_send {
                                            launch_chat_send(state);
                                        }
                                    },
                                    span { "{send_t}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
