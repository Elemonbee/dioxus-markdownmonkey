//! 文件外部修改提示弹窗 / File External Modification Modal
//!
//! 当检测到文件被外部程序修改时显示此提示

use crate::actions::FileActions;
use crate::components::icons::{CloseIcon, RefreshIcon};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 文件修改提示弹窗 / File Modified Modal
#[component]

pub fn FileModifiedModal() -> Element {
    let mut state = use_context::<AppState>();
    let show = *state.file_external_modified.read();
    let lang = *state.language.read();

    let current_file = state.current_file.read().clone();
    let untitled_text = t("untitled", lang);
    let filename = current_file
        .as_ref()
        .map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&untitled_text)
                .to_string()
        })
        .unwrap_or_else(|| format!("{untitled_text}.md"));

    if !show {
        return rsx! {};
    }

    let title = t("file_modified", lang);
    let msg = t("file_modified_msg", lang);
    let ignore_text = t("ignore", lang);
    let reload_text = t("reload", lang);

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                state.refresh_file_watch();
                *state.file_external_modified.write() = false;
            },

            div {
                class: "modal file-modified-modal",
                onclick: move |e| e.stop_propagation(),

                // 头部
                div { class: "modal-header",
                    h2 { "{title}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            state.refresh_file_watch();
                            *state.file_external_modified.write() = false;
                        },
                        CloseIcon { size: 20 }
                    }
                }

                // 内容
                div { class: "modal-body",
                    p {
                        "{msg}"
                    }
                    p {
                        strong { "{filename}" }
                    }
                }

                // 按钮
                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            state.refresh_file_watch();
                            *state.file_external_modified.write() = false;
                        },
                        "{ignore_text}"
                    }
                    button {
                        class: "btn-primary",
                        onclick: {
                            let current_file = current_file.clone();
                            move |_| {
                                if let Some(path) = &current_file {
                                    if let Ok(content) = FileActions::read_file_with_encoding(path) {
                                        *state.content.write() = content;
                                        *state.modified.write() = false;
                                        let mut history = state.history.write();
                                        history.past.clear();
                                        history.future.clear();
                                        drop(history);
                                        state.update_outline();
                                        state.refresh_file_watch();
                                    } else {
                                        tracing::error!(
                                            "Failed to reload externally modified file: {:?}",
                                            path
                                        );
                                    }
                                }
                                *state.file_external_modified.write() = false;
                            }
                        },
                        RefreshIcon { size: 16 }
                        " {reload_text}"
                    }
                }
            }
        }
    }
}
