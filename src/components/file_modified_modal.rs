//! 文件外部修改提示弹窗 / File External Modification Modal
//!
//! 当检测到文件被外部程序修改时显示此提示

use crate::actions::{AppActions, FileActions};
use crate::components::icons::{CloseIcon, RefreshIcon};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 文件修改提示弹窗 / File Modified Modal
#[component]
pub fn FileModifiedModal() -> Element {
    let mut state = use_context::<AppState>();
    let doc = state.document();
    let ui = state.ui();
    let show = *doc.file_external_modified.read();
    let lang = *ui.language.read();

    let current_file = doc.current_file.read().clone();
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
                AppActions::dismiss_file_external_modified(&mut state);
            },

            div {
                class: "modal file-modified-modal",
                role: "dialog",
                "aria-modal": "true",
                onclick: move |e| e.stop_propagation(),

                // 头部
                div { class: "modal-header",
                    h2 { "{title}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            AppActions::dismiss_file_external_modified(&mut state);
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
                            AppActions::dismiss_file_external_modified(&mut state);
                        },
                        "{ignore_text}"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let mut state = state;
                            spawn(async move {
                                if let Err(e) =
                                    FileActions::reload_current_file_flushed(&mut state).await
                                {
                                    tracing::error!(
                                        "Failed to reload externally modified file: {}",
                                        e
                                    );
                                    AppActions::dismiss_file_external_modified(&mut state);
                                }
                            });
                        },
                        RefreshIcon { size: 16 }
                        " {reload_text}"
                    }
                }
            }
        }
    }
}
