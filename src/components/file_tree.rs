//! 文件树组件 / File Tree Component
//!
//! 遵循 PAL 架构：使用 Actions 处理业务逻辑
//! Provides file browsing, create, rename, delete functions

use crate::actions::FileActions;
use crate::components::icons::*;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::{ReadableExt, WritableExt, *};
use std::fs;
use std::path::{Path, PathBuf};

/// 文件树组件 / File Tree Component
#[component]
pub fn FileTree() -> Element {
    // 所有 hooks 在顶部
    let mut state = use_context::<AppState>();
    let mut search_query = use_signal(String::new);
    let mut is_searching = use_signal(|| false);
    // 折叠状态：记录哪些目录被折叠 / Collapse state: tracks which dirs are collapsed
    let collapsed_dirs = use_signal(std::collections::HashSet::<String>::new);

    // 读取状态
    let workspace = state.workspace_root.read().clone();
    let file_list = state.file_list.read().clone();

    // 预先克隆用于闭包
    let workspace_for_new_file = workspace.clone();
    let workspace_for_new_folder = workspace.clone();

    // 计算搜索结果
    let search_results: Vec<PathBuf> = if *is_searching.read() && !search_query.read().is_empty() {
        if let Some(root) = workspace.clone() {
            search_files(&root, &search_query.read())
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 计算显示状态
    let show_search = *is_searching.read() && !search_query.read().is_empty();
    let show_file_tree = !show_search;
    let show_empty_hint = show_search && search_results.is_empty();
    let show_no_workspace = file_list.is_empty() && workspace.is_none();
    let show_workspace_path = workspace.is_some();
    let show_search_results_info = show_search && !search_results.is_empty();
    let show_clear_btn = !search_query.read().is_empty();

    // CSS 计算
    let workspace_path_display = if show_workspace_path {
        ""
    } else {
        "display: none;"
    };
    let search_results_info_display = if show_search_results_info {
        ""
    } else {
        "display: none;"
    };
    let file_tree_display = if show_file_tree { "" } else { "display: none;" };
    let search_display = if show_search { "" } else { "display: none;" };
    let empty_hint_display = if show_empty_hint {
        ""
    } else {
        "display: none;"
    };
    let no_workspace_display = if show_no_workspace {
        ""
    } else {
        "display: none;"
    };

    // i18n
    let lang = *state.language.read();
    let files_t = t("files", lang);
    let new_file_t = t("new_file_btn", lang);
    let new_folder_t = t("new_folder", lang);
    let select_folder_t = t("select_folder", lang);
    let refresh_t = t("refresh", lang);
    let search_files_t = t("search_files", lang);
    let files_found_t = t("files_found", lang);
    let no_match_t = t("no_matching_files", lang);
    let click_select_t = t("click_to_select", lang);

    let tree_items = if let Some(ref root) = workspace {
        build_tree_from_files(root, &file_list, 0, &collapsed_dirs)
    } else {
        Vec::new()
    };

    rsx! {
        div { class: "file-tree",
            // 文件树头部
            div { class: "file-tree-header",
                span { "{files_t}" }
                div { class: "file-tree-actions",
                    button {
                        class: "btn-icon",
                        title: "{new_file_t}",
                        onclick: move |_| {
                            if let Some(root) = workspace_for_new_file.clone() {
                                if let Err(e) = FileActions::create_new_file(&mut state, &root, "untitled.md") {
                                    tracing::error!("Failed to create file: {}", e);
                                }
                            }
                        },
                        NewFileIcon { size: 14 }
                    }
                    button {
                        class: "btn-icon",
                        title: "{new_folder_t}",
                        onclick: move |_| {
                            if let Some(root) = workspace_for_new_folder.clone() {
                                if let Err(e) = FileActions::create_new_folder(&mut state, &root, "new_folder") {
                                    tracing::error!("Failed to create folder: {}", e);
                                }
                            }
                        },
                        FolderIcon { size: 14 }
                    }
                    button {
                        class: "btn-icon",
                        title: "{select_folder_t}",
                        onclick: move |_| {
                            let mut state = state;
                            spawn(async move {
                                let folder = rfd::AsyncFileDialog::new().pick_folder().await;
                                if let Some(path) = folder {
                                    let path = path.path().to_path_buf();
                                    FileActions::set_workspace(&mut state, path);
                                }
                            });
                        },
                        OpenFileIcon { size: 14 }
                    }
                    button {
                        class: "btn-icon",
                        title: "{refresh_t}",
                        onclick: move |_| {
                            FileActions::refresh_workspace(&mut state);
                        },
                        RefreshIcon { size: 14 }
                    }
                }
            }

            // 工作区路径显示 - 始终渲染
            div {
                class: "workspace-path",
                style: "{workspace_path_display}",
                if let Some(ref root) = workspace {
                    "{root.display()}"
                }
            }

            // 文件搜索框
            div { class: "file-search",
                input {
                    class: "file-search-input",
                    r#type: "text",
                    placeholder: "{search_files_t}",
                    value: "{search_query}",
                    onfocus: move |_| *is_searching.write() = true,
                    oninput: move |e| {
                        *search_query.write() = e.value();
                    },
                    onkeydown: move |e| {
                        if e.key() == Key::Escape {
                            *search_query.write() = String::new();
                            *is_searching.write() = false;
                        }
                    },
                }
                // 清除按钮 - 始终渲染
                button {
                    class: "file-search-clear",
                    style: if show_clear_btn { "" } else { "display: none;" },
                    onclick: move |_| {
                        *search_query.write() = String::new();
                    },
                    "×"
                }
            }

            // 文件列表
            div { class: "file-list",
                // 搜索结果信息 - 始终渲染
                div {
                    class: "search-results-info",
                    style: "{search_results_info_display}",
                    "{search_results.len()} {files_found_t}"
                }

                // 搜索结果 - 始终渲染
                div {
                    class: "search-results",
                    style: "{search_display}",
                    for (idx, file) in search_results.iter().enumerate() {
                        SearchResultItem {
                            key: "{idx}",
                            path: file.clone()
                        }
                    }
                }

                // 空搜索结果提示 - 始终渲染
                div {
                    class: "empty-state",
                    style: "{empty_hint_display}",
                    p { "{no_match_t}" }
                }

                // 文件树 - 使用缓存的 tree_items 渲染
                div {
                    class: "file-tree-flat",
                    style: "{file_tree_display}",
                    for item in tree_items.iter() {
                        FileTreeItemFlat {
                            key: "{item.path.display()}",
                            path: item.path.clone(),
                            depth: item.depth,
                            is_dir: item.is_dir,
                            name: item.name.clone(),
                            collapsed_dirs: collapsed_dirs,
                        }
                    }
                }

                // 空状态提示 - 始终渲染
                div {
                    class: "empty-state",
                    style: "{no_workspace_display}",
                    p { "{click_select_t}" }
                }
            }
        }
    }
}

/// 扁平化的文件树项
pub struct FlatItem {
    path: PathBuf,
    depth: usize,
    is_dir: bool,
    name: String,
}

/// 从缓存的文件列表构建树形结构
/// Build tree structure from cached file list
fn build_tree_from_files(
    root: &Path,
    files: &[PathBuf],
    depth: usize,
    collapsed_dirs: &Signal<std::collections::HashSet<String>>,
) -> Vec<FlatItem> {
    let mut items = Vec::new();

    // 收集唯一目录和文件 / Collect unique directories and files
    let mut all_paths: std::collections::BTreeMap<String, (PathBuf, bool)> =
        std::collections::BTreeMap::new();

    for file_path in files {
        if !file_path.starts_with(root) {
            continue;
        }

        // 获取相对于根目录的路径 / Get path relative to root
        if let Ok(rel_path) = file_path.strip_prefix(root) {
            let mut dir_path = root.to_path_buf();
            let parts: Vec<_> = rel_path.iter().collect();

            for (i, part) in parts.iter().enumerate() {
                dir_path = dir_path.join(part);
                let key = dir_path.to_string_lossy().to_string();

                if i < parts.len() - 1 {
                    // 这是目录 / This is a directory
                    all_paths
                        .entry(key)
                        .or_insert_with(|| (dir_path.clone(), true));
                } else {
                    // 这是文件 / This is a file
                    all_paths.insert(key, (file_path.clone(), false));
                }
            }
        }
    }

    build_tree_recursive(root, root, depth, &mut items, collapsed_dirs, &all_paths);
    items
}

/// 递归构建树 / Recursively build tree
#[allow(clippy::only_used_in_recursion)]
fn build_tree_recursive(
    root: &Path,
    current_dir: &Path,
    depth: usize,
    items: &mut Vec<FlatItem>,
    collapsed_dirs: &Signal<std::collections::HashSet<String>>,
    all_paths: &std::collections::BTreeMap<String, (PathBuf, bool)>,
) {
    if items.len() > 500 {
        return;
    }

    // 获取当前目录的直接子项 / Get direct children of current directory
    let mut children: Vec<(String, PathBuf, bool)> = all_paths
        .values()
        .filter(|(path, _is_dir)| path.parent() == Some(current_dir))
        .map(|(path, is_dir)| {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            (name, path.clone(), *is_dir)
        })
        .collect();

    // 排序：文件夹优先 / Sort: directories first
    children.sort_by(|a, b| match (a.2, b.2) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.0.cmp(&b.0),
    });

    for (name, path, is_dir) in children {
        items.push(FlatItem {
            path: path.clone(),
            depth,
            is_dir,
            name: name.clone(),
        });

        // 如果是目录且未被折叠，递归展开 / Recurse if dir and not collapsed
        if is_dir {
            let dir_key = path.to_string_lossy().to_string();
            let is_collapsed = collapsed_dirs.read().contains(&dir_key);
            if !is_collapsed {
                build_tree_recursive(root, &path, depth + 1, items, collapsed_dirs, all_paths);
            }
        }
    }
}

/// 搜索结果项属性
#[derive(Props, Clone, PartialEq)]
struct SearchResultItemProps {
    path: PathBuf,
}

/// 搜索结果项组件
#[component]
fn SearchResultItem(props: SearchResultItemProps) -> Element {
    let mut state = use_context::<AppState>();
    let path = props.path.clone();

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    rsx! {
        div {
            class: "search-result-item",
            onclick: move |_| {
                if let Err(e) = FileActions::open_file(&mut state, path.clone()) {
                    tracing::error!("Failed to open file: {}", e);
                }
            },
            FileIcon { size: 14, class: "icon".to_string() }
            span { class: "name", "{name}" }
        }
    }
}

/// 扁平化文件树项属性
#[derive(Props, Clone, PartialEq)]
struct FileTreeItemFlatProps {
    path: PathBuf,
    depth: usize,
    is_dir: bool,
    name: String,
    #[props(!optional)]
    collapsed_dirs: Signal<std::collections::HashSet<String>>,
}

/// 获取文件类型图标的 SVG 字符串（用于 inline 渲染）
fn file_type_icon_svg(is_dir: bool, ext: &str) -> &'static str {
    if is_dir {
        r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>"#
    } else {
        match ext {
            "md" | "markdown" | "txt" => {
                r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>"#
            }
            "rs" => {
                r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.26.6.77 1.05 1.4 1.18H21a2 2 0 0 1 0 4h-.09c-.63.13-1.14.58-1.4 1.18z"/></svg>"#
            }
            "js" | "ts" => {
                r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>"#
            }
            "json" => {
                r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>"#
            }
            "png" | "jpg" | "gif" | "svg" | "webp" => {
                r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>"#
            }
            _ => {
                r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>"#
            }
        }
    }
}

/// 折叠/展开箭头 SVG / Collapse/expand arrow SVG
fn collapse_arrow_svg(is_collapsed: bool) -> &'static str {
    if is_collapsed {
        // 右箭头（折叠）/ Right arrow (collapsed)
        r#"<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>"#
    } else {
        // 下箭头（展开）/ Down arrow (expanded)
        r#"<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>"#
    }
}

/// 扁平化文件树项组件
#[component]
fn FileTreeItemFlat(props: FileTreeItemFlatProps) -> Element {
    let mut state = use_context::<AppState>();
    let mut collapsed_dirs = props.collapsed_dirs;
    let mut show_context_menu = use_signal(|| false);
    let mut show_rename_input = use_signal(|| false);
    let mut rename_value = use_signal(String::new);

    let path = props.path.clone();
    let depth = props.depth;
    let is_dir = props.is_dir;
    let name = props.name.clone();

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();
    let is_markdown = !is_dir && (ext == "md" || ext == "markdown" || ext == "txt");
    let icon_svg = file_type_icon_svg(is_dir, &ext);

    let dir_key = path.to_string_lossy().to_string();
    let is_collapsed = is_dir && collapsed_dirs.read().contains(&dir_key);
    let arrow_svg = if is_dir {
        Some(collapse_arrow_svg(is_collapsed))
    } else {
        None
    };

    let indent = depth * 16;
    let item_class = if is_dir {
        "file-item folder"
    } else if is_markdown {
        "file-item markdown"
    } else {
        "file-item"
    };

    let lang = *state.language.read();
    let rename_t = t("rename_file", lang);
    let delete_t = t("delete_file", lang);
    let new_file_t = t("new_file_btn", lang);
    let new_folder_t = t("new_folder", lang);

    let path_for_click = path.clone();
    let path_for_rename = path.clone();
    let path_for_delete = path.clone();
    let path_for_new_file = path.clone();
    let path_for_new_folder = path.clone();
    let name_for_rename = name.clone();
    let name_for_rename_ctx = name.clone();

    rsx! {
        div {
            class: "{item_class}",
            style: "padding-left: {indent}px;",
            onclick: move |e| {
                e.stop_propagation();
                if *show_context_menu.read() {
                    show_context_menu.set(false);
                    return;
                }
                if *show_rename_input.read() {
                    return;
                }
                if is_dir {
                    let key = path_for_click.to_string_lossy().to_string();
                    let mut dirs = collapsed_dirs.write();
                    if dirs.contains(&key) {
                        dirs.remove(&key);
                    } else {
                        dirs.insert(key);
                    }
                } else if is_markdown {
                    if let Err(err) = FileActions::open_file(&mut state, path_for_click.clone()) {
                        tracing::error!("Failed to open file: {}", err);
                    }
                }
            },
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                show_context_menu.set(true);
            },

            if let Some(arrow) = arrow_svg {
                span { class: "collapse-arrow", dangerous_inner_html: arrow }
            } else {
                span { class: "collapse-arrow-spacer" }
            }

            span { class: "icon", dangerous_inner_html: icon_svg }

            if *show_rename_input.read() {
                input {
                    class: "file-rename-input",
                    r#type: "text",
                    value: "{rename_value}",
                    autofocus: true,
                    oninput: move |e| {
                        rename_value.set(e.value());
                    },
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            let new_name = rename_value.read().clone();
                            if !new_name.is_empty() && new_name != name_for_rename {
                                if let Err(err) = FileActions::rename_file(&mut state, &path_for_rename, &new_name) {
                                    tracing::error!("Rename failed: {}", err);
                                }
                            }
                            show_rename_input.set(false);
                        } else if e.key() == Key::Escape {
                            show_rename_input.set(false);
                        }
                    },
                    onblur: move |_| {
                        show_rename_input.set(false);
                    },
                    onclick: move |e| e.stop_propagation(),
                }
            } else {
                span { class: "name", "{name}" }
            }

            if *show_context_menu.read() {
                div {
                    class: "context-menu-overlay",
                    onclick: move |_| {
                        show_context_menu.set(false);
                    },
                    oncontextmenu: move |e| {
                        e.prevent_default();
                        show_context_menu.set(false);
                    },
                }
                div {
                    class: "context-menu",

                    if is_dir {
                        button {
                            class: "context-menu-item",
                            onclick: move |e| {
                                e.stop_propagation();
                                show_context_menu.set(false);
                                if let Err(err) = FileActions::create_new_file(&mut state, &path_for_new_file, "untitled.md") {
                                    tracing::error!("Create file failed: {}", err);
                                }
                            },
                            "{new_file_t}"
                        }
                        button {
                            class: "context-menu-item",
                            onclick: move |e| {
                                e.stop_propagation();
                                show_context_menu.set(false);
                                if let Err(err) = FileActions::create_new_folder(&mut state, &path_for_new_folder, "new_folder") {
                                    tracing::error!("Create folder failed: {}", err);
                                }
                            },
                            "{new_folder_t}"
                        }
                    }

                    button {
                        class: "context-menu-item",
                        onclick: move |e| {
                            e.stop_propagation();
                            show_context_menu.set(false);
                            rename_value.set(name_for_rename_ctx.clone());
                            show_rename_input.set(true);
                        },
                        "{rename_t}"
                    }

                    button {
                        class: "context-menu-item danger",
                        onclick: move |e| {
                            e.stop_propagation();
                            show_context_menu.set(false);
                            if let Err(err) = FileActions::delete_file(&mut state, &path_for_delete) {
                                tracing::error!("Delete failed: {}", err);
                            }
                        },
                        "{delete_t}"
                    }
                }
            }
        }
    }
}

/// 搜索文件最大深度 / Max search depth
const MAX_SEARCH_DEPTH: usize = 10;
/// 搜索最大结果数 / Max search results
const MAX_SEARCH_RESULTS: usize = 500;

/// 搜索文件（按文件名）
fn search_files(dir: &Path, query: &str) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();
    search_files_recursive(dir, &query_lower, &mut results, 0);
    results
}

/// 递归搜索文件
fn search_files_recursive(dir: &Path, query: &str, results: &mut Vec<PathBuf>, depth: usize) {
    if depth >= MAX_SEARCH_DEPTH || results.len() >= MAX_SEARCH_RESULTS {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }

            if path.is_dir() {
                search_files_recursive(&path, query, results, depth + 1);
            } else {
                let name_lower = name.to_lowercase();
                if name_lower.contains(query) {
                    if let Some(ext) = path.extension() {
                        if ext == "md" || ext == "markdown" || ext == "txt" {
                            results.push(path);
                        }
                    }
                }
            }
        }
    }
}
