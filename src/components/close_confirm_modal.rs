//! 关闭未保存标签确认弹窗 / Close Unsaved Tab Confirmation Modal
//!
//! 当关闭已修改但未保存的标签时显示确认提示，防止数据丢失
//! Prevents data loss when closing modified but unsaved tabs

use crate::actions::FileActions;
use crate::components::icons::{CloseIcon, WarningIcon};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 关闭未保存标签确认弹窗 / Close Unsaved Tab Confirmation Modal
#[component]
pub fn CloseConfirmModal() -> Element {
    let state = use_context::<AppState>();
    let mut doc = state.document();
    let ui = state.ui();
    let show = *doc.show_close_confirm.read();
    let lang = *ui.language.read();

    if !show {
        return rsx! {};
    }

    let pending_index = *doc.pending_close_tab_index.read();
    let tab_title = pending_index
        .and_then(|idx| {
            let tabs = doc.tabs.read();
            tabs.get(idx).map(|t| t.title.clone())
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
                *doc.show_close_confirm.write() = false;
                *doc.pending_close_tab_index.write() = None;
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
                            *doc.show_close_confirm.write() = false;
                            *doc.pending_close_tab_index.write() = None;
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
                            *doc.show_close_confirm.write() = false;
                            *doc.pending_close_tab_index.write() = None;
                        },
                        "{cancel}"
                    }
                    button {
                        class: "btn-secondary",
                        style: "background: #e74c3e; border-color: #e74c3e; color: #fff;",
                        onclick: move |_| {
                            let index = *doc.pending_close_tab_index.read();
                            if let Some(idx) = index {
                                let mut state = state;
                                spawn(async move {
                                    FileActions::discard_and_close_tab_flushed(&mut state, idx).await;
                                });
                            } else {
                                *doc.show_close_confirm.write() = false;
                                *doc.pending_close_tab_index.write() = None;
                            }
                        },
                        "{dont_save}"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let index = *doc.pending_close_tab_index.read();
                            if let Some(idx) = index {
                                let mut state = state;
                                spawn(async move {
                                    FileActions::save_and_close_tab_flushed(&mut state, idx).await;
                                });
                            } else {
                                *doc.show_close_confirm.write() = false;
                                *doc.pending_close_tab_index.write() = None;
                            }
                        },
                        "{save_close}"
                    }
                }
            }
        }
    }
}
