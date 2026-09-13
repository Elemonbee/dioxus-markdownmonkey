//! 工具栏组件 / Toolbar Component
//!
//! 使用 AsyncFileDialog 避免文件对话框阻塞 UI

use crate::actions::shortcut_actions::ShortcutActions;
use crate::actions::{AppActions, EditorActions, EditorFormat, FileActions};
use crate::components::icons::*;
use crate::services::export::{ExportService, HtmlExportOptions};
use crate::services::theme_detector::ThemeDetector;
use crate::state::{AppState, Language, Theme};
use crate::utils::i18n::t;
use dioxus::prelude::{ReadableExt, *};
use rfd::AsyncFileDialog;

/// 显示导出失败对话框 / Show export failure dialog
fn show_export_error(title: &str, err: impl std::fmt::Display) {
    tracing::error!("Export failed: {}", err);
    let msg = err.to_string();
    let _ = rfd::MessageDialog::new()
        .set_title(title)
        .set_description(&msg)
        .set_level(rfd::MessageLevel::Error)
        .show();
}

/// 导出格式 / Export format
#[derive(Clone, Copy)]
enum ExportKind {
    Html,
    Text,
}

impl ExportKind {
    /// 对话框过滤器名称与扩展名 / Dialog filter name and extension
    fn filter(self) -> (&'static str, &'static str) {
        match self {
            Self::Html => ("HTML", "html"),
            Self::Text => ("Text", "txt"),
        }
    }
}

/// 统一导出流程：flush → 选路径 → 后台写出
/// Unified export flow: flush → pick path → write off UI thread
fn run_export(mut state: AppState, kind: ExportKind, err_title: String) {
    spawn(async move {
        EditorActions::flush_from_dom(&mut state).await;
        let content = state.document().content.read().clone();
        let source_dir = state
            .document()
            .current_file
            .read()
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .or_else(|| state.ui().workspace_root.read().clone());
        let (filter_name, ext) = kind.filter();
        let file = AsyncFileDialog::new()
            .add_filter(filter_name, &[ext])
            .save_file()
            .await;
        let Some(file) = file else {
            return;
        };
        let path = file.path().to_path_buf();
        let path = if path.extension().is_none() {
            let mut p = path;
            p.set_extension(ext);
            p
        } else {
            path
        };

        let language = match *state.ui().language.read() {
            Language::EnUS => "en-US",
            Language::ZhCN => "zh-CN",
        }
        .to_string();
        let dark = match *state.ui().theme.read() {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::System => ThemeDetector::detect() == "dark",
        };
        let options = HtmlExportOptions { language, dark };

        let result = tokio::task::spawn_blocking(move || match kind {
            ExportKind::Html => ExportService::export_to_html_with_options(
                &content,
                &path,
                source_dir.as_deref(),
                &options,
            ),
            ExportKind::Text => ExportService::export_to_text(&content, &path),
        })
        .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => show_export_error(&err_title, e),
            Err(e) => show_export_error(&err_title, e),
        }
    });
}

/// 从工具栏调度统一格式化入口 / Dispatch the shared formatting entry point from the toolbar
fn run_format(mut state: AppState, format: EditorFormat) {
    spawn(async move {
        EditorActions::apply_format(&mut state, format).await;
    });
}

/// 工具栏组件 / Toolbar Component
#[component]
pub fn Toolbar() -> Element {
    let mut state = use_context::<AppState>();
    let doc = state.document();
    let ui = state.ui();

    let show_sidebar = *ui.sidebar_visible.read();
    let show_preview = *ui.show_preview.read();
    let lang = *ui.language.read();

    // i18n
    let new_file_t = t("new_file", lang);
    let open_file_t = t("open_file", lang);
    let save_file_t = t("save_file", lang);
    let export_html_t = t("export_html", lang);
    let export_text_t = t("export_text", lang);
    let export_print_t = t("export_print", lang);
    let export_menu_t = t("export", lang);
    let export_failed_t = t("export_failed", lang);
    let aria_toolbar_t = t("aria_toolbar", lang);
    let undo_t = t("undo", lang);
    let redo_t = t("redo", lang);
    let bold_t = t("bold", lang);
    let italic_t = t("italic", lang);
    let code_t = t("code", lang);
    let link_t = t("link", lang);
    let h1_t = t("heading_1", lang);
    let h2_t = t("heading_2", lang);
    let h3_t = t("heading_3", lang);
    let bullet_t = t("bullet_list", lang);
    let numbered_t = t("numbered_list", lang);
    let quote_t = t("quote", lang);
    let table_t = t("table_editor_btn", lang);
    let code_block_t = t("code_block", lang);
    let hr_t = t("horizontal_rule", lang);
    let image_t = t("image", lang);
    let toggle_sidebar_t = t("toggle_sidebar", lang);
    let toggle_preview_t = t("toggle_preview", lang);
    let theme_t = t("theme", lang);
    let auto_on_t = t("auto_save_on", lang);
    let auto_off_t = t("auto_save_off", lang);
    let ai_t = t("ai_assistant", lang);

    // 监听 trigger_save_as 信号，触发另存为对话框 / Listen for trigger_save_as signal
    let trigger_save_as = *doc.trigger_save_as.read();
    use_effect(move || {
        if trigger_save_as {
            let close_intent = FileActions::consume_save_as_trigger(&mut state);
            let is_close_intent = close_intent.is_some();
            let mut state = state;
            spawn(async move {
                let file = AsyncFileDialog::new()
                    .add_filter("Markdown", &["md"])
                    .save_file()
                    .await;
                if let Some(file) = file {
                    let path = file.path().to_path_buf();
                    let result = if let Some(snapshot) = close_intent {
                        FileActions::save_close_snapshot_as(&mut state, snapshot, path)
                    } else {
                        FileActions::save_as(&mut state, path)
                    };
                    if let Err(e) = result {
                        tracing::error!("Save As failed: {}", e);
                        if is_close_intent {
                            FileActions::cancel_close_request(&mut state);
                        }
                    }
                } else {
                    // 用户取消关闭型另存为时保留目标标签 / Keep target tab when close Save-As is canceled
                    if is_close_intent {
                        FileActions::cancel_close_request(&mut state);
                    }
                }
            });
        }
    });
    let settings_t = t("settings", lang);
    let language_t = t("language", lang);

    let sidebar_class = if show_sidebar {
        "toolbar-btn active"
    } else {
        "toolbar-btn"
    };
    let preview_class = if show_preview {
        "toolbar-btn active"
    } else {
        "toolbar-btn"
    };
    let auto_save_title = if *ui.auto_save_enabled.read() {
        auto_on_t.clone()
    } else {
        auto_off_t.clone()
    };
    let auto_save_class = if *ui.auto_save_enabled.read() {
        "toolbar-btn active"
    } else {
        "toolbar-btn"
    };
    let modifier = ShortcutActions::primary_modifier_label();
    let mut export_menu_open = use_signal(|| false);
    let export_trigger_class = if *export_menu_open.read() {
        "toolbar-btn toolbar-export-trigger active"
    } else {
        "toolbar-btn toolbar-export-trigger"
    };

    rsx! {
        div { class: "toolbar", role: "toolbar", "aria-label": "{aria_toolbar_t}",
            // 文件操作 / File Operations
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{new_file_t} ({modifier}+N)",
                    onclick: move |_| {
                        let mut state = state;
                        spawn(async move {
                            FileActions::new_tab_flushed(&mut state).await;
                        });
                    },
                    NewFileIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{open_file_t} ({modifier}+O)",
                    onclick: move |_| {
                        let mut state = state;
                        spawn(async move {
                            if let Err(e) = FileActions::open_file_dialog(&mut state).await {
                                tracing::warn!("Open file failed: {}", e);
                            }
                        });
                    },
                    OpenFileIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{save_file_t} ({modifier}+S)",
                    onclick: move |_| {
                        if doc.current_file.read().is_some() {
                            let mut state = state;
                            spawn(async move {
                                EditorActions::flush_from_dom(&mut state).await;
                                if let Err(e) =
                                    FileActions::save_current_file_async(&mut state).await
                                {
                                    tracing::error!("Save failed: {}", e);
                                }
                            });
                        } else {
                            let mut state = state;
                            spawn(async move {
                                EditorActions::flush_from_dom(&mut state).await;
                                let file = AsyncFileDialog::new()
                                    .add_filter("Markdown", &["md"])
                                    .save_file()
                                    .await;
                                if let Some(file) = file {
                                    let path = file.path().to_path_buf();
                                    if let Err(e) = FileActions::save_as(&mut state, path) {
                                        tracing::error!("Save failed: {}", e);
                                    }
                                }
                            });
                        }
                    },
                    SaveIcon { size: 18 }
                }
            }

            // 导出下拉菜单 / Export dropdown
            div { class: "toolbar-group toolbar-export-group",
                div { class: "toolbar-dropdown",
                    button {
                        class: "{export_trigger_class}",
                        title: "{export_menu_t}",
                        "aria-haspopup": "menu",
                        "aria-expanded": "{*export_menu_open.read()}",
                        onclick: move |_| {
                            let next = !*export_menu_open.read();
                            export_menu_open.set(next);
                        },
                        ExportIcon { size: 18 }
                        span { class: "toolbar-export-label", "{export_menu_t}" }
                        ChevronDownIcon { size: 14 }
                    }
                    if *export_menu_open.read() {
                        // 点击外部关闭 / Click-outside to close
                        div {
                            class: "toolbar-dropdown-overlay",
                            onclick: move |_| {
                                export_menu_open.set(false);
                            },
                            onkeydown: move |e| {
                                if e.key().to_string() == "Escape" {
                                    export_menu_open.set(false);
                                    e.prevent_default();
                                }
                            },
                        }
                        div {
                            class: "toolbar-dropdown-menu",
                            role: "menu",
                            button {
                                class: "toolbar-dropdown-item",
                                role: "menuitem",
                                onclick: {
                                    let err_title = export_failed_t.clone();
                                    move |_| {
                                        export_menu_open.set(false);
                                        run_export(state, ExportKind::Html, err_title.clone());
                                    }
                                },
                                "{export_html_t}"
                            }
                            button {
                                class: "toolbar-dropdown-item",
                                role: "menuitem",
                                onclick: {
                                    let err_title = export_failed_t.clone();
                                    move |_| {
                                        export_menu_open.set(false);
                                        run_export(state, ExportKind::Text, err_title.clone());
                                    }
                                },
                                "{export_text_t}"
                            }
                            button {
                                class: "toolbar-dropdown-item",
                                role: "menuitem",
                                onclick: move |_| {
                                    export_menu_open.set(false);
                                    spawn(async move {
                                        EditorActions::print_document(&mut state).await;
                                    });
                                },
                                "{export_print_t}"
                            }
                        }
                    }
                }
            }

            div { class: "toolbar-divider" }

            // 编辑操作 / Edit Operations
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{undo_t} ({modifier}+Z)",
                    onclick: move |_| {
                        let mut state = state;
                        spawn(async move {
                            EditorActions::undo_via_editor(&mut state).await;
                        });
                    },
                    UndoIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{redo_t} ({modifier}+Y)",
                    onclick: move |_| {
                        let mut state = state;
                        spawn(async move {
                            EditorActions::redo_via_editor(&mut state).await;
                        });
                    },
                    RedoIcon { size: 18 }
                }
            }

            div { class: "toolbar-divider" }

            // 格式化 / Formatting（非受控模式先 flush 再改写并推回 DOM）
            // Formatting (flush then rewrite + push DOM in uncontrolled mode)
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{bold_t} ({modifier}+B)",
                    onclick: move |_| {
                        run_format(state, EditorFormat::Bold);
                    },
                    BoldIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{italic_t} ({modifier}+I)",
                    onclick: move |_| {
                        run_format(state, EditorFormat::Italic);
                    },
                    ItalicIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{code_t} ({modifier}+`)",
                    onclick: move |_| {
                        run_format(state, EditorFormat::Code);
                    },
                    CodeIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{link_t} ({modifier}+K)",
                    onclick: move |_| {
                        run_format(state, EditorFormat::Link);
                    },
                    LinkIcon { size: 18 }
                }
            }

            div { class: "toolbar-divider" }

            // 标题 / Headings
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn heading-btn",
                    title: "{h1_t}",
                    onclick: move |_| {
                        run_format(state, EditorFormat::Heading1);
                    },
                    "H1"
                }
                button {
                    class: "toolbar-btn heading-btn",
                    title: "{h2_t}",
                    onclick: move |_| {
                        run_format(state, EditorFormat::Heading2);
                    },
                    "H2"
                }
                button {
                    class: "toolbar-btn heading-btn",
                    title: "{h3_t}",
                    onclick: move |_| {
                        run_format(state, EditorFormat::Heading3);
                    },
                    "H3"
                }
            }

            div { class: "toolbar-divider" }

            // 列表 / Lists
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{bullet_t}",
                    onclick: move |_| {
                        run_format(state, EditorFormat::BulletList);
                    },
                    ListIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{numbered_t}",
                    onclick: move |_| {
                        run_format(state, EditorFormat::NumberedList);
                    },
                    OrderedListIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{quote_t}",
                    onclick: move |_| {
                        run_format(state, EditorFormat::Quote);
                    },
                    QuoteIcon { size: 18 }
                }
            }

            div { class: "toolbar-divider" }

            // 插入 / Insert
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{table_t}",
                    onclick: move |_| { AppActions::show_table_editor(&mut state); },
                    TableIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{code_block_t}",
                    onclick: move |_| {
                        run_format(state, EditorFormat::CodeBlock);
                    },
                    CodeIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{hr_t}",
                    onclick: move |_| {
                        run_format(state, EditorFormat::HorizontalRule);
                    },
                    DividerIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{image_t}",
                    onclick: move |_| {
                        let mut state = state;
                        let image_t = image_t.clone();
                        spawn(async move {
                            let file = AsyncFileDialog::new()
                                .add_filter(&image_t, &["png", "jpg", "jpeg", "gif", "webp"])
                                .pick_file()
                                .await;
                            if let Some(file) = file {
                                let path = file.path().to_path_buf();
                                let img_markdown = format!("![{image_t}]({})", path.display());
                                EditorActions::insert_text_from_dom(&mut state, &img_markdown).await;
                            }
                        });
                    },
                    ImageIcon { size: 18 }
                }
            }

            div { class: "toolbar-spacer" }

            // 视图控制 / View Controls
            div { class: "toolbar-group",
                button {
                    class: "{sidebar_class}",
                    title: "{toggle_sidebar_t} ({modifier}+\\)",
                    aria_pressed: "{show_sidebar}",
                    onclick: move |_| { AppActions::toggle_sidebar(&mut state); },
                    SidebarIcon { size: 18 }
                }
                button {
                    class: "{preview_class}",
                    title: "{toggle_preview_t} ({modifier}+P)",
                    aria_pressed: "{show_preview}",
                    onclick: move |_| { AppActions::toggle_preview(&mut state); },
                    PreviewIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{theme_t}",
                    onclick: move |_| { AppActions::toggle_theme(&mut state); },
                    ThemeIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{language_t}",
                    onclick: move |_| { AppActions::toggle_language(&mut state); },
                    LanguageIcon { size: 18 }
                }
                button {
                    class: "{auto_save_class}",
                    title: "{auto_save_title}",
                    aria_pressed: "{*ui.auto_save_enabled.read()}",
                    onclick: move |_| {
                        AppActions::toggle_auto_save(&mut state);
                    },
                    SaveIcon { size: 18 }
                }
            }

            div { class: "toolbar-divider" }

            // AI 助手 / AI Assistant
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{ai_t}",
                    onclick: move |_| { AppActions::show_ai_chat(&mut state); },
                    AIIcon { size: 18 }
                }

                button {
                    class: "toolbar-btn",
                    title: "{settings_t}",
                    onclick: move |_| { AppActions::show_settings(&mut state); },
                    SettingsIcon { size: 18 }
                }
            }
        }
    }
}
