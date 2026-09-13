//! 快捷键弹窗组件 / Keyboard Shortcuts Modal Component

use crate::actions::shortcut_actions::ShortcutActions;
use crate::actions::AppActions;
use crate::components::icons::CloseIcon;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 快捷键弹窗 / Keyboard Shortcuts Modal
#[component]
pub fn ShortcutsModal() -> Element {
    let mut state = use_context::<AppState>();
    let ui = state.ui();
    let show = *ui.show_shortcuts.read();
    let lang = *ui.language.read();

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
    let search_t = t("shortcut_search", lang);
    let global_search_t = t("shortcut_global_search", lang);
    let print_t = t("shortcut_print", lang);

    // 始终渲染，但用 CSS 控制显示/隐藏
    let display_class = if show { "" } else { "hidden" };
    let modifier = ShortcutActions::primary_modifier_label();

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                AppActions::hide_shortcuts(&mut state);
            },

            div {
                class: "modal shortcuts-modal",
                role: "dialog",
                "aria-modal": "true",
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
                            tr { td { "{modifier}+N" } td { "{new_file_t}" } }
                            tr { td { "{modifier}+O" } td { "{open_file_t}" } }
                            tr { td { "{modifier}+S" } td { "{save_file_t}" } }
                            tr { td { "{modifier}+Z" } td { "{undo_t}" } }
                            tr { td { "{modifier}+Y / {modifier}+Shift+Z" } td { "{redo_t}" } }
                            tr { td { "{modifier}+B" } td { "{bold_t}" } }
                            tr { td { "{modifier}+I" } td { "{italic_t}" } }
                            tr { td { "{modifier}+`" } td { "{code_t}" } }
                            tr { td { "{modifier}+K" } td { "{insert_link_t}" } }
                            tr { td { "{modifier}+F" } td { "{search_t}" } }
                            tr { td { "{modifier}+Shift+F" } td { "{global_search_t}" } }
                            tr { td { "{modifier}+Shift+P" } td { "{print_t}" } }
                            tr { td { "{modifier}+\\" } td { "{toggle_sidebar_t}" } }
                            tr { td { "{modifier}+P" } td { "{toggle_preview_t}" } }
                            tr { td { "{modifier}+T" } td { "{theme_t}" } }
                            tr { td { "{modifier}+," } td { "{settings_t}" } }
                            tr { td { "{modifier}+/" } td { "{show_shortcuts_t}" } }
                            tr { td { "{modifier}+J" } td { "{ai_t}" } }
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
