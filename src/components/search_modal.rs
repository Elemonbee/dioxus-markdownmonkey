//! 搜索替换弹窗组件 / Search and Replace Modal Component

use crate::actions::{EditorActions, SearchActions};
use crate::components::icons::{CloseIcon, SearchIcon};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 搜索替换弹窗 / Search and Replace Modal
#[component]
pub fn SearchModal() -> Element {
    let mut state = use_context::<AppState>();
    let ui = state.ui();
    let show = *ui.show_search.read();
    let lang = *ui.language.read();

    let display_class = if show { "" } else { "hidden" };

    // 打开时 flush，保证非受控模式下匹配计数准确
    // Flush on open so match counts are accurate in uncontrolled mode
    let mut state_for_flush = state;
    use_effect(move || {
        if show {
            spawn(async move {
                EditorActions::flush_from_dom(&mut state_for_flush).await;
            });
        }
    });

    let search_query = ui.search_query.read().clone();
    let replace_query = ui.replace_query.read().clone();
    let search_index = *ui.search_index.read();
    let search_total = *ui.search_total.read();
    let case_insensitive = *ui.search_case_insensitive.read();
    let use_regex = *ui.search_regex.read();

    let find_text = t("find", lang);
    let replace_text = t("replace", lang);
    let case_sensitive_text = t("case_sensitive", lang);
    let regex_text = t("regex", lang);
    let prev_text = t("previous", lang);
    let next_text = t("next", lang);
    let replace_btn_text = t("replace_btn", lang);
    let replace_all_text = t("replace_all", lang);
    let no_results_text = t("no_results", lang);

    rsx! {
        div {
            class: "search-modal-bar {display_class}",

            // 搜索输入行
            div { class: "search-row",
                div { class: "search-input-wrapper",
                    span { class: "search-icon", SearchIcon { size: 12 } }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "{find_text}...",
                        value: "{search_query}",
                        oninput: move |e| {
                            SearchActions::set_query(&mut state, e.value());
                        },
                    }
                    span { class: "search-count",
                        if search_total > 0 {
                            "{search_index}/{search_total}"
                        } else {
                            "{no_results_text}"
                        }
                    }
                }

                // 搜索选项
                div { class: "search-options",
                    button {
                        class: if case_insensitive { "search-option-btn active" } else { "search-option-btn" },
                        title: "{case_sensitive_text}",
                        onclick: move |_| {
                            SearchActions::toggle_case_insensitive(&mut state);
                        },
                        "Aa"
                    }
                }

                // 导航按钮
                div { class: "search-nav",
                    button {
                        class: "search-nav-btn",
                        title: "{prev_text}",
                        onclick: move |_| {
                            SearchActions::prev_match(&mut state);
                        },
                        "↑"
                    }
                    button {
                        class: "search-nav-btn",
                        title: "{next_text}",
                        onclick: move |_| {
                            SearchActions::next_match(&mut state);
                        },
                        "↓"
                    }
                }

                    // Regex toggle
                    button {
                        class: if use_regex { "search-option-btn active" } else { "search-option-btn" },
                        title: "{regex_text}",
                        onclick: move |_| {
                            SearchActions::toggle_regex(&mut state);
                        },
                        ".*"
                    }

                    button {
                        class: "search-close-btn",
                    onclick: move |_| {
                        SearchActions::hide(&mut state);
                    },
                    CloseIcon { size: 14 }
                }
            }

            // 替换输入行
            div { class: "search-row",
                div { class: "search-input-wrapper",
                    input {
                        r#type: "text",
                        class: "search-input replace-input",
                        placeholder: "{replace_text}...",
                        value: "{replace_query}",
                        oninput: move |e| {
                            SearchActions::set_replace_query(&mut state, e.value());
                        },
                    }
                }

                div { class: "replace-actions",
                    button {
                        class: "replace-btn",
                        onclick: move |_| {
                            let mut state = state;
                            spawn(async move {
                                SearchActions::replace_current(&mut state).await;
                            });
                        },
                        "{replace_btn_text}"
                    }
                    button {
                        class: "replace-btn",
                        onclick: move |_| {
                            let mut state = state;
                            spawn(async move {
                                SearchActions::replace_all(&mut state).await;
                            });
                        },
                        "{replace_all_text}"
                    }
                }
            }
        }
    }
}
