//! AI 聊天弹窗组件 / AI Chat Modal Component

use crate::actions::AppActions;
use crate::components::icons::{
    CloseIcon, ContinueIcon, GrammarIcon, ImproveIcon, OutlineIcon, TranslateIcon,
};
use crate::services::ai::{AIError, AIService};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// AI 聊天弹窗 / AI Chat Modal
#[component]
pub fn AiChatModal() -> Element {
    let mut state = use_context::<AppState>();
    let show = *state.show_ai_chat.read();
    let lang = *state.language.read();

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
    let send_t = t("send", lang);

    let display_class = if show { "" } else { "hidden" };

    let ai_config = state.ai_config.read().clone();
    let ai_enabled = ai_config.enabled;
    let ai_loading = *state.ai_loading.read();
    let input_text = state.ai_input.read().clone();
    let content = state.content.read().clone();

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                AppActions::hide_ai_chat(&mut state);
            },

            div {
                class: "modal ai-chat-modal",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { "{ai_title_t}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            AppActions::hide_ai_chat(&mut state);
                        },
                        CloseIcon { size: 20 }
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

                        div { class: "ai-actions",
                            AiActionBtn {
                                label: continue_t.clone(),
                                icon: Some("continue".to_string()),
                                task_type: "continue",
                                config: ai_config.clone(),
                                content: content.clone(),
                                input: input_text.clone(),
                            }
                            AiActionBtn {
                                label: improve_t.clone(),
                                icon: Some("improve".to_string()),
                                task_type: "improve",
                                config: ai_config.clone(),
                                content: content.clone(),
                                input: input_text.clone(),
                            }
                            AiActionBtn {
                                label: outline_t.clone(),
                                icon: Some("outline".to_string()),
                                task_type: "outline",
                                config: ai_config.clone(),
                                content: content.clone(),
                                input: input_text.clone(),
                            }
                            AiActionBtn {
                                label: translate_t.clone(),
                                icon: Some("translate".to_string()),
                                task_type: "translate",
                                config: ai_config.clone(),
                                content: content.clone(),
                                input: input_text.clone(),
                            }
                            AiActionBtn {
                                label: grammar_t.clone(),
                                icon: Some("grammar".to_string()),
                                task_type: "fix_grammar",
                                config: ai_config.clone(),
                                content: content.clone(),
                                input: input_text.clone(),
                            }
                        }

                        div { class: "ai-input-section",
                            textarea {
                                class: "ai-input",
                                placeholder: "{placeholder_t}",
                                value: "{input_text}",
                                oninput: move |e| {
                                    *state.ai_input.write() = e.value();
                                },
                            }

                            div { class: "ai-input-actions",
                                button {
                                    class: "btn-secondary",
                                    onclick: move |_| {
                                        *state.ai_input.write() = String::new();
                                    },
                                    "{clear_t}"
                                }
                                AiActionBtn {
                                    label: send_t.clone(),
                                    task_type: "custom",
                                    config: ai_config.clone(),
                                    content: content.clone(),
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
    let mut state = use_context::<AppState>();
    let icon_type = props.icon.clone();
    let lang = *state.language.read();

    // Pre-compute i18n strings for async use
    let title_continue = t("ai_continue_result", lang);
    let title_improve = t("ai_improve_result", lang);
    let title_outline = t("ai_outline_result", lang);
    let title_translate = t("ai_translate_result", lang);
    let title_grammar = t("ai_grammar_result", lang);
    let title_default = t("ai_response", lang);
    let title_error = t("ai_error", lang);
    let error_prefix = t("error", lang);

    fn format_ai_error(error: &AIError, prefix: &str) -> String {
        match error {
            AIError::Config(msg)
            | AIError::Authentication(msg)
            | AIError::RateLimit(msg)
            | AIError::ServiceUnavailable(msg)
            | AIError::Timeout(msg)
            | AIError::Api(msg)
            | AIError::Parse(msg) => format!("{}: {}", prefix, msg),
            AIError::Network(err) => format!(
                "{}: 网络请求失败，请检查连接后重试 / Network request failed: {}",
                prefix, err
            ),
        }
    }

    rsx! {
        button {
            class: "ai-action-btn",
            onclick: move |_| {
                let task_type = props.task_type.clone();
                let api_key = props.config.api_key.clone();
                let base_url = props.config.base_url.clone();
                let model = props.config.model.clone();
                let content = props.content.clone();
                let input = props.input.clone();

                *state.ai_loading.write() = true;
                AppActions::hide_ai_chat(&mut state);

                let mut ai_loading = state.ai_loading;
                let mut ai_result = state.ai_result;
                let mut ai_title = state.ai_title;
                let mut show_ai_result_signal = state.show_ai_result;

                // Capture i18n strings for async
                let tc = title_continue.clone();
                let ti = title_improve.clone();
                let to = title_outline.clone();
                let tt = title_translate.clone();
                let tg = title_grammar.clone();
                let td = title_default.clone();
                let te = title_error.clone();
                let ep = error_prefix.clone();

                // 立即显示结果弹窗（流式更新）/ Show result modal immediately (streaming updates)
                *ai_result.write() = String::new();
                *ai_title.write() = match task_type.as_str() {
                    "continue" => tc,
                    "improve" => ti,
                    "outline" => to,
                    "translate" => tt,
                    "fix_grammar" => tg,
                    _ => td,
                };
                AppActions::show_ai_result(&mut state);
                *show_ai_result_signal.write() = true;

                spawn(async move {
                    let service = AIService::new(api_key, Some(base_url), Some(model));

                    // 构建消息 / Build messages
                    let messages = match task_type.as_str() {
                        "continue" => crate::services::ai::build_continue_messages(&content),
                        "improve" => crate::services::ai::build_improve_messages(&content),
                        "outline" => crate::services::ai::build_outline_messages(&input),
                        "translate" => crate::services::ai::build_translate_messages(&content, "English"),
                        "fix_grammar" => crate::services::ai::build_grammar_messages(&content),
                        "custom" => crate::services::ai::build_custom_messages(&input, &content),
                        _ => vec![],
                    };

                    // 使用流式响应 / Use streaming response
                    let result = service.chat_stream(messages, |chunk| {
                        // 实时追加内容到结果 / Append content to result in real-time
                        let current = ai_result.read().clone();
                        *ai_result.write() = format!("{}{}", current, chunk);
                    }).await;

                    *ai_loading.write() = false;

                    if let Err(e) = result {
                        let current_result = ai_result.read().clone();
                        if current_result.is_empty() {
                            *ai_result.write() = format_ai_error(&e, &ep);
                            *ai_title.write() = te;
                            *show_ai_result_signal.write() = true;
                        }
                        // 如果已有部分内容，保留已接收的部分 / If partial content exists, keep it
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
