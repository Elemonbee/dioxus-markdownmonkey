//! AI 结果弹窗组件 / AI Result Modal Component

use crate::actions::AppActions;
use crate::components::icons::CloseIcon;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// AI 结果弹窗 / AI Result Modal
#[component]
pub fn AiResultModal() -> Element {
    let mut state = use_context::<AppState>();
    let show = *state.show_ai_result.read();
    let lang = *state.language.read();

    let close_t = t("close", lang);
    let copy_t = t("copy", lang);
    let append_t = t("append", lang);
    let replace_t = t("replace_doc", lang);

    let display_class = if show { "" } else { "hidden" };

    let title = state.ai_title.read().clone();
    let result = state.ai_result.read().clone();
    let content = state.content.read().clone();

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                AppActions::hide_ai_result(&mut state);
            },

            div {
                class: "modal ai-result-modal",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { "{title}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            AppActions::hide_ai_result(&mut state);
                        },
                        CloseIcon { size: 20 }
                    }
                }

                div { class: "modal-body",
                    div { class: "ai-result-content",
                        pre { "{result}" }
                    }
                }

                div { class: "modal-footer",
                    CopyButton { result: result.clone(), copy_text: copy_t.clone() }
                    AppendButton {
                        result: result.clone(),
                        content: content.clone(),
                        append_text: append_t.clone(),
                    }
                    ReplaceButton {
                        result: result.clone(),
                        replace_text: replace_t.clone(),
                    }
                    button {
                        class: "btn-primary",
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

/// 复制按钮 / Copy Button
fn CopyButton(props: CopyButtonProps) -> Element {
    let mut state = use_context::<AppState>();
    let result = props.result.clone();

    rsx! {
        button {
            class: "btn-secondary",
            onclick: move |_| {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Err(e) = clipboard.set_text(&result) {
                        tracing::error!("Failed to copy to clipboard: {}", e);
                    }
                }
                AppActions::hide_ai_result(&mut state);
            },
            "{props.copy_text}"
        }
    }
}

/// 追加按钮组件属性 / Append Button Props
#[derive(Props, Clone, PartialEq)]
struct AppendButtonProps {
    result: String,
    content: String,
    append_text: String,
}

/// 追加按钮 / Append Button
fn AppendButton(props: AppendButtonProps) -> Element {
    let mut state = use_context::<AppState>();
    let result = props.result.clone();
    let content = props.content.clone();

    rsx! {
        button {
            class: "btn-secondary",
            onclick: move |_| {
                let new_content = format!("{}\n\n{}", content, result);
                *state.content.write() = new_content;
                *state.modified.write() = true;
                AppActions::hide_ai_result(&mut state);
                state.update_outline();
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
}

/// 替换按钮 / Replace Button
fn ReplaceButton(props: ReplaceButtonProps) -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        button {
            class: "btn-secondary",
            onclick: move |_| {
                *state.content.write() = props.result.clone();
                *state.modified.write() = true;
                AppActions::hide_ai_result(&mut state);
                state.update_outline();
            },
            "{props.replace_text}"
        }
    }
}
