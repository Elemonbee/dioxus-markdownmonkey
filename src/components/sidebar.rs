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
    let ui = state.ui();

    // 读取状态 - 在顶部完成所有读取（领域视图）
    // Read state via domain view
    let sidebar_visible = *ui.sidebar_visible.read();
    let sidebar_tab = *ui.sidebar_tab.read();
    let sidebar_width = *ui.sidebar_width.read();
    let lang = *ui.language.read();

    let outline_text = t("outline", lang);
    let files_text = t("files", lang);
    let close_t = t("close", lang);
    let resize_t = t("sidebar_resize", lang);
    let aria_sidebar_t = t("aria_sidebar", lang);
    let aria_sidebar_content_t = t("aria_sidebar_content", lang);

    // 边缘拖拽调整宽度 / Edge-drag to resize width
    let mut resizing = use_signal(|| false);
    let mut drag_start_x = use_signal(|| 0.0_f64);
    let mut drag_start_width = use_signal(|| SIDEBAR_MIN_WIDTH);

    // 计算 CSS 类 - 纯计算，不涉及 hooks
    let container_class = if !sidebar_visible {
        "sidebar sidebar-collapsed"
    } else if *resizing.read() {
        "sidebar sidebar-resizing"
    } else {
        "sidebar"
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
    let header_title = if is_outline {
        outline_text.clone()
    } else {
        files_text.clone()
    };

    rsx! {
        div {
            class: "{container_class}",
            style: "width: {sidebar_width}px; min-width: {sidebar_width}px;",
            role: "complementary",
            "aria-label": "{aria_sidebar_t}",

            div { class: "sidebar-header",
                span { class: "sidebar-header-title", "{header_title}" }
                button {
                    class: "btn-icon",
                    title: "{close_t}",
                    onclick: move |_| {
                        AppActions::set_sidebar_visible(&mut state, false);
                    },
                    CloseIcon { size: 14 }
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
            }

            // 内容区域 - 条件渲染当前活动的视图
            // Content area - conditionally render the active view
            div { class: "sidebar-content", role: "region", "aria-label": "{aria_sidebar_content_t}",
                if is_outline {
                    OutlineView {}
                }
                if is_files {
                    FileTree {}
                }
            }

            // 右侧边缘拖拽把手（类似 IDE 分隔条）/ Right-edge drag handle (IDE-style splitter)
            div {
                class: "sidebar-resize-handle",
                role: "separator",
                "aria-orientation": "vertical",
                "aria-valuenow": "{sidebar_width}",
                "aria-valuemin": "{SIDEBAR_MIN_WIDTH}",
                "aria-valuemax": "{SIDEBAR_MAX_WIDTH}",
                title: "{resize_t}",
                onmousedown: move |e: Event<MouseData>| {
                    e.stop_propagation();
                    drag_start_x.set(e.client_coordinates().x);
                    drag_start_width.set(sidebar_width);
                    resizing.set(true);
                },
            }
        }

        // 全屏捕获层：拖拽时跟踪鼠标并更新宽度
        // Fullscreen capture layer: track mouse and update width while dragging
        if *resizing.read() {
            div {
                class: "sidebar-resize-overlay",
                onmousemove: move |e: Event<MouseData>| {
                    let dx = e.client_coordinates().x - *drag_start_x.read();
                    let next = (*drag_start_width.read() as f64 + dx)
                        .round()
                        .clamp(SIDEBAR_MIN_WIDTH as f64, SIDEBAR_MAX_WIDTH as f64)
                        as u32;
                    AppActions::set_sidebar_width(&mut state, next);
                },
                onmouseup: move |_| {
                    resizing.set(false);
                },
                onmouseleave: move |_| {
                    // 鼠标离开窗口时结束拖拽，避免卡住
                    // End drag if pointer leaves the window to avoid stuck resize
                    if *resizing.read() {
                        resizing.set(false);
                    }
                },
            }
        }
    }
}

/// 大纲视图 / Outline View
#[component]
fn OutlineView() -> Element {
    let state = use_context::<AppState>();
    let doc = state.document();
    let ui = state.ui();

    let outline_items = doc.outline_items.read().clone();
    let is_empty = outline_items.is_empty();
    let lang = *ui.language.read();

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
