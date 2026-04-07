//! 侧边栏组件 / Sidebar Component
//!
//! 遵循 PAL 架构：使用 Actions 处理业务逻辑
//! Following PAL architecture: Use Actions for business logic

use crate::actions::AppActions;
use crate::components::file_tree::FileTree;
use crate::components::icons::CloseIcon;
use crate::config::{SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::state::AppState;
use crate::state::SidebarTab;
use crate::utils::i18n::t;
use dioxus::document;
use dioxus::prelude::*;

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
    let is_outline = sidebar_tab == SidebarTab::Outline;
    let is_files = sidebar_tab == SidebarTab::Files;

    rsx! {
        div {
            class: "{container_class}",
            style: "width: {sidebar_width}px; min-width: {sidebar_width}px;",

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
            div { class: "sidebar-tabs",
                button {
                    class: "{tab_outline_class}",
                    onclick: move |_| {
                        AppActions::set_sidebar_tab(&mut state, SidebarTab::Outline);
                    },
                    "{outline_text}"
                }
                button {
                    class: "{tab_files_class}",
                    onclick: move |_| {
                        AppActions::set_sidebar_tab(&mut state, SidebarTab::Files);
                    },
                    "{files_text}"
                }
            }

            // 内容区域 - 条件渲染当前活动的视图
            // Content area - conditionally render the active view
            div { class: "sidebar-content",
                if is_outline {
                    OutlineView {}
                }
                if is_files {
                    FileTree {}
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
