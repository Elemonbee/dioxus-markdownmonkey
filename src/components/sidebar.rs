//! 侧边栏组件 / Sidebar Component
//!
//! 遵循 PAL 架构：使用 Actions 处理业务逻辑
//! Following PAL architecture: Use Actions for business logic

use crate::actions::AppActions;
use crate::components::file_tree::FileTree;
use crate::components::icons::{CloseIcon, FileIcon};
use crate::config::{SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::services::recent_files::RecentFile;
use crate::state::{AppState, Language, SidebarTab};
use crate::utils::i18n::t;
use dioxus::document;
use dioxus::prelude::*;
use std::path::PathBuf;

/// 侧边栏组件 / Sidebar Component
#[component]
pub fn Sidebar() -> Element {
    // 所有 hooks 必须在组件顶部无条件调用
    let mut state = use_context::<AppState>();

    // 读取状态 - 在顶部完成所有读取
    let sidebar_visible = *state.sidebar_visible.read();
    let sidebar_tab = *state.sidebar_tab.read();
    let sidebar_width = *state.sidebar_width.read();
    let lang = *state.language.read();

    let outline_text = t("outline", lang);
    let files_text = t("files", lang);
    let recent_text = t("recent", lang);
    let close_t = t("close", lang);
    let sidebar_width_text = t("sidebar_width", lang);

    // 计算 CSS 类 - 纯计算，不涉及 hooks
    let container_class = if sidebar_visible {
        "sidebar"
    } else {
        "sidebar sidebar-collapsed"
    };
    let tab_outline_class = if sidebar_tab == SidebarTab::Outline {
        "sidebar-tab active"
    } else {
        "sidebar-tab"
    };
    let tab_files_class = if sidebar_tab == SidebarTab::Files {
        "sidebar-tab active"
    } else {
        "sidebar-tab"
    };
    let tab_recent_class = if sidebar_tab == SidebarTab::Recent {
        "sidebar-tab active"
    } else {
        "sidebar-tab"
    };
    let is_outline = sidebar_tab == SidebarTab::Outline;
    let is_files = sidebar_tab == SidebarTab::Files;
    let is_recent = sidebar_tab == SidebarTab::Recent;

    rsx! {
        div {
            class: "{container_class}",
            style: "width: {sidebar_width}px; min-width: {sidebar_width}px;",
            role: "complementary",
            "aria-label": "Sidebar",

            div { class: "sidebar-header",
                span { class: "sidebar-header-title", "{files_text}" }
                div { class: "sidebar-header-actions",
                    input {
                        class: "sidebar-width-slider",
                        r#type: "range",
                        min: "{SIDEBAR_MIN_WIDTH}",
                        max: "{SIDEBAR_MAX_WIDTH}",
                        value: "{sidebar_width}",
                        title: "{sidebar_width_text}",
                        oninput: move |e| {
                            if let Ok(width) = e.value().parse::<u32>() {
                                AppActions::set_sidebar_width(&mut state, width);
                            }
                        },
                    }
                    button {
                        class: "btn-icon",
                        title: "{close_t}",
                        onclick: move |_| {
                            AppActions::set_sidebar_visible(&mut state, false);
                        },
                        CloseIcon { size: 14 }
                    }
                }
            }

            // 标签切换 / Tab Switching
            div { class: "sidebar-tabs", role: "tablist",
                button {
                    class: "{tab_outline_class}",
                    role: "tab",
                    aria_selected: "{is_outline}",
                    onclick: move |_| {
                        AppActions::set_sidebar_tab(&mut state, SidebarTab::Outline);
                    },
                    "{outline_text}"
                }
                button {
                    class: "{tab_files_class}",
                    role: "tab",
                    aria_selected: "{is_files}",
                    onclick: move |_| {
                        AppActions::set_sidebar_tab(&mut state, SidebarTab::Files);
                    },
                    "{files_text}"
                }
                button {
                    class: "{tab_recent_class}",
                    role: "tab",
                    aria_selected: "{is_recent}",
                    onclick: move |_| {
                        AppActions::set_sidebar_tab(&mut state, SidebarTab::Recent);
                    },
                    "{recent_text}"
                }
            }

            // 内容区域 - 条件渲染当前活动的视图
            // Content area - conditionally render the active view
            div { class: "sidebar-content", role: "region", "aria-label": "Sidebar content",
                if is_outline {
                    OutlineView {}
                }
                if is_files {
                    FileTree {}
                }
                if is_recent {
                    RecentFilesView {}
                }
            }
        }
    }
}

/// 大纲视图 / Outline View
#[component]
fn OutlineView() -> Element {
    let state = use_context::<AppState>();

    let outline_items = state.outline_items.read().clone();
    let is_empty = outline_items.is_empty();
    let lang = *state.language.read();

    let no_outline_text = t("no_outline", lang);
    let add_headings_text = t("add_headings", lang);

    rsx! {
        div { class: "outline-view",
            if is_empty {
                div {
                    class: "empty-hint",
                    p { "{no_outline_text}" }
                    p { class: "hint-desc", "{add_headings_text}" }
                }
            } else {
                div {
                    class: "outline-list",

                    for item in outline_items.iter() {
                        OutlineItemView {
                            key: "{item.line}-{item.level}",
                            level: item.level,
                            text: item.text.clone(),
                            line: item.line,
                        }
                    }
                }
            }
        }
    }
}

/// 大纲项组件属性 / Outline Item Props
#[derive(Props, Clone, PartialEq)]
struct OutlineItemProps {
    level: u8,    // 标题级别 / Heading Level
    text: String, // 标题文本 / Heading Text
    line: usize,  // 行号 / Line Number
}

/// 大纲项视图 / Outline Item View
#[component]
fn OutlineItemView(props: OutlineItemProps) -> Element {
    let indent = (props.level.saturating_sub(1)) * 16;
    let marker = "#".repeat(props.level as usize);

    rsx! {
        div {
            class: "outline-item outline-level-{props.level}",
            style: "padding-left: {indent}px;",
            onclick: move |_| {
                let target_line = props.line;
                let _ = document::eval(&format!(
                    "if(window._mm_scrollToLine) window._mm_scrollToLine({})",
                    target_line
                ));
            },
            span { class: "outline-marker", "{marker}" }
            span { class: "outline-text", "{props.text}" }
        }
    }
}

// ========== 最近文件视图 / Recent Files View ==========

/// 最近文件视图 / Recent Files View
#[component]
fn RecentFilesView() -> Element {
    let state = use_context::<AppState>();
    let lang = *state.language.read();

    // 获取排序后的最近文件列表 / Get sorted recent files list
    let files: Vec<RecentFile> = {
        let rf = state.recent_files.read();
        rf.sorted_by_frecency().into_iter().cloned().collect()
    };

    let is_empty = files.is_empty();
    let no_recent_text = t("no_recent_files", lang);
    let no_recent_hint = t("no_recent_hint", lang);

    rsx! {
        div { class: "recent-files-view",
            if is_empty {
                div { class: "empty-hint",
                    p { "{no_recent_text}" }
                    p { class: "hint-desc", "{no_recent_hint}" }
                }
            } else {
                div { class: "recent-files-list",
                    for file in files.iter() {
                        RecentFileItem {
                            key: "{file.path.display()}",
                            path: file.path.clone(),
                            name: file.name.clone(),
                            last_opened: file.last_opened,
                        }
                    }
                }
            }
        }
    }
}

/// 最近文件项属性 / Recent File Item Props
#[derive(Props, Clone, PartialEq)]
struct RecentFileItemProps {
    path: PathBuf,
    name: String,
    last_opened: u64,
}

/// 最近文件项视图 / Recent File Item View
#[component]
fn RecentFileItem(props: RecentFileItemProps) -> Element {
    let mut state = use_context::<AppState>();
    let lang = *state.language.read();
    let remove_text = t("remove", lang);

    // 计算相对时间 / Calculate relative time
    let relative_time = format_relative_time(props.last_opened, lang);

    // 预克隆用于闭包 / Pre-clone for closures
    let path_for_open = props.path.clone();
    let path_for_remove = props.path.clone();
    let path_display = props.path.to_string_lossy().to_string();

    rsx! {
        div { class: "recent-file-item",
            div {
                class: "recent-file-info",
                onclick: move |_| {
                    AppActions::open_recent_file(&mut state, path_for_open.clone());
                },
                FileIcon { size: 14 }
                div { class: "recent-file-details",
                    span { class: "recent-file-name", "{props.name}" }
                    span { class: "recent-file-path", "{path_display}" }
                }
            }
            div { class: "recent-file-meta",
                span { class: "recent-file-time", "{relative_time}" }
                button {
                    class: "btn-icon recent-file-remove",
                    title: "{remove_text}",
                    onclick: move |e| {
                        e.stop_propagation();
                        AppActions::remove_recent_file(&mut state, path_for_remove.clone());
                    },
                    CloseIcon { size: 12 }
                }
            }
        }
    }
}

/// 格式化相对时间 / Format relative time
fn format_relative_time(timestamp: u64, lang: Language) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let elapsed_secs = now.saturating_sub(timestamp);

    if elapsed_secs < 60 {
        t("just_now", lang)
    } else if elapsed_secs < 3600 {
        let mins = elapsed_secs / 60;
        match lang {
            Language::ZhCN => format!("{} 分钟前", mins),
            Language::EnUS => format!("{}m ago", mins),
        }
    } else if elapsed_secs < 86400 {
        let hours = elapsed_secs / 3600;
        match lang {
            Language::ZhCN => format!("{} 小时前", hours),
            Language::EnUS => format!("{}h ago", hours),
        }
    } else {
        let days = elapsed_secs / 86400;
        match lang {
            Language::ZhCN => format!("{} 天前", days),
            Language::EnUS => format!("{}d ago", days),
        }
    }
}
