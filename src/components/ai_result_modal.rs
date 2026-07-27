//! AI 结果弹窗组件 / AI Result Modal Component

use crate::actions::{AppActions, EditorActions};
use crate::components::icons::CloseIcon;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// AI 结果弹窗 / AI Result Modal
#[component]
pub fn AiResultModal() -> Element {
    let mut state = use_context::<AppState>();
    let ai = state.ai();
    let ui = state.ui();
    let show = *ai.show_ai_result.read();
    let lang = *ui.language.read();
    let ai_loading = *ai.ai_loading.read();

    let close_t = t("close", lang);
    let copy_t = t("copy", lang);
    let append_t = t("append", lang);
    let replace_t = t("replace_doc", lang);
    let follow_up_t = t("ai_follow_up", lang);
    let clear_history_t = t("ai_clear_history", lang);
    let clear_confirm_t = t("ai_clear_history_confirm", lang);
    let history_turns_t = t("ai_history_turns", lang);
    let stop_t = t("ai_stop", lang);
    let generating_t = t("ai_generating", lang);
    let thinking_t = t("ai_thinking", lang);
    let history_turns = ai.ai_history.read().len() / 2;
    let session_hint = if history_turns > 0 {
        history_turns_t.replace("{n}", &history_turns.to_string())
    } else {
        String::new()
    };

    let display_class = if show { "" } else { "hidden" };

    let title = ai.ai_title.read().clone();
    let result = ai.ai_result.read().clone();

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                if !ai_loading {
                    AppActions::hide_ai_result(&mut state);
                }
            },

            div {
                class: "modal ai-result-modal",
                role: "dialog",
                "aria-modal": "true",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { "{title}" }
                    button {
                        class: "modal-close",
                        disabled: ai_loading,
                        onclick: move |_| {
                            AppActions::hide_ai_result(&mut state);
                        },
                        CloseIcon { size: 20 }
                    }
                }

                div { class: "modal-body",
                    if ai_loading && result.is_empty() {
                        div { class: "ai-result-loading", "aria-live": "polite", "{thinking_t}" }
                    }
                    div {
                        class: "ai-result-content",
                        "aria-live": "polite",
                        pre { "{result}" }
                    }
                    if ai_loading {
                        div { class: "ai-result-generating", "aria-live": "polite", "{generating_t}" }
                    }
                }

                div { class: "modal-footer",
                    if !session_hint.is_empty() {
                        span { class: "ai-session-hint", "{session_hint}" }
                    }
                    if ai_loading {
                        button {
                            class: "btn-secondary",
                            onclick: move |_| {
                                AppActions::cancel_ai_generation(&mut state);
                            },
                            "{stop_t}"
                        }
                    }
                    CopyButton { result: result.clone(), copy_text: copy_t.clone() }
                    AppendButton {
                        result: result.clone(),
                        append_text: append_t.clone(),
                        disabled: ai_loading,
                    }
                    ReplaceButton {
                        result: result.clone(),
                        replace_text: replace_t.clone(),
                        disabled: ai_loading,
                    }
                    if history_turns > 0 && !ai_loading {
                        button {
                            class: "btn-secondary",
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
                        class: "btn-secondary",
                        disabled: ai_loading,
                        onclick: move |_| {
                            AppActions::follow_up_ai_chat(&mut state);
                        },
                        "{follow_up_t}"
                    }
                    button {
                        class: "btn-primary",
                        disabled: ai_loading,
                        onclick: move |_| {
                            AppActions::hide_ai_result(&mut state);
                        },
                        "{close_t}"
                    }
                }
            }
        }
    }
}

/// 复制按钮组件属性 / Copy Button Props
#[derive(Props, Clone, PartialEq)]
struct CopyButtonProps {
    result: String,
    copy_text: String,
}

/// 复制按钮组件 / Copy Button Component
fn CopyButton(props: CopyButtonProps) -> Element {
    rsx! {
        button {
            class: "btn-secondary",
            onclick: move |_| {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(props.result.clone());
                }
            },
            "{props.copy_text}"
        }
    }
}

/// 追加按钮组件属性 / Append Button Props
#[derive(Props, Clone, PartialEq)]
struct AppendButtonProps {
    result: String,
    append_text: String,
    #[props(default = false)]
    disabled: bool,
}

/// 追加按钮（先 flush，避免非受控模式下丢编辑）
/// Append button (flush first so uncontrolled edits are not lost)
fn AppendButton(props: AppendButtonProps) -> Element {
    let state = use_context::<AppState>();
    let result = props.result.clone();
    let disabled = props.disabled;

    rsx! {
        button {
            class: "btn-secondary",
            disabled: disabled,
            onclick: move |_| {
                let result = result.clone();
                let mut state = state;
                spawn(async move {
                    EditorActions::flush_from_dom(&mut state).await;
                    let content = state.document().content.read().clone();
                    let new_content = format!("{}\n\n{}", content, result);
                    EditorActions::update_content(&mut state, new_content);
                    EditorActions::push_to_dom(&state.document().content.read());
                    AppActions::hide_ai_result(&mut state);
                });
            },
            "{props.append_text}"
        }
    }
}

/// 替换按钮组件属性 / Replace Button Props
#[derive(Props, Clone, PartialEq)]
struct ReplaceButtonProps {
    result: String,
    replace_text: String,
    #[props(default = false)]
    disabled: bool,
}

/// 替换按钮（先 flush，保证历史栈含最新正文）
/// Replace button (flush first so undo history includes latest body)
fn ReplaceButton(props: ReplaceButtonProps) -> Element {
    let state = use_context::<AppState>();
    let disabled = props.disabled;

    rsx! {
        button {
            class: "btn-secondary",
            disabled: disabled,
            onclick: move |_| {
                let result = props.result.clone();
                let mut state = state;
                spawn(async move {
                    EditorActions::flush_from_dom(&mut state).await;
                    EditorActions::update_content(&mut state, result);
                    EditorActions::push_to_dom(&state.document().content.read());
                    AppActions::hide_ai_result(&mut state);
                });
            },
            "{props.replace_text}"
        }
    }
}
