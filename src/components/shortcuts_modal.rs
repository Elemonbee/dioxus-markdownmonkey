//! 快捷键弹窗组件 / Keyboard Shortcuts Modal Component

use crate::actions::AppActions;
use crate::components::icons::CloseIcon;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 快捷键弹窗 / Keyboard Shortcuts Modal
#[component]
pub fn ShortcutsModal() -> Element {
    let mut state = use_context::<AppState>();
    let show = *state.show_shortcuts.read();
    let lang = *state.language.read();

    // i18n
    let shortcuts_t = t("shortcuts", lang);
    let close_t = t("close", lang);
    let new_file_t = t("new_file", lang);
    let open_file_t = t("open_file", lang);
    let save_file_t = t("save_file", lang);
    let undo_t = t("undo", lang);
    let redo_t = t("redo", lang);
    let bold_t = t("bold", lang);
    let italic_t = t("italic", lang);
    let code_t = t("code", lang);
    let insert_link_t = t("insert_link", lang);
    let toggle_sidebar_t = t("toggle_sidebar", lang);
    let toggle_preview_t = t("toggle_preview", lang);
    let theme_t = t("theme", lang);
    let settings_t = t("settings", lang);
    let show_shortcuts_t = t("show_shortcuts", lang);
    let ai_t = t("ai_assistant", lang);
    let close_modal_t = t("close_modal", lang);

    // 始终渲染，但用 CSS 控制显示/隐藏
    let display_class = if show { "" } else { "hidden" };

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                AppActions::hide_shortcuts(&mut state);
            },

            div {
                class: "modal shortcuts-modal",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { "{shortcuts_t}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            AppActions::hide_shortcuts(&mut state);
                        },
                        CloseIcon { size: 20 }
                    }
                }

                div { class: "modal-body",
                    table { class: "shortcuts-table",
                        tbody {
                            tr { td { "Ctrl+N" } td { "{new_file_t}" } }
                            tr { td { "Ctrl+O" } td { "{open_file_t}" } }
                            tr { td { "Ctrl+S" } td { "{save_file_t}" } }
                            tr { td { "Ctrl+Z" } td { "{undo_t}" } }
                            tr { td { "Ctrl+Y" } td { "{redo_t}" } }
                            tr { td { "Ctrl+B" } td { "{bold_t}" } }
                            tr { td { "Ctrl+I" } td { "{italic_t}" } }
                            tr { td { "Ctrl+`" } td { "{code_t}" } }
                            tr { td { "Ctrl+K" } td { "{insert_link_t}" } }
                            tr { td { "Ctrl+\\" } td { "{toggle_sidebar_t}" } }
                            tr { td { "Ctrl+P" } td { "{toggle_preview_t}" } }
                            tr { td { "Ctrl+T" } td { "{theme_t}" } }
                            tr { td { "Ctrl+," } td { "{settings_t}" } }
                            tr { td { "Ctrl+/" } td { "{show_shortcuts_t}" } }
                            tr { td { "Ctrl+J" } td { "{ai_t}" } }
                            tr { td { "Escape" } td { "{close_modal_t}" } }
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            AppActions::hide_shortcuts(&mut state);
                        },
                        "{close_t}"
                    }
                }
            }
        }
    }
}
