//! 确认类弹窗：关闭未保存、外部修改、大文件警告
//! Confirmation dialogs: unsaved close, external modify, large-file warning

use crate::actions::{AppActions, FileActions};
use crate::components::icons::{CloseIcon, RefreshIcon, WarningIcon};
use crate::config::LARGE_FILE_THRESHOLD_BYTES;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 渲染当前需要的确认弹窗 / Render whichever confirmation dialog is active
#[component]
pub fn ConfirmModals() -> Element {
    rsx! {
        CloseConfirmModal {}
        FileModifiedModal {}
        LargeFileWarningModal {}
    }
}

/// 格式化文件大小 / Format file size
fn format_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// 关闭未保存标签确认弹窗 / Close unsaved tab confirmation
#[component]
fn CloseConfirmModal() -> Element {
    let state = use_context::<AppState>();
    let doc = state.document();
    let ui = state.ui();
    let show = *doc.show_close_confirm.read();
    let lang = *ui.language.read();

    if !show {
        return rsx! {};
    }

    let pending_id = *doc.pending_close_tab_id.read();
    let tab_title = pending_id
        .and_then(|tab_id| {
            doc.tabs
                .read()
                .iter()
                .find(|tab| tab.id == tab_id)
                .map(|tab| tab.title.clone())
        })
        .unwrap_or_else(|| t("untitled", lang));

    let title = t("close_confirm_title", lang);
    let msg = t("close_confirm_msg", lang);
    let save_close = t("save_close", lang);
    let dont_save = t("dont_save", lang);
    let cancel = t("cancel", lang);

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                let mut state = state;
                FileActions::cancel_close_request(&mut state);
            },

            div {
                class: "modal close-confirm-modal",
                role: "dialog",
                "aria-modal": "true",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { WarningIcon { size: 20, color: "#f0ad4e".to_string() } " {title}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            let mut state = state;
                            FileActions::cancel_close_request(&mut state);
                        },
                        CloseIcon { size: 20 }
                    }
                }

                div { class: "modal-body",
                    p { "{msg}" }
                    p {
                        style: "font-weight: bold; margin: 8px 0;",
                        "{tab_title}"
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            let mut state = state;
                            FileActions::cancel_close_request(&mut state);
                        },
                        "{cancel}"
                    }
                    button {
                        class: "btn-secondary",
                        style: "background: #e74c3e; border-color: #e74c3e; color: #fff;",
                        onclick: move |_| {
                            let tab_id = *doc.pending_close_tab_id.read();
                            if let Some(tab_id) = tab_id {
                                let mut state = state;
                                spawn(async move {
                                    FileActions::discard_and_close_tab_flushed(&mut state, tab_id).await;
                                });
                            } else {
                                let mut state = state;
                                FileActions::cancel_close_request(&mut state);
                            }
                        },
                        "{dont_save}"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let tab_id = *doc.pending_close_tab_id.read();
                            if let Some(tab_id) = tab_id {
                                let mut state = state;
                                spawn(async move {
                                    FileActions::save_and_close_tab_flushed(&mut state, tab_id).await;
                                });
                            } else {
                                let mut state = state;
                                FileActions::cancel_close_request(&mut state);
                            }
                        },
                        "{save_close}"
                    }
                }
            }
        }
    }
}

/// 文件外部修改提示弹窗 / File externally modified prompt
#[component]
fn FileModifiedModal() -> Element {
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

                div { class: "modal-body",
                    p { "{msg}" }
                    p {
                        strong { "{filename}" }
                    }
                }

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

/// 大文件警告弹窗 / Large file warning
#[component]
fn LargeFileWarningModal() -> Element {
    let mut state = use_context::<AppState>();
    let doc = state.document();
    let ui = state.ui();
    let show = *doc.show_large_file_warning.read();
    let file_size = *doc.file_size_bytes.read();
    let lang = *ui.language.read();

    if !show {
        return rsx! {};
    }

    let size_str = format_size(file_size.max(LARGE_FILE_THRESHOLD_BYTES));
    let title = t("large_file_warning", lang);
    let msg1 = t("large_file_msg1", lang);
    let msg2 = t("large_file_msg2", lang);
    let continue_text = t("continue_edit", lang);
    let cancel_text = t("cancel", lang);

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| {
                FileActions::cancel_load_large_file(&mut state);
            },

            div {
                class: "modal large-file-modal",
                role: "dialog",
                "aria-modal": "true",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { WarningIcon { size: 20, color: "#f0ad4e".to_string() } " {title}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            FileActions::cancel_load_large_file(&mut state);
                        },
                        CloseIcon { size: 20 }
                    }
                }

                div { class: "modal-body",
                    p { "{msg1}" }
                    p {
                        style: "font-size: 1.2em; font-weight: bold; color: #f0ad4e; margin: 12px 0;",
                        "{size_str}"
                    }
                    p {
                        style: "color: #888; font-size: 0.9em;",
                        "{msg2}"
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            FileActions::cancel_load_large_file(&mut state);
                        },
                        "{cancel_text}"
                    }
                    button {
                        class: "btn-primary",
                        style: "background: #f0ad4e; border-color: #f0ad4e;",
                        onclick: move |_| {
                            let mut state = state;
                            spawn(async move {
                                if let Err(e) =
                                    FileActions::confirm_load_large_file_flushed(&mut state).await
                                {
                                    tracing::error!(
                                        "加载大文件失败 / Failed to load large file: {}",
                                        e
                                    );
                                }
                            });
                        },
                        "{continue_text}"
                    }
                }
            }
        }
    }
}
