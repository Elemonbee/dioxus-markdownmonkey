//! 全局搜索组件 / Global Search Component
//!
//! 在工作区所有 Markdown 文件中搜索

use crate::actions::shortcut_actions::ShortcutActions;
use crate::actions::FileActions;
use crate::components::icons::CloseIcon;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;
use std::path::{Path, PathBuf};

/// 搜索结果项 / Search Result Item
#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult {
    /// 文件路径 / File path
    pub path: PathBuf,
    /// 匹配的行号 / Matching line number
    pub line: usize,
    /// 匹配的行内容 / Matching line content
    pub content: String,
    /// 匹配位置 / Match position
    pub start: usize,
    pub end: usize,
}

/// 全局搜索弹窗 / Global Search Modal
#[component]
pub fn GlobalSearchModal() -> Element {
    let mut state = use_context::<AppState>();
    let show = *state.show_global_search.read();

    // 搜索状态 / Search state
    let mut search_input = use_signal(String::new);
    let mut results = use_signal(Vec::<SearchResult>::new);
    let mut searching = use_signal(|| false);
    let mut selected_index = use_signal(|| 0usize);

    let lang = *state.language.read();
    let global_search_t = t("global_search", lang);
    let search_workspace_t = t("search_workspace", lang);
    let searching_t = t("searching", lang);
    let search_t = t("search_placeholder", lang);
    let no_results_t = t("no_results", lang);
    let line_t = t("line", lang);
    let navigate_t = t("navigate_open", lang);
    let current_file_t = t("current_file", lang);
    let current_file_t_key = current_file_t.clone();

    let display_class = if show { "" } else { "hidden" };

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                *state.show_global_search.write() = false;
            },

            div {
                class: "modal global-search-modal",
                onclick: move |e| e.stop_propagation(),

                // 头部 / Header
                div { class: "modal-header",
                    h2 { "{global_search_t}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            *state.show_global_search.write() = false;
                        },
                        CloseIcon { size: 20 }
                    }
                }

                // 搜索框 / Search Input
                div { class: "search-input-container",
                    input {
                        class: "search-input",
                        r#type: "text",
                        placeholder: "{search_workspace_t}",
                        value: "{search_input}",
                        autofocus: true,
                        oninput: move |e| {
                            *search_input.write() = e.value();
                        },
                        onkeydown: move |e| {
                            let key = e.key().to_string();
                            match key.as_str() {
                                "Escape" => {
                                    *state.show_global_search.write() = false;
                                    e.prevent_default();
                                }
                                "Enter" => {
                                    let query = search_input.read().clone();
                                    if !query.is_empty() {
                                        *searching.write() = true;
                                        *selected_index.write() = 0;

                                        let workspace = state.workspace_root.read().clone();
                                        let cft = current_file_t_key.clone();
                                        let found = if let Some(root) = workspace {
                                            search_in_directory(&root, &query)
                                        } else {
                                            search_in_current_file(&state.content.read(), &query, &cft)
                                        };

                                        *results.write() = found;
                                        *searching.write() = false;
                                    }
                                    e.prevent_default();
                                }
                                "ArrowDown" => {
                                    let total = results.read().len();
                                    if total > 0 {
                                        let current = *selected_index.read();
                                        *selected_index.write() = (current + 1) % total;
                                    }
                                    e.prevent_default();
                                }
                                "ArrowUp" => {
                                    let total = results.read().len();
                                    if total > 0 {
                                        let current = *selected_index.read();
                                        *selected_index.write() = if current == 0 { total - 1 } else { current - 1 };
                                    }
                                    e.prevent_default();
                                }
                                _ => {
                                    if ShortcutActions::handle_event(&mut state, &e) {
                                        e.prevent_default();
                                    }
                                }
                            }
                        },
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let query = search_input.read().clone();
                            if !query.is_empty() {
                                *searching.write() = true;
                                *selected_index.write() = 0;

                                let workspace = state.workspace_root.read().clone();
                                let found = if let Some(root) = workspace {
                                    search_in_directory(&root, &query)
                                } else {
                                    search_in_current_file(&state.content.read(), &query, &current_file_t)
                                };

                                *results.write() = found;
                                *searching.write() = false;
                            }
                        },
                        disabled: *searching.read(),
                        if *searching.read() { "{searching_t}" } else { "{search_t}" }
                    }
                }

                // 搜索结果 / Search Results
                div { class: "search-results",
                    if *searching.read() {
                        div { class: "search-loading", "{searching_t}" }
                    } else if results.read().is_empty() && !search_input.read().is_empty() {
                        div { class: "search-empty", "{no_results_t}" }
                    } else {
                        for (idx, result) in results.read().iter().enumerate() {
                            {
                                let result = result.clone();
                                let is_selected = idx == *selected_index.read();
                                let item_class = if is_selected { "search-result-item selected" } else { "search-result-item" };

                                rsx! {
                                    div {
                                        class: "{item_class}",
                                        onclick: move |_| {
                                            let file_path = result.path.clone();
                                            let target_line = result.line;
                                            *state.show_global_search.write() = false;
                                            let _ = FileActions::open_file(&mut state, file_path.clone());
                                            let _ = dioxus::document::eval(&format!(
                                                "if(window._mm_scrollToLine) window._mm_scrollToLine({})",
                                                target_line
                                            ));
                                        },

                                        div { class: "result-file", "{result.path.display()}" }
                                        div { class: "result-line", "{line_t} {result.line + 1}" }
                                        div { class: "result-content", "{result.content}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // 底部提示 / Footer
                div { class: "modal-footer",
                    span { class: "search-hint", "{navigate_t}" }
                }
            }
        }
    }
}

/// 在目录中搜索 / Search in directory
fn search_in_directory(dir: &Path, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // 跳过隐藏文件和特殊目录 / Skip hidden files and special directories
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }

            if path.is_dir() {
                // 递归搜索子目录 / Recursively search subdirectories
                results.extend(search_in_directory(&path, query));
            } else if path.is_file() {
                // 检查文件扩展名 / Check file extension
                if let Some(ext) = path.extension() {
                    if ext == "md" || ext == "markdown" || ext == "txt" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            results.extend(search_in_content(&path, &content, &query_lower, true));
                        }
                    }
                }
            }
        }
    }

    // 按文件路径和行号排序 / Sort by file path and line number
    results.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));

    // 限制总结果数 / Limit total results
    results.truncate(100);
    results
}

/// 在当前文件内容中搜索 / Search in current file content
fn search_in_current_file(content: &str, query: &str, label: &str) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    search_in_content(&PathBuf::from(label), content, &query_lower, false)
}

/// 在内容中搜索 / Search in content
fn search_in_content(
    path: &Path,
    content: &str,
    query_lower: &str,
    case_insensitive: bool,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let path_buf = path.to_path_buf();

    for (line_idx, line) in content.lines().enumerate() {
        let search_in = if case_insensitive {
            line.to_lowercase()
        } else {
            line.to_string()
        };

        let mut start = 0;
        while let Some(pos) = search_in[start..].find(query_lower) {
            let abs_pos = start + pos;
            results.push(SearchResult {
                path: path_buf.clone(),
                line: line_idx,
                content: line.to_string(),
                start: abs_pos,
                end: abs_pos + query_lower.len(),
            });
            start = abs_pos + query_lower.len();
            if start >= search_in.len() {
                break;
            }
        }
    }

    // 限制每个文件最多返回 50 个结果 / Limit to 50 results per file
    results.truncate(50);
    results
}
