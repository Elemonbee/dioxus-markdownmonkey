//! 大文件警告弹窗 / Large File Warning Modal
//!
//! 当打开的文件超过阈值时显示性能警告
//! 文件内容不会在显示警告前读入内存，而是在用户确认后才加载
//! File content is NOT loaded into memory before showing the warning; it loads after user confirmation

use crate::actions::FileActions;
use crate::components::icons::{CloseIcon, WarningIcon};
use crate::config::LARGE_FILE_THRESHOLD_BYTES;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

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

/// 大文件警告弹窗 / Large File Warning Modal
#[component]
pub fn LargeFileWarningModal() -> Element {
    let mut state = use_context::<AppState>();
    let show = *state.show_large_file_warning.read();
    let file_size = *state.file_size_bytes.read();
    let lang = *state.language.read();

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
                *state.show_large_file_warning.write() = false;
            },

            div {
                class: "modal large-file-modal",
                onclick: move |e| e.stop_propagation(),

                // 头部
                div { class: "modal-header",
                    h2 { WarningIcon { size: 20, color: "#f0ad4e".to_string() } " {title}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            *state.show_large_file_warning.write() = false;
                        },
                        CloseIcon { size: 20 }
                    }
                }

                // 内容
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

                // 按钮
                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            // 取消加载 - 清除待加载状态 / Cancel loading - clear pending state
                            FileActions::cancel_load_large_file(&mut state);
                        },
                        "{cancel_text}"
                    }
                    button {
                        class: "btn-primary",
                        style: "background: #f0ad4e; border-color: #f0ad4e;",
                        onclick: move |_| {
                            // 确认加载 - 读取文件内容 / Confirm loading - read file content
                            if let Err(e) = FileActions::confirm_load_large_file(&mut state) {
                                tracing::error!("加载大文件失败 / Failed to load large file: {}", e);
                            }
                        },
                        "{continue_text}"
                    }
                }
            }
        }
    }
}
