//! 工具栏组件 / Toolbar Component
//!
//! 使用 AsyncFileDialog 避免文件对话框阻塞 UI

use crate::actions::{AppActions, EditorActions, FileActions};
use crate::components::icons::*;
use crate::services::export::ExportService;
use crate::services::recent_files::RecentFiles;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::{ReadableExt, WritableExt, *};
use rfd::AsyncFileDialog;

/// 工具栏组件 / Toolbar Component
#[component]
pub fn Toolbar() -> Element {
    let mut state = use_context::<AppState>();

    let show_sidebar = *state.sidebar_visible.read();
    let show_preview = *state.show_preview.read();
    let lang = *state.language.read();

    // i18n
    let new_file_t = t("new_file", lang);
    let open_file_t = t("open_file", lang);
    let save_file_t = t("save_file", lang);
    let export_html_t = t("export_html", lang);
    let export_pdf_t = t("export_pdf", lang);
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
    let trigger_save_as = *state.trigger_save_as.read();
    use_effect(move || {
        if trigger_save_as {
            *state.trigger_save_as.write() = false;
            let mut state = state;
            spawn(async move {
                let file = AsyncFileDialog::new()
                    .add_filter("Markdown", &["md"])
                    .save_file()
                    .await;
                if let Some(file) = file {
                    let path = file.path().to_path_buf();
                    if let Err(e) = FileActions::save_as(&mut state, path) {
                        tracing::error!("Save As failed: {}", e);
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
    let auto_save_title = if *state.auto_save_enabled.read() {
        auto_on_t.clone()
    } else {
        auto_off_t.clone()
    };
    let auto_save_class = if *state.auto_save_enabled.read() {
        "toolbar-btn active"
    } else {
        "toolbar-btn"
    };

    rsx! {
        div { class: "toolbar",
            // 文件操作 / File Operations
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{new_file_t} (Ctrl+N)",
                    onclick: move |_| { FileActions::new_tab(&mut state); },
                    NewFileIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{open_file_t} (Ctrl+O)",
                    onclick: move |_| {
                        let mut state = state;
                        spawn(async move {
                            let file = AsyncFileDialog::new()
                                .add_filter("Markdown", &["md", "markdown", "txt"])
                                .pick_file()
                                .await;
                            if let Some(file) = file {
                                let path = file.path().to_path_buf();
                                if FileActions::open_file(&mut state, path.clone()).is_ok() {
                                    let mut recent = RecentFiles::load();
                                    recent.add(path);
                                    let _ = recent.save();
                                }
                            }
                        });
                    },
                    OpenFileIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{save_file_t} (Ctrl+S)",
                    onclick: move |_| {
                        if state.current_file.read().is_some() {
                            if let Err(e) = FileActions::save_current_file(&mut state) {
                                tracing::error!("Save failed: {}", e);
                            }
                        } else {
                            let mut state = state;
                            spawn(async move {
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

            // 导出 / Export
            div { class: "toolbar-group toolbar-export-group",
                button {
                    class: "toolbar-btn",
                    title: "{export_html_t}",
                    onclick: move |_| {
                        let content = state.content.read().clone();
                        spawn(async move {
                            let file = AsyncFileDialog::new()
                                .add_filter("HTML", &["html"])
                                .save_file()
                                .await;
                            if let Some(file) = file {
                                let path = file.path().to_path_buf();
                                let path = if path.extension().is_none() {
                                    let mut p = path;
                                    p.set_extension("html");
                                    p
                                } else { path };

                                if let Err(e) = ExportService::export_to_html(&content, &path) {
                                    tracing::error!("Export failed: {}", e);
                                }
                            }
                        });
                    },
                    "HTML"
                }
                button {
                    class: "toolbar-btn",
                    title: "{export_pdf_t}",
                    onclick: move |_| {
                        let content = state.content.read().clone();
                        spawn(async move {
                            let file = AsyncFileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .save_file()
                                .await;
                            if let Some(file) = file {
                                let path = file.path().to_path_buf();
                                let path = if path.extension().is_none() {
                                    let mut p = path;
                                    p.set_extension("pdf");
                                    p
                                } else { path };

                                if let Err(e) = ExportService::export_to_pdf(&content, &path) {
                                    tracing::error!("Export failed: {}", e);
                                }
                            }
                        });
                    },
                    "PDF"
                }
            }

            div { class: "toolbar-divider" }

            // 编辑操作 / Edit Operations
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{undo_t} (Ctrl+Z)",
                    onclick: move |_| { EditorActions::undo(&mut state); },
                    UndoIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{redo_t} (Ctrl+Y)",
                    onclick: move |_| { EditorActions::redo(&mut state); },
                    RedoIcon { size: 18 }
                }
            }

            div { class: "toolbar-divider" }

            // 格式化 / Formatting
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{bold_t} (Ctrl+B)",
                    onclick: move |_| { EditorActions::insert_bold(&mut state); },
                    BoldIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{italic_t} (Ctrl+I)",
                    onclick: move |_| { EditorActions::insert_italic(&mut state); },
                    ItalicIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{code_t} (Ctrl+`)",
                    onclick: move |_| { EditorActions::insert_code(&mut state); },
                    CodeIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{link_t} (Ctrl+K)",
                    onclick: move |_| { EditorActions::insert_link(&mut state); },
                    LinkIcon { size: 18 }
                }
            }

            div { class: "toolbar-divider" }

            // 标题 / Headings
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn heading-btn",
                    title: "{h1_t}",
                    onclick: move |_| { EditorActions::insert_h1(&mut state); },
                    "H1"
                }
                button {
                    class: "toolbar-btn heading-btn",
                    title: "{h2_t}",
                    onclick: move |_| { EditorActions::insert_h2(&mut state); },
                    "H2"
                }
                button {
                    class: "toolbar-btn heading-btn",
                    title: "{h3_t}",
                    onclick: move |_| { EditorActions::insert_h3(&mut state); },
                    "H3"
                }
            }

            div { class: "toolbar-divider" }

            // 列表 / Lists
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{bullet_t}",
                    onclick: move |_| { EditorActions::insert_bullet_list(&mut state); },
                    ListIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{numbered_t}",
                    onclick: move |_| { EditorActions::insert_numbered_list(&mut state); },
                    OrderedListIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{quote_t}",
                    onclick: move |_| { EditorActions::insert_quote(&mut state); },
                    QuoteIcon { size: 18 }
                }
            }

            div { class: "toolbar-divider" }

            // 插入 / Insert
            div { class: "toolbar-group",
                button {
                    class: "toolbar-btn",
                    title: "{table_t}",
                    onclick: move |_| { *state.show_table_editor.write() = true; },
                    TableIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{code_block_t}",
                    onclick: move |_| { EditorActions::insert_code_block(&mut state); },
                    CodeIcon { size: 18 }
                }
                button {
                    class: "toolbar-btn",
                    title: "{hr_t}",
                    onclick: move |_| { EditorActions::insert_horizontal_rule(&mut state); },
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
                                EditorActions::insert_text(&mut state, &img_markdown);
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
                    title: "{toggle_sidebar_t} (Ctrl+\\)",
                    onclick: move |_| { AppActions::toggle_sidebar(&mut state); },
                    SidebarIcon { size: 18 }
                }
                button {
                    class: "{preview_class}",
                    title: "{toggle_preview_t} (Ctrl+P)",
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
                    onclick: move |_| {
                        let enabled = !*state.auto_save_enabled.read();
                        *state.auto_save_enabled.write() = enabled;
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
