//! 编辑器组件 / Editor Component
//!
//! 遵循 PAL 林构构：使用 Actions 处理理编辑器操作
//! 支持虚拟滚动行号以提升大文件性能 / Following PAL architecture: Using Actions for business logic
//! Support virtual scrolling line numbers for large file performance

use crate::actions::shortcut_actions::ShortcutActions;
use crate::actions::{AppActions, EditorActions};
use crate::components::icons::PreviewIcon;
use crate::config::{
    EDITOR_LINE_HEIGHT_PX, EDITOR_VIRTUAL_SCROLL_BUFFER_LINES,
    EDITOR_VIRTUAL_SCROLL_THRESHOLD_LINES,
};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::document;
use dioxus::prelude::{ReadableExt, WritableExt, *};

/// 编辑器滚动比例（全局信号）/ Editor scroll ratio (global signal)
pub static EDITOR_SCROLL_RATIO: GlobalSignal<f32> = Signal::global(|| 0.0);

/// 编辑器组件 / Editor Component
#[component]
pub fn Editor() -> Element {
    // 所有 hooks 在顶部 / All hooks at top
    let mut state = use_context::<AppState>();
    let mut is_dragging = use_signal(|| false);

    let mut scroll_top = use_signal(|| 0.0_f32);
    let mut container_height = use_signal(|| 600.0_f32);

    let content = state.content.read().clone();
    let modified = *state.modified.read();
    let current_file = state.current_file.read().clone();
    let show_preview = *state.show_preview.read();

    let search_query_hl = state.search_query.read().clone();
    let search_index_hl = *state.search_index.read();
    let search_total_hl = *state.search_total.read();
    let case_insensitive_hl = *state.search_case_insensitive.read();
    let show_search_hl = *state.show_search.read();

    let lang = *state.language.read();
    let placeholder_text = t("placeholder_input", lang);
    let untitled_text = t("untitled", lang);

    let filename = current_file
        .as_ref()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("{untitled_text}.md"))
        })
        .unwrap_or_else(|| format!("{untitled_text}.md"));

    let line_count = content.lines().count().max(1);

    let use_virtual = line_count > EDITOR_VIRTUAL_SCROLL_THRESHOLD_LINES;
    let (render_start, render_end) = if use_virtual {
        let st = *scroll_top.read();
        let ch = *container_height.read();
        let first_visible = (st / EDITOR_LINE_HEIGHT_PX) as usize;
        let visible_count = (ch / EDITOR_LINE_HEIGHT_PX) as usize + 1;
        let start = first_visible.saturating_sub(EDITOR_VIRTUAL_SCROLL_BUFFER_LINES);
        let end =
            (first_visible + visible_count + EDITOR_VIRTUAL_SCROLL_BUFFER_LINES).min(line_count);
        (start, end)
    } else {
        (0, line_count)
    };

    let pane_class = if *is_dragging.read() {
        "editor-pane drag-over"
    } else {
        "editor-pane"
    };
    let preview_btn_class = if show_preview {
        "btn-icon active"
    } else {
        "btn-icon"
    };
    let modified_display = if modified { "" } else { "display: none;" };

    let line_numbers_style = if use_virtual {
        let padding_top = render_start as f32 * EDITOR_LINE_HEIGHT_PX;
        let total_height = line_count as f32 * EDITOR_LINE_HEIGHT_PX;
        format!("padding-top: {padding_top}px; min-height: {total_height}px;")
    } else {
        String::new()
    };

    let _ = use_effect(move || {
        if show_search_hl && !search_query_hl.is_empty() && search_total_hl > 0 {
            let idx = if search_index_hl > 0 {
                search_index_hl - 1
            } else {
                0
            };
            let safe_query =
                serde_json::to_string(&search_query_hl).unwrap_or_else(|_| "\"\"".to_string());
            let _ = document::eval(&format!(
                "if(window._mm_highlightSearch) window._mm_highlightSearch({}, {}, {})",
                safe_query, case_insensitive_hl, idx
            ));
        } else {
            let _ = document::eval(
                "if(window._mm_highlightSearch) window._mm_highlightSearch('', false, 0)",
            );
        }
    });

    let _ = use_effect(|| {
        let js = include_str!("../../assets/editor_enhance.js");
        let _ = document::eval(js);
    });

    let _ = use_effect(move || {
        let _ = state.content.read();
        let _ = document::eval("if(window._mm_initEditor) window._mm_initEditor();");
    });

    rsx! {
        div {
            class: "{pane_class}",

            ondragover: move |e| {
                e.prevent_default();
                *is_dragging.write() = true;
            },
            ondragleave: move |_| {
                *is_dragging.write() = false;
            },
            ondrop: move |_| {
                tracing::info!("Drag detected - use Ctrl+O to open files");
            },

            div { class: "editor-header",
                span { class: "filename",
                    "{filename}"
                    span {
                        class: "modified-indicator",
                        style: "{modified_display}",
                        " ●"
                    }
                }

                button {
                    class: "{preview_btn_class}",
                    title: "切换预览 / Toggle Preview (Ctrl+P)",
                    onclick: move |_| {
                        AppActions::toggle_preview(&mut state);
                    },
                    PreviewIcon { size: 16 }
                }
            }

            div {
                class: "editor-content",
                div {
                    class: "line-numbers",
                    style: "{line_numbers_style}",

                    if use_virtual {
                        for i in (render_start + 1)..=(render_end) {
                            div { class: "line-number", key: "{i}", "{i}" }
                        }
                    } else {
                        for i in 1..=line_count {
                            div { class: "line-number", key: "{i}", "{i}" }
                        }
                    }
                }
                textarea {
                    class: "editor-textarea",
                    value: "{content}",
                    placeholder: "{placeholder_text}",
                    spellcheck: false,
                    onscroll: move |e| {
                        let scroll_data = e.data();
                        let sh = scroll_data.scroll_height() as f32;
                        let ch = scroll_data.client_height() as f32;
                        let st = scroll_data.scroll_top() as f32;
                        scroll_top.set(st);
                        container_height.set(ch);
                        let ratio = if sh > ch {
                            st / (sh - ch)
                        } else {
                            0.0
                        };
                        *EDITOR_SCROLL_RATIO.write() = ratio;
                    },
                    oninput: move |e| {
                        EditorActions::update_content(&mut state, e.value());
                    },
                    onkeydown: move |e| {
                        if ShortcutActions::handle_event(&mut state, &e) {
                            e.prevent_default();
                        }
                    },
                }
            }
        }
    }
}
