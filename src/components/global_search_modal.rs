//! 全局搜索组件 / Global Search Component
//!
//! 在工作区所有 Markdown 文件中搜索；优先使用已打开标签的内存内容
//! Search workspace Markdown files; prefer open-tab buffers over disk

use crate::actions::shortcut_actions::ShortcutActions;
use crate::actions::{EditorActions, FileActions};
use crate::components::icons::CloseIcon;
use crate::state::AppState;
use crate::utils::i18n::t;
use crate::utils::workspace_search::{
    collect_open_buffer_overrides, collect_workspace_files, preview_workspace_replace_counts,
    search_in_content, search_in_directory, WorkspaceSearchHit,
};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// 搜索结果项（UI）/ Search result item (UI)
#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult {
    pub path: PathBuf,
    pub line: usize,
    pub content: String,
    pub start: usize,
    pub end: usize,
}

impl From<WorkspaceSearchHit> for SearchResult {
    fn from(h: WorkspaceSearchHit) -> Self {
        Self {
            path: h.path,
            line: h.line,
            content: h.content,
            start: h.start,
            end: h.end,
        }
    }
}

/// 在后台线程启动工作区搜索 / Start workspace search on a background thread
fn spawn_workspace_search(
    query: String,
    workspace: Option<PathBuf>,
    current_content: String,
    current_file_label: String,
    open_overrides: HashMap<PathBuf, String>,
    mut results: Signal<Vec<SearchResult>>,
    mut searching: Signal<bool>,
) {
    if query.is_empty() {
        return;
    }
    *searching.write() = true;

    spawn(async move {
        let found = match tokio::task::spawn_blocking(move || {
            if let Some(root) = workspace {
                search_in_directory(&root, &query, &open_overrides)
                    .into_iter()
                    .map(SearchResult::from)
                    .collect()
            } else {
                let query_lower = query.to_lowercase();
                search_in_content(
                    &PathBuf::from(current_file_label),
                    &current_content,
                    &query_lower,
                    false,
                )
                .into_iter()
                .map(SearchResult::from)
                .collect::<Vec<_>>()
            }
        })
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("全局搜索任务失败 / Global search task failed: {}", e);
                Vec::new()
            }
        };

        *results.write() = found;
        *searching.write() = false;
    });
}

/// 全局搜索弹窗 / Global Search Modal
#[component]
pub fn GlobalSearchModal() -> Element {
    let mut state = use_context::<AppState>();
    let mut ui = state.ui();
    let show = *ui.show_global_search.read();

    let mut search_input = use_signal(String::new);
    let mut replace_input = use_signal(String::new);
    let results = use_signal(Vec::<SearchResult>::new);
    let searching = use_signal(|| false);
    let mut replacing = use_signal(|| false);
    let mut replace_status = use_signal(String::new);
    let mut selected_index = use_signal(|| 0usize);

    let lang = *ui.language.read();
    let global_search_t = t("global_search", lang);
    let search_workspace_t = t("search_workspace", lang);
    let searching_t = t("searching", lang);
    let search_t = t("search_placeholder", lang);
    let no_results_t = t("no_results", lang);
    let line_t = t("line", lang);
    let navigate_t = t("navigate_open", lang);
    let current_file_t = t("current_file", lang);
    let replace_with_t = t("replace", lang);
    let replace_all_workspace_t = t("replace_all_workspace", lang);
    let replacing_t = t("replacing", lang);
    let replace_confirm_title_t = t("replace_workspace_confirm_title", lang);
    let replace_confirm_msg_t = t("replace_workspace_confirm_msg", lang);
    let current_file_t_enter = current_file_t.clone();
    let current_file_t_click = current_file_t;

    let display_class = if show { "" } else { "hidden" };

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                *ui.show_global_search.write() = false;
            },

            div {
                class: "modal global-search-modal",
                role: "dialog",
                "aria-modal": "true",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { "{global_search_t}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            *ui.show_global_search.write() = false;
                        },
                        CloseIcon { size: 20 }
                    }
                }

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
                                    *ui.show_global_search.write() = false;
                                    e.prevent_default();
                                }
                                "Enter" => {
                                    *selected_index.write() = 0;
                                    let query = search_input.read().clone();
                                    let workspace = ui.workspace_root.read().clone();
                                    let label = current_file_t_enter.clone();
                                    let mut state = state;
                                    spawn(async move {
                                        EditorActions::flush_from_dom(&mut state).await;
                                        let overrides = collect_open_buffer_overrides(&state);
                                        spawn_workspace_search(
                                            query,
                                            workspace,
                                            state.document().content.read().clone(),
                                            label,
                                            overrides,
                                            results,
                                            searching,
                                        );
                                    });
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
                            *selected_index.write() = 0;
                            let query = search_input.read().clone();
                            let workspace = ui.workspace_root.read().clone();
                            let label = current_file_t_click.clone();
                            let mut state = state;
                            spawn(async move {
                                EditorActions::flush_from_dom(&mut state).await;
                                let overrides = collect_open_buffer_overrides(&state);
                                spawn_workspace_search(
                                    query,
                                    workspace,
                                    state.document().content.read().clone(),
                                    label,
                                    overrides,
                                    results,
                                    searching,
                                );
                            });
                        },
                        disabled: *searching.read(),
                        if *searching.read() { "{searching_t}" } else { "{search_t}" }
                    }
                }

                div { class: "search-input-container replace-row",
                    input {
                        class: "search-input",
                        r#type: "text",
                        placeholder: "{replace_with_t}",
                        value: "{replace_input}",
                        oninput: move |e| {
                            *replace_input.write() = e.value();
                        },
                    }
                    button {
                        class: "btn-secondary",
                        disabled: *searching.read() || *replacing.read() || search_input.read().is_empty(),
                        onclick: move |_| {
                            let query = search_input.read().clone();
                            let replacement = replace_input.read().clone();
                            if query.is_empty() {
                                return;
                            }
                            let mut state = state;
                            let confirm_title = replace_confirm_title_t.clone();
                            let confirm_msg_tpl = replace_confirm_msg_t.clone();
                            *replacing.write() = true;
                            *replace_status.write() = String::new();
                            spawn(async move {
                                EditorActions::flush_from_dom(&mut state).await;

                                // 预览影响范围并确认 / Preview impact and confirm
                                let overrides = collect_open_buffer_overrides(&state);
                                let (files_n, matches_n) = if let Some(root) =
                                    state.ui().workspace_root.read().clone()
                                {
                                    let files = collect_workspace_files(&root, &overrides);
                                    preview_workspace_replace_counts(&files, &query)
                                } else {
                                    let content = state.document().content.read().clone();
                                    let n = crate::utils::replace::count_matches(
                                        &content, &query, true, false,
                                    );
                                    if n > 0 { (1, n) } else { (0, 0) }
                                };

                                if files_n == 0 {
                                    *replace_status.write() =
                                        t("replace_workspace_none", *state.ui().language.read());
                                    *replacing.write() = false;
                                    return;
                                }

                                let description = confirm_msg_tpl
                                    .replace("{files}", &files_n.to_string())
                                    .replace("{matches}", &matches_n.to_string());
                                let confirmed = rfd::MessageDialog::new()
                                    .set_title(&confirm_title)
                                    .set_description(&description)
                                    .set_buttons(rfd::MessageButtons::OkCancel)
                                    .show();
                                if confirmed != rfd::MessageDialogResult::Ok {
                                    *replacing.write() = false;
                                    return;
                                }

                                let report = FileActions::replace_in_workspace(
                                    &mut state,
                                    &query,
                                    &replacement,
                                );
                                let lang = *state.ui().language.read();
                                let msg = if report.files_touched == 0 && report.matches_total == 0 {
                                    t("replace_workspace_none", lang)
                                } else {
                                    format!(
                                        "{} — {} / {}",
                                        t("replace_workspace_done", lang),
                                        report.files_touched,
                                        report.matches_total
                                    )
                                };
                                *replace_status.write() = msg;
                                *replacing.write() = false;
                                let workspace = state.ui().workspace_root.read().clone();
                                let overrides = collect_open_buffer_overrides(&state);
                                spawn_workspace_search(
                                    query,
                                    workspace,
                                    state.document().content.read().clone(),
                                    t("current_file", lang),
                                    overrides,
                                    results,
                                    searching,
                                );
                            });
                        },
                        if *replacing.read() { "{replacing_t}" } else { "{replace_all_workspace_t}" }
                    }
                }

                if !replace_status.read().is_empty() {
                    div { class: "search-replace-status", "{replace_status}" }
                }

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
                                            *ui.show_global_search.write() = false;
                                            let mut state = state;
                                            spawn(async move {
                                                let _ = FileActions::open_file_flushed(
                                                    &mut state,
                                                    file_path,
                                                )
                                                .await;
                                                let _ = dioxus::document::eval(&format!(
                                                    "if(window._mm_scrollToLine) window._mm_scrollToLine({})",
                                                    target_line
                                                ));
                                            });
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

                div { class: "modal-footer",
                    span { class: "search-hint", "{navigate_t}" }
                }
            }
        }
    }
}
