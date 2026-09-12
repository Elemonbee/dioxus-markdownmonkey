//! 编辑器组件 / Editor Component
//!
//! 遵循 PAL 架构：使用 Actions 处理编辑器操作
//! 小文件：受控 textarea；大文件：非受控 + 防抖同步，降低按键路径开销
//! Following PAL architecture with Actions
//! Small files: controlled textarea; large files: uncontrolled + debounced sync

use crate::actions::shortcut_actions::ShortcutActions;
use crate::actions::{AppActions, EditorActions, FileActions};
use crate::components::icons::PreviewIcon;
use crate::config::{
    EDITOR_LINE_HEIGHT_PX, EDITOR_VIRTUAL_SCROLL_BUFFER_LINES,
    EDITOR_VIRTUAL_SCROLL_THRESHOLD_LINES, UNCONTROLLED_EDITOR_SYNC_DEBOUNCE_MS,
    UNCONTROLLED_EDITOR_THRESHOLD_BYTES,
};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::document;
use dioxus::html::HasFileData;
use dioxus::prelude::{ReadableExt, WritableExt, *};
use std::path::PathBuf;

/// 编辑器滚动比例（全局信号）/ Editor scroll ratio (global signal)
pub static EDITOR_SCROLL_RATIO: GlobalSignal<f32> = Signal::global(|| 0.0);

/// 统计行数（按换行符，至少为 1）/ Count lines by newlines (at least 1)
fn count_lines(content: &str) -> usize {
    if content.is_empty() {
        return 1;
    }
    content.bytes().filter(|&b| b == b'\n').count() + 1
}

/// 从 DOM 同步 textarea 选区到 UI 域光标信号
/// Sync textarea selection from DOM into UI-domain cursor signals
fn sync_selection_from_dom(state: &mut AppState) {
    let mut state = *state;
    spawn(async move {
        let mut eval = document::eval(
            r#"
            (function() {
                if (window._mm_getSelection) {
                    dioxus.send(window._mm_getSelection());
                    return;
                }
                const ta = document.querySelector('.editor-textarea');
                if (!ta) { dioxus.send([0, 0]); return; }
                const value = ta.value || '';
                const toBytes = function(offset) {
                    let safe = Math.max(0, Math.min(Number(offset) || 0, value.length));
                    if (safe > 0 && safe < value.length) {
                        const before = value.charCodeAt(safe - 1);
                        const after = value.charCodeAt(safe);
                        if (before >= 0xD800 && before <= 0xDBFF &&
                            after >= 0xDC00 && after <= 0xDFFF) safe -= 1;
                    }
                    return new TextEncoder().encode(value.slice(0, safe)).length;
                };
                dioxus.send([toBytes(ta.selectionStart), toBytes(ta.selectionEnd)]);
            })();
            "#,
        );
        if let Ok(vals) = eval.recv::<Vec<usize>>().await {
            if vals.len() >= 2 {
                EditorActions::set_selection(&mut state, vals[0], vals[1]);
            }
        }
    });
}

/// 处理编辑器拖放：带路径的 md/txt 打开为标签
/// Handle editor drop: open md/txt with filesystem paths as tabs
fn handle_editor_drop(mut state: AppState, e: Event<DragData>) {
    e.prevent_default();
    let paths: Vec<PathBuf> = e
        .files()
        .into_iter()
        .map(|f| f.path())
        .filter(|p| !p.as_os_str().is_empty())
        .collect();
    if paths.is_empty() {
        return;
    }
    spawn(async move {
        FileActions::handle_editor_file_drop_flushed(&mut state, paths).await;
    });
}

/// 调度非受控模式的防抖同步 / Schedule debounced sync for uncontrolled mode
fn schedule_uncontrolled_sync(state: AppState, mut sync_gen: Signal<u64>) {
    let next = *sync_gen.read() + 1;
    sync_gen.set(next);
    let mut state = state;
    spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(
            UNCONTROLLED_EDITOR_SYNC_DEBOUNCE_MS,
        ))
        .await;
        if *sync_gen.read() != next {
            return;
        }
        EditorActions::flush_from_dom(&mut state).await;
    });
}

/// 编辑器组件 / Editor Component
#[component]
pub fn Editor() -> Element {
    // 所有 hooks 在顶部 / All hooks at top
    let mut state = use_context::<AppState>();
    let doc = state.document();
    let ui = state.ui();
    let mut is_dragging = use_signal(|| false);

    let mut scroll_top = use_signal(|| 0.0_f32);
    let mut container_height = use_signal(|| 600.0_f32);
    let sync_gen = use_signal(|| 0u64);
    let mut last_pushed_rev = use_signal(|| u64::MAX);

    // 仅在 content_revision 变化时克隆全文，滚动等局部更新不再 O(n) 拷贝
    // Clone full content only when content_revision changes; scroll updates skip O(n) copy
    let mut cached_content = use_signal(String::new);
    let mut cached_line_count = use_signal(|| 1usize);
    let mut cached_rev = use_signal(|| u64::MAX);

    let rev = *doc.content_revision.read();
    if rev != *cached_rev.read() {
        let borrowed = doc.content.read();
        let lines = count_lines(&borrowed);
        cached_content.set(borrowed.clone());
        cached_line_count.set(lines);
        cached_rev.set(rev);
    }

    let content = cached_content.read().clone();
    let line_count = *cached_line_count.read();
    let modified = *doc.modified.read();
    let current_file = doc.current_file.read().clone();
    let show_preview = *ui.show_preview.read();
    let tab_index = *doc.current_tab_index.read();
    let file_size = (*doc.file_size_bytes.read()).max(content.len());
    let use_uncontrolled = file_size >= UNCONTROLLED_EDITOR_THRESHOLD_BYTES;

    // CodeMirror 与受控/非受控共用：修订变化时把 Rust 正文推回 DOM（JS 侧相同则跳过）
    // Shared by CodeMirror and both editor modes: push Rust text on revision; JS no-ops if equal
    if rev != *last_pushed_rev.read() {
        last_pushed_rev.set(rev);
        EditorActions::push_to_dom(&content);
    }

    let search_query_hl = ui.search_query.read().clone();
    let search_index_hl = *ui.search_index.read();
    let search_total_hl = *ui.search_total.read();
    let case_insensitive_hl = *ui.search_case_insensitive.read();
    let show_search_hl = *ui.show_search.read();

    let lang = *ui.language.read();
    let placeholder_text = t("placeholder_input", lang);
    let aria_editor_t = t("aria_editor", lang);
    let untitled_text = t("untitled", lang);

    let filename = current_file
        .as_ref()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("{untitled_text}.md"))
        })
        .unwrap_or_else(|| format!("{untitled_text}.md"));

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

    // 注入增强脚本并按当前设置同步滚动开关（同一 eval 避免时序竞态）
    // Inject enhance script and apply sync-scroll in one eval to avoid races
    let _ = use_effect(move || {
        let _ = tab_index;
        let sync = *ui.sync_scroll.read();
        let js = include_str!("../../assets/editor_enhance.js");
        let cm_js = include_str!("../../assets/editor_codemirror.js");
        let _ = document::eval(&format!(
            "{}\n{}\nif (window._mm_initEditor) window._mm_initEditor();\nif (window._mm_upgradeToCodeMirror) window._mm_upgradeToCodeMirror();\nif (window._mm_setSyncScroll) window._mm_setSyncScroll({});",
            js,
            cm_js,
            if sync { "true" } else { "false" }
        ));
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
            ondrop: move |e| {
                *is_dragging.write() = false;
                handle_editor_drop(state, e);
            },
            onkeydown: move |e| {
                if ShortcutActions::handle_event(&mut state, &e) {
                    e.prevent_default();
                }
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
                    title: "{t(\"toggle_preview_shortcut\", lang)}",
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
                if use_uncontrolled {
                    // 大文件非受控：不绑定 value，按键不回写全文到 DOM
                    // Large-file uncontrolled: no value binding; keystrokes avoid full DOM rewrite
                    textarea {
                        class: "editor-textarea",
                        key: "uc-{tab_index}",
                        placeholder: "{placeholder_text}",
                        spellcheck: false,
                        "aria-label": "{aria_editor_t}",
                        "aria-multiline": "true",
                        role: "textbox",
                        onmounted: move |_| {
                            EditorActions::push_to_dom(&cached_content.read());
                        },
                        ondragover: move |e| {
                            e.prevent_default();
                        },
                        ondrop: move |e| {
                            handle_editor_drop(state, e);
                        },
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
                        oninput: move |_| {
                            schedule_uncontrolled_sync(state, sync_gen);
                        },
                        onselect: move |_| {
                            sync_selection_from_dom(&mut state);
                        },
                        onmouseup: move |_| {
                            sync_selection_from_dom(&mut state);
                        },
                    }
                } else {
                    textarea {
                        class: "editor-textarea",
                        key: "c-{tab_index}",
                        value: "{content}",
                        placeholder: "{placeholder_text}",
                        spellcheck: false,
                        "aria-label": "{aria_editor_t}",
                        "aria-multiline": "true",
                        role: "textbox",
                        ondragover: move |e| {
                            e.prevent_default();
                        },
                        ondrop: move |e| {
                            handle_editor_drop(state, e);
                        },
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
                        onselect: move |_| {
                            sync_selection_from_dom(&mut state);
                        },
                        onmouseup: move |_| {
                            sync_selection_from_dom(&mut state);
                        },
                    }
                }
            }
        }
    }
}
