//! 编辑器组件 / Editor Component
//!
//! 正文以 CodeMirror 为准；textarea 只做桥，不绑定受控 value。
//! 小文件按键即同步；大文件防抖 flush，降低按键路径开销。
//! CodeMirror is the source of truth; the textarea is an uncontrolled bridge.
//! Small files sync on input; large files debounce flush.

use crate::actions::shortcut_actions::ShortcutActions;
use crate::actions::{AppActions, EditorActions, FileActions};
use crate::components::icons::PreviewIcon;
use crate::config::{
    EDITOR_LINE_HEIGHT_PX, EDITOR_VIRTUAL_SCROLL_BUFFER_LINES,
    EDITOR_VIRTUAL_SCROLL_THRESHOLD_LINES, UNCONTROLLED_EDITOR_SYNC_DEBOUNCE_MS,
    UNCONTROLLED_EDITOR_THRESHOLD_BYTES,
};
use crate::services::ai::selected_ai_context;
use crate::state::{AppState, Language};
use crate::utils::i18n::t;
use dioxus::document;
use dioxus::html::HasFileData;
use dioxus::prelude::{ReadableExt, WritableExt, *};
use std::path::PathBuf;

/// 若 `<script src>` 未就绪，在同一 eval 内联注入 CodeMirror 6，避免空白编辑区
/// Inline CodeMirror 6 in the same eval if the script tag is not ready, avoiding a blank editor
const CODEMIRROR_BOOT: &str = concat!(
    "if(!window.MarkdownMonkeyCM){\n",
    include_str!("../../assets/vendor/codemirror6.bundle.js"),
    "\n}\n"
);

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

/// 从右键菜单启动预设 AI 任务 / Launch a preset AI task from the context menu
fn launch_editor_ai_preset(
    mut state: AppState,
    task_id: &'static str,
    translate_target: Option<Language>,
) {
    spawn(async move {
        let lang = *state.ui().language.read();
        let title_error = t("ai_error", lang);
        let error_prefix = t("error", lang);
        AppActions::run_editor_ai_preset(
            &mut state,
            task_id.to_string(),
            translate_target,
            title_error,
            error_prefix,
        )
        .await;
    });
}

/// 从右键菜单执行剪贴板动作 / Run a clipboard action from the context menu
fn launch_clipboard(mut state: AppState, action: &'static str) {
    spawn(async move {
        EditorActions::exec_clipboard(&mut state, action).await;
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
    let mut show_editor_menu = use_signal(|| false);
    let mut editor_menu_pos = use_signal(|| (0_i32, 0_i32));
    let mut show_ai_submenu = use_signal(|| false);
    let mut menu_has_selection = use_signal(|| false);

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
    let tab_id = doc
        .tabs
        .read()
        .get(tab_index)
        .map(|tab| tab.id)
        .unwrap_or(0);
    let retain_ids: Vec<u64> = doc.tabs.read().iter().map(|tab| tab.id).collect();
    let file_size = (*doc.file_size_bytes.read()).max(content.len());
    let use_uncontrolled = file_size >= UNCONTROLLED_EDITOR_THRESHOLD_BYTES;

    // CodeMirror 与受控/非受控共用：修订变化时把 Rust 正文推回 DOM（JS 侧相同则跳过）
    // Shared by CodeMirror and both editor modes: push Rust text on revision; JS no-ops if equal
    if rev != *last_pushed_rev.read() {
        last_pushed_rev.set(rev);
        EditorActions::push_editor_to_dom(&content, Some(tab_id), &retain_ids);
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
    let copy_t = t("copy", lang);
    let cut_t = t("cut", lang);
    let paste_t = t("paste", lang);
    let ai_title_t = t("ai_assistant", lang);
    let continue_t = t("ai_continue", lang);
    let improve_t = t("ai_improve", lang);
    let outline_t = t("ai_outline", lang);
    let grammar_t = t("ai_fix_grammar", lang);
    let translate_en_t = t("ai_translate_to_en", lang);
    let translate_zh_t = t("ai_translate_to_zh", lang);
    let need_selection_t = t("ai_need_selection", lang);

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

    // 脚本只装一次；切标签靠 push_editor_to_dom，避免每次把 500KB+ bundle 再 eval 一遍
    // Install scripts once; tab switches use push_editor_to_dom so the 500KB+ bundle is not re-eval'd
    let mut boot_ready = use_signal(|| false);
    let _ = use_effect(move || {
        if *boot_ready.peek() {
            return;
        }
        boot_ready.set(true);
        let js = include_str!("../../assets/editor_enhance.js");
        let cm_js = include_str!("../../assets/editor_codemirror.js");
        let print_js = include_str!("../../assets/print.js");
        let _ = document::eval(&format!(
            "{boot}\n\
             if(!window._mm_enhanceReady){{{enhance}\nwindow._mm_enhanceReady=true;}}\n\
             if(!window._mm_cmUpgradeInstalled){{{cm}}}\n\
             if(!window._mm_printHtml){{{print}}}\n\
             if(window._mm_initEditor)window._mm_initEditor();\n\
             if(window._mm_upgradeToCodeMirror)window._mm_upgradeToCodeMirror();",
            boot = CODEMIRROR_BOOT,
            enhance = js,
            cm = cm_js,
            print = print_js
        ));
    });

    // 换行 / 行号 / 同步滚动 / 工作区补全跟设置走，不重装内核
    // Wrap / line numbers / sync scroll / workspace completion follow settings without remounting
    let _ = use_effect(move || {
        let wrap = *ui.word_wrap.read();
        let lines = *ui.line_numbers.read();
        let sync = *ui.sync_scroll.read();
        let files = serde_json::to_string(&EditorActions::workspace_completion_paths(&state))
            .unwrap_or_else(|_| "[]".to_string());
        let _ = document::eval(&format!(
            "if(window._mm_setWordWrap)window._mm_setWordWrap({wrap});\
             if(window._mm_setLineNumbers)window._mm_setLineNumbers({lines});\
             if(window._mm_setSyncScroll)window._mm_setSyncScroll({sync});\
             if(window._mm_setWorkspaceFiles)window._mm_setWorkspaceFiles({files});",
            wrap = if wrap { "true" } else { "false" },
            lines = if lines { "true" } else { "false" },
            sync = if sync { "true" } else { "false" },
            files = files
        ));
    });

    let editor_menu_style = {
        let (mx, my) = *editor_menu_pos.read();
        format!("left: {mx}px; top: {my}px;")
    };
    let editor_menu_open = *show_editor_menu.read();
    let ai_submenu_open = *show_ai_submenu.read();
    let editor_has_sel = *menu_has_selection.read();

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
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                let coords = e.client_coordinates();
                editor_menu_pos.set((coords.x as i32, coords.y as i32));
                show_ai_submenu.set(false);
                let start = *ui.cursor_start.read();
                let end = *ui.cursor_end.read();
                menu_has_selection
                    .set(selected_ai_context(&cached_content.read(), start, end).is_some());
                show_editor_menu.set(true);
                let mut state = state;
                spawn(async move {
                    EditorActions::flush_from_dom(&mut state).await;
                    let content = state.document().content.read().clone();
                    let start = *state.ui().cursor_start.read();
                    let end = *state.ui().cursor_end.read();
                    menu_has_selection.set(selected_ai_context(&content, start, end).is_some());
                });
            },
            onkeydown: move |e| {
                if *show_editor_menu.read() && e.key() == Key::Escape {
                    show_editor_menu.set(false);
                    show_ai_submenu.set(false);
                    e.prevent_default();
                    return;
                }
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
                textarea {
                    class: "editor-textarea",
                    placeholder: "{placeholder_text}",
                    spellcheck: false,
                    "aria-label": "{aria_editor_t}",
                    "aria-multiline": "true",
                    role: "textbox",
                    onmounted: move |_| {
                        EditorActions::push_editor_to_dom(
                            &cached_content.read(),
                            Some(tab_id),
                            &retain_ids,
                        );
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
                        // 只信内核/DOM 快照，不信 Dioxus 对隐藏 textarea 的 e.value()
                        // Trust the kernel/DOM snapshot, not Dioxus e.value() on a hidden textarea
                        if use_uncontrolled {
                            schedule_uncontrolled_sync(state, sync_gen);
                        } else {
                            let mut state = state;
                            spawn(async move {
                                EditorActions::flush_from_dom(&mut state).await;
                            });
                        }
                    },
                    onselect: move |_| {
                        sync_selection_from_dom(&mut state);
                    },
                    onmouseup: move |_| {
                        sync_selection_from_dom(&mut state);
                    },
                }
            }

            if editor_menu_open {
                div {
                    class: "context-menu-overlay",
                    onclick: move |_| {
                        show_editor_menu.set(false);
                        show_ai_submenu.set(false);
                    },
                    oncontextmenu: move |e| {
                        e.prevent_default();
                        show_editor_menu.set(false);
                        show_ai_submenu.set(false);
                    },
                }
                div {
                    class: "context-menu",
                    style: "{editor_menu_style}",

                    button {
                        class: "context-menu-item",
                        onclick: move |e| {
                            e.stop_propagation();
                            show_editor_menu.set(false);
                            show_ai_submenu.set(false);
                            launch_clipboard(state, "copy");
                        },
                        "{copy_t}"
                    }
                    button {
                        class: "context-menu-item",
                        onclick: move |e| {
                            e.stop_propagation();
                            show_editor_menu.set(false);
                            show_ai_submenu.set(false);
                            launch_clipboard(state, "cut");
                        },
                        "{cut_t}"
                    }
                    button {
                        class: "context-menu-item",
                        onclick: move |e| {
                            e.stop_propagation();
                            show_editor_menu.set(false);
                            show_ai_submenu.set(false);
                            launch_clipboard(state, "paste");
                        },
                        "{paste_t}"
                    }

                    div { class: "context-menu-divider" }

                    div { class: "context-menu-submenu-host",
                        button {
                            class: if editor_has_sel { "context-menu-item has-submenu" } else { "context-menu-item has-submenu disabled" },
                            title: if editor_has_sel { String::new() } else { need_selection_t.clone() },
                            onclick: move |e| {
                                e.stop_propagation();
                                if editor_has_sel {
                                    let open = *show_ai_submenu.read();
                                    show_ai_submenu.set(!open);
                                }
                            },
                            span { "{ai_title_t}" }
                            span { class: "context-menu-caret", "›" }
                        }
                        if ai_submenu_open && editor_has_sel {
                            div { class: "context-submenu",
                                button {
                                    class: "context-menu-item",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        show_editor_menu.set(false);
                                        show_ai_submenu.set(false);
                                        launch_editor_ai_preset(state, "continue", None);
                                    },
                                    "{continue_t}"
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        show_editor_menu.set(false);
                                        show_ai_submenu.set(false);
                                        launch_editor_ai_preset(state, "improve", None);
                                    },
                                    "{improve_t}"
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        show_editor_menu.set(false);
                                        show_ai_submenu.set(false);
                                        launch_editor_ai_preset(state, "outline", None);
                                    },
                                    "{outline_t}"
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        show_editor_menu.set(false);
                                        show_ai_submenu.set(false);
                                        launch_editor_ai_preset(
                                            state,
                                            "translate",
                                            Some(Language::EnUS),
                                        );
                                    },
                                    "{translate_en_t}"
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        show_editor_menu.set(false);
                                        show_ai_submenu.set(false);
                                        launch_editor_ai_preset(
                                            state,
                                            "translate",
                                            Some(Language::ZhCN),
                                        );
                                    },
                                    "{translate_zh_t}"
                                }
                                button {
                                    class: "context-menu-item",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        show_editor_menu.set(false);
                                        show_ai_submenu.set(false);
                                        launch_editor_ai_preset(state, "fix_grammar", None);
                                    },
                                    "{grammar_t}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    /// 打包的 IIFE 必须暴露 MarkdownMonkeyCM，供桌面 WebView 挂载
    /// The vendored IIFE must expose MarkdownMonkeyCM for the desktop WebView
    #[test]
    fn vendor_bundle_exports_markdownmonkey_cm() {
        let bundle = include_str!("../../assets/vendor/codemirror6.bundle.js");
        assert!(bundle.contains("MarkdownMonkeyCM"));
        assert!(!bundle.contains("window.CodeMirror.fromTextArea"));
    }

    /// 升级层走 CodeMirror 6 API，不再调用 5 的 fromTextArea
    /// The upgrade layer uses CodeMirror 6 APIs and no longer calls fromTextArea
    #[test]
    fn editor_bridge_targets_codemirror_6() {
        let bridge = include_str!("../../assets/editor_codemirror.js");
        assert!(bridge.contains("MarkdownMonkeyCM"));
        assert!(bridge.contains("EditorView"));
        assert!(bridge.contains("_mm_applyFormat"));
        assert!(bridge.contains("_mm_setWordWrap"));
        assert!(bridge.contains("_mm_retainTabStates"));
        assert!(bridge.contains("_mm_clipboardAction"));
        assert!(!bridge.contains("fromTextArea"));
        assert!(!bridge.contains("material-darker"));
    }

    /// 内核入口包含折叠、围栏着色、任务框和补全
    /// Kernel entry includes folding, fenced highlighting, task boxes, and completion
    #[test]
    fn kernel_entry_exports_cm6_editor_features() {
        let src = include_str!("../../scripts/codemirror/src/index.js");
        assert!(src.contains("codeLanguages"));
        assert!(src.contains("foldGutter"));
        assert!(src.contains("taskCheckbox"));
        assert!(src.contains("markdownAutocompletion"));
        assert!(src.contains("applyMarkdownFormat"));
    }
}
