//! 搜索替换弹窗组件 / Search and Replace Modal Component

use crate::actions::EditorActions;
use crate::components::icons::{CloseIcon, SearchIcon};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;
use regex::{Regex, RegexBuilder};

/// 构建正则表达式 / Build regex from query
fn build_regex(query: &str, case_insensitive: bool) -> Result<Regex, regex::Error> {
    let mut builder = RegexBuilder::new(query);
    builder.case_insensitive(case_insensitive);
    builder.build()
}

/// 统计匹配数量 / Count matches
fn count_matches(content: &str, query: &str, case_insensitive: bool, use_regex: bool) -> usize {
    if use_regex {
        if let Ok(re) = build_regex(query, case_insensitive) {
            re.find_iter(content).count()
        } else {
            0
        }
    } else {
        let (search_content, search_query) = if case_insensitive {
            (content.to_lowercase(), query.to_lowercase())
        } else {
            (content.to_string(), query.to_string())
        };
        search_content.matches(&search_query).count()
    }
}

/// 替换第 n 个匹配 / Replace the n-th match (0-indexed)
fn replace_nth_match(
    content: &str,
    query: &str,
    replacement: &str,
    index: usize,
    case_insensitive: bool,
    use_regex: bool,
) -> String {
    if index == 0 || content.is_empty() || query.is_empty() {
        return content.to_string();
    }
    if use_regex {
        if let Ok(re) = build_regex(query, case_insensitive) {
            let mut result = String::with_capacity(content.len());
            let mut last_end = 0;
            for (i, m) in re.find_iter(content).enumerate() {
                if i + 1 == index {
                    result.push_str(&content[last_end..m.start()]);
                    result.push_str(replacement);
                    last_end = m.end();
                    break;
                }
            }
            result.push_str(&content[last_end..]);
            result
        } else {
            content.to_string()
        }
    } else {
        let (search_content, search_query) = if case_insensitive {
            (content.to_lowercase(), query.to_lowercase())
        } else {
            (content.to_string(), query.to_string())
        };
        let mut count = 0;
        let mut pos = 0;
        while let Some(idx) = search_content[pos..].find(&search_query) {
            count += 1;
            if count == index {
                let abs_idx = pos + idx;
                return format!(
                    "{}{}{}",
                    &content[..abs_idx],
                    replacement,
                    &content[abs_idx + query.len()..]
                );
            }
            pos += idx + 1;
        }
        content.to_string()
    }
}

/// 大小写不敏感的全部字面替换 / Case-insensitive replace-all for literal queries
fn replace_all_case_insensitive(content: &str, query: &str, replacement: &str) -> String {
    if query.is_empty() {
        return content.to_string();
    }
    let lower_content = content.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut result = String::with_capacity(content.len());
    let mut pos = 0;
    while let Some(idx) = lower_content[pos..].find(&lower_query) {
        let abs = pos + idx;
        result.push_str(&content[pos..abs]);
        result.push_str(replacement);
        pos = abs + query.len();
    }
    result.push_str(&content[pos..]);
    result
}

/// 搜索替换弹窗 / Search and Replace Modal
#[component]
pub fn SearchModal() -> Element {
    let state = use_context::<AppState>();
    let mut ui = state.ui();
    let doc = state.document();
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
                            *ui.search_query.write() = e.value();
                            // 执行搜索
                            let content = doc.content.read().clone();
                            let query = e.value();
                            if query.is_empty() {
                                *ui.search_total.write() = 0;
                                *ui.search_index.write() = 0;
                            } else {
                                let count = count_matches(&content, &query, *ui.search_case_insensitive.read(), *ui.search_regex.read());
                                *ui.search_total.write() = count;
                                *ui.search_index.write() = if count > 0 { 1 } else { 0 };
                            }
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
                            let new_value = !*ui.search_case_insensitive.read();
                            *ui.search_case_insensitive.write() = new_value;
                            // 重新搜索
                            let content = doc.content.read().clone();
                            let query = ui.search_query.read().clone();
                            if !query.is_empty() {
                                let count = count_matches(&content, &query, *ui.search_case_insensitive.read(), *ui.search_regex.read());
                                *ui.search_total.write() = count;
                                *ui.search_index.write() = if count > 0 { 1 } else { 0 };
                            }
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
                            let total = *ui.search_total.read();
                            let current = *ui.search_index.read();
                            if total > 0 {
                                let new_index = if current <= 1 { total } else { current - 1 };
                                *ui.search_index.write() = new_index;
                            }
                        },
                        "↑"
                    }
                    button {
                        class: "search-nav-btn",
                        title: "{next_text}",
                        onclick: move |_| {
                            let total = *ui.search_total.read();
                            let current = *ui.search_index.read();
                            if total > 0 {
                                let new_index = if current >= total { 1 } else { current + 1 };
                                *ui.search_index.write() = new_index;
                            }
                        },
                        "↓"
                    }
                }

                    // Regex toggle
                    button {
                        class: if *ui.search_regex.read() { "search-option-btn active" } else { "search-option-btn" },
                        title: "{regex_text}",
                        onclick: move |_| {
                            let new_value = !*ui.search_regex.read();
                            *ui.search_regex.write() = new_value;
                            // Re-search
                            let content = doc.content.read().clone();
                            let query = ui.search_query.read().clone();
                            if !query.is_empty() {
                                let count = count_matches(&content, &query, *ui.search_case_insensitive.read(), *ui.search_regex.read());
                                *ui.search_total.write() = count;
                                *ui.search_index.write() = if count > 0 { 1 } else { 0 };
                            }
                        },
                        ".*"
                    }

                    button {
                        class: "search-close-btn",
                    onclick: move |_| {
                        *ui.show_search.write() = false;
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
                            *ui.replace_query.write() = e.value();
                        },
                    }
                }

                div { class: "replace-actions",
                    button {
                        class: "replace-btn",
                        onclick: move |_| {
                            let mut state = state;
                            let mut ui = ui;
                            let doc = doc;
                            spawn(async move {
                                // 替换前 flush，避免非受控模式下写到过期内容
                                // Flush before replace so uncontrolled mode is not stale
                                EditorActions::flush_from_dom(&mut state).await;
                                let content = doc.content.read().clone();
                                let query = ui.search_query.read().clone();
                                let replacement = ui.replace_query.read().clone();
                                let idx = *ui.search_index.read();
                                let case_insensitive = *ui.search_case_insensitive.read();
                                let use_regex = *ui.search_regex.read();

                                if !query.is_empty() && idx > 0 {
                                    let new_content = replace_nth_match(
                                        &content,
                                        &query,
                                        &replacement,
                                        idx,
                                        case_insensitive,
                                        use_regex,
                                    );
                                    state.update_content(new_content);
                                    EditorActions::push_to_dom(&doc.content.read());
                                    let updated_content = doc.content.read().clone();
                                    if !query.is_empty() {
                                        let count = count_matches(
                                            &updated_content,
                                            &query,
                                            case_insensitive,
                                            use_regex,
                                        );
                                        *ui.search_total.write() = count;
                                        *ui.search_index.write() = if count > 0 { 1 } else { 0 };
                                    }
                                }
                            });
                        },
                        "{replace_btn_text}"
                    }
                    button {
                        class: "replace-btn",
                        onclick: move |_| {
                            let mut state = state;
                            let mut ui = ui;
                            let doc = doc;
                            spawn(async move {
                                EditorActions::flush_from_dom(&mut state).await;
                                let content = doc.content.read().clone();
                                let query = ui.search_query.read().clone();
                                let replacement = ui.replace_query.read().clone();
                                let case_insensitive = *ui.search_case_insensitive.read();
                                let use_regex = *ui.search_regex.read();

                                if !query.is_empty() {
                                    let new_content = if use_regex {
                                        if let Ok(re) = build_regex(&query, case_insensitive) {
                                            re.replace_all(&content, replacement.as_str())
                                                .into_owned()
                                        } else {
                                            content
                                        }
                                    } else if case_insensitive {
                                        // 大小写不敏感的字面替换 / Case-insensitive literal replace
                                        replace_all_case_insensitive(&content, &query, &replacement)
                                    } else {
                                        content.replace(&query, &replacement)
                                    };
                                    state.update_content(new_content);
                                    EditorActions::push_to_dom(&doc.content.read());
                                    let updated_content = doc.content.read().clone();
                                    let count = count_matches(
                                        &updated_content,
                                        &query,
                                        case_insensitive,
                                        use_regex,
                                    );
                                    *ui.search_total.write() = count;
                                    *ui.search_index.write() = if count > 0 { 1 } else { 0 };
                                }
                            });
                        },
                        "{replace_all_text}"
                    }
                }
            }
        }
    }
}
