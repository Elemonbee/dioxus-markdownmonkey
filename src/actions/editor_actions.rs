//! 编辑器操作 Actions / Editor Operation Actions
//!
//! 处理文本编辑、撤销重做、格式化等操作
//!
//! 注意：部分功能为预留功能，暂未使用
//! Note: Some functions are reserved for future use, not yet used

use crate::config::{FONT_SIZE_MAX, FONT_SIZE_MIN};
use crate::services::export::{ExportService, HtmlExportOptions};
use crate::services::theme_detector::ThemeDetector;
use crate::state::{AppState, Language, Theme};
use crate::utils::i18n::t;
use dioxus::document;
use dioxus::prelude::{ReadableExt, WritableExt};

/// 编辑器格式化命令 / Editor formatting command
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorFormat {
    Bold,
    Italic,
    Code,
    Link,
    CodeBlock,
    Heading1,
    Heading2,
    Heading3,
    BulletList,
    NumberedList,
    Quote,
    HorizontalRule,
}

/// DOM 编辑器快照，偏移量均为 UTF-8 字节位置
/// DOM editor snapshot whose offsets are UTF-8 byte positions
struct EditorSnapshot {
    value: String,
    start: usize,
    end: usize,
    direction: String,
}

/// 编辑器 Actions 处理器 / Editor Actions Handler
pub struct EditorActions;

impl EditorActions {
    /// 更新内容 / Update Content
    pub fn update_content(state: &mut AppState, content: String) {
        state.update_content(content);
    }

    /// 原子读取 DOM 正文、选区和方向，并将 UTF-16 偏移转换为 UTF-8 字节偏移
    /// Atomically read DOM text, selection and direction, converting UTF-16 offsets to UTF-8 bytes
    async fn read_editor_snapshot() -> Option<EditorSnapshot> {
        let mut eval = document::eval(
            r#"
            (function() {
                if (window._mm_getEditorSnapshot) {
                    dioxus.send(window._mm_getEditorSnapshot());
                    return;
                }
                const ta = document.querySelector('.editor-textarea');
                if (!ta) { dioxus.send(['', 0, 0, 'none']); return; }
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
                dioxus.send([
                    value,
                    toBytes(ta.selectionStart),
                    toBytes(ta.selectionEnd),
                    ta.selectionDirection || 'none'
                ]);
            })();
            "#,
        );
        let (value, start, end, direction) =
            eval.recv::<(String, usize, usize, String)>().await.ok()?;
        Some(EditorSnapshot {
            value,
            start,
            end,
            direction,
        })
    }

    /// 设置光标选区（UTF-8 字节偏移）/ Set cursor selection (UTF-8 byte offsets)
    pub fn set_selection(state: &mut AppState, start: usize, end: usize) {
        let mut ui = state.ui();
        *ui.cursor_start.write() = start;
        *ui.cursor_end.write() = end;
    }

    /// 从 DOM 拉取编辑器正文和字节选区并写入状态
    /// Pull editor text and byte-based selection from the DOM into state
    pub async fn flush_from_dom(state: &mut AppState) {
        if let Some(snapshot) = Self::read_editor_snapshot().await {
            state.update_content(snapshot.value);
            Self::set_selection(state, snapshot.start, snapshot.end);
        }
    }

    /// 格式化命令对应的内核 kind 字符串
    /// Kernel kind string for a formatting command
    fn format_kind(format: EditorFormat) -> &'static str {
        match format {
            EditorFormat::Bold => "bold",
            EditorFormat::Italic => "italic",
            EditorFormat::Code => "code",
            EditorFormat::Link => "link",
            EditorFormat::CodeBlock => "codeblock",
            EditorFormat::Heading1 => "h1",
            EditorFormat::Heading2 => "h2",
            EditorFormat::Heading3 => "h3",
            EditorFormat::BulletList => "bullet",
            EditorFormat::NumberedList => "numbered",
            EditorFormat::Quote => "quote",
            EditorFormat::HorizontalRule => "hr",
        }
    }

    /// 走 CodeMirror 语法树/选区格式化；没有内核时返回 false
    /// Apply format in CodeMirror; return false when the kernel is not mounted
    async fn format_via_codemirror(format: EditorFormat) -> bool {
        let kind =
            serde_json::to_string(Self::format_kind(format)).unwrap_or_else(|_| "\"bold\"".into());
        let mut eval = document::eval(&format!(
            "dioxus.send(!!(window._mm_applyFormat && window._mm_applyFormat({kind})));"
        ));
        eval.recv::<bool>().await.unwrap_or(false)
    }

    /// 走内核插入；没有实例时返回 false
    /// Insert via the kernel; return false when no instance is mounted
    async fn insert_via_codemirror(text: &str) -> bool {
        let safe = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
        let mut eval = document::eval(&format!(
            "dioxus.send(!!(window._mm_insertText && window._mm_insertText({safe})));"
        ));
        eval.recv::<bool>().await.unwrap_or(false)
    }

    /// 通过唯一入口执行格式化；优先内核事务，避免整篇回写
    /// Apply formatting through one entry; prefer an in-kernel transaction over a full rewrite
    pub async fn apply_format(state: &mut AppState, format: EditorFormat) {
        if Self::format_via_codemirror(format).await {
            Self::flush_from_dom(state).await;
            return;
        }

        let direction = if let Some(snapshot) = Self::read_editor_snapshot().await {
            state.update_content(snapshot.value);
            Self::set_selection(state, snapshot.start, snapshot.end);
            snapshot.direction
        } else {
            "none".to_string()
        };

        match format {
            EditorFormat::Bold => Self::insert_bold(state),
            EditorFormat::Italic => Self::insert_italic(state),
            EditorFormat::Code => Self::insert_code(state),
            EditorFormat::Link => Self::insert_link(state),
            EditorFormat::CodeBlock => Self::insert_code_block(state),
            EditorFormat::Heading1 => Self::insert_h1(state),
            EditorFormat::Heading2 => Self::insert_h2(state),
            EditorFormat::Heading3 => Self::insert_h3(state),
            EditorFormat::BulletList => Self::insert_bullet_list(state),
            EditorFormat::NumberedList => Self::insert_numbered_list(state),
            EditorFormat::Quote => Self::insert_quote(state),
            EditorFormat::HorizontalRule => Self::insert_horizontal_rule(state),
        }

        let content = state.document().content.read().clone();
        let ui = state.ui();
        Self::restore_editor(
            &content,
            *ui.cursor_start.read(),
            *ui.cursor_end.read(),
            &direction,
        );
    }

    /// 基于最新 DOM 选区插入文本；优先内核，再同步 Rust
    /// Insert text at the latest DOM selection; prefer the kernel, then sync Rust
    pub async fn insert_text_from_dom(state: &mut AppState, text: &str) {
        if Self::insert_via_codemirror(text).await {
            Self::flush_from_dom(state).await;
            return;
        }
        let direction = if let Some(snapshot) = Self::read_editor_snapshot().await {
            state.update_content(snapshot.value);
            Self::set_selection(state, snapshot.start, snapshot.end);
            snapshot.direction
        } else {
            "none".to_string()
        };
        Self::insert_text(state, text);
        let content = state.document().content.read().clone();
        let ui = state.ui();
        Self::restore_editor(
            &content,
            *ui.cursor_start.read(),
            *ui.cursor_end.read(),
            &direction,
        );
    }

    /// 将 Rust 正文推送到 DOM（非受控模式下撤销/切换标签后调用）
    /// Push Rust content into the DOM (after undo/tab switch in uncontrolled mode)
    pub fn push_to_dom(content: &str) {
        Self::push_editor_to_dom(content, None, &[]);
    }

    /// 推送正文并绑定稳定标签，以便内核恢复光标与撤销栈
    /// Push text and bind a stable tab so the kernel can restore cursor and undo
    pub fn push_editor_to_dom(content: &str, tab_id: Option<u64>, retain_ids: &[u64]) {
        let safe = serde_json::to_string(content).unwrap_or_else(|_| "\"\"".to_string());
        let retain = serde_json::to_string(retain_ids).unwrap_or_else(|_| "[]".to_string());
        let tab = tab_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "null".to_string());
        let _ = document::eval(&format!(
            concat!(
                "window._mm_pendingEditorValue={safe};",
                "window._mm_pendingEditorTabId={tab};",
                "if(window._mm_retainTabStates)window._mm_retainTabStates({retain});",
                "if(window._mm_setEditorValue){{window._mm_setEditorValue({safe},{tab});}}",
                "else{{var ta=document.querySelector('.editor-textarea');if(ta)ta.value={safe};}}"
            ),
            safe = safe,
            tab = tab,
            retain = retain
        ));
    }

    /// 将正文推送到 DOM，并按 UTF-8 字节偏移恢复焦点、选区和方向
    /// Push text to the DOM and restore focus, range and direction from UTF-8 byte offsets
    fn restore_editor(content: &str, start: usize, end: usize, direction: &str) {
        let safe_content = serde_json::to_string(content).unwrap_or_else(|_| "\"\"".to_string());
        let safe_direction =
            serde_json::to_string(direction).unwrap_or_else(|_| "\"none\"".to_string());
        let _ = document::eval(&format!(
            "if(window._mm_setEditorState) window._mm_setEditorState({}, {}, {}, {});",
            safe_content, start, end, safe_direction
        ));
    }

    /// 撤销 / Undo
    pub fn undo(state: &mut AppState) -> bool {
        state.undo()
    }

    /// 重做 / Redo
    pub fn redo(state: &mut AppState) -> bool {
        state.redo()
    }

    /// 走 CodeMirror 历史；没有内核时返回 false
    /// Use the CodeMirror history; return false when the kernel is not mounted
    async fn history_via_codemirror(op: &str) -> bool {
        let safe = serde_json::to_string(op).unwrap_or_else(|_| "\"undo\"".to_string());
        let mut eval = document::eval(&format!(
            "dioxus.send(!!(window._mm_cmHistory && window._mm_cmHistory({safe})));"
        ));
        eval.recv::<bool>().await.unwrap_or(false)
    }

    /// 优先用编辑器内核撤销，再同步回 Rust；无内核时走状态栈
    /// Prefer kernel undo, then sync Rust; fall back to the state stack
    pub async fn undo_via_editor(state: &mut AppState) {
        if Self::history_via_codemirror("undo").await {
            Self::flush_from_dom(state).await;
            return;
        }
        Self::flush_from_dom(state).await;
        Self::undo(state);
    }

    /// 优先用编辑器内核重做，再同步回 Rust；无内核时走状态栈
    /// Prefer kernel redo, then sync Rust; fall back to the state stack
    pub async fn redo_via_editor(state: &mut AppState) {
        if Self::history_via_codemirror("redo").await {
            Self::flush_from_dom(state).await;
            return;
        }
        Self::flush_from_dom(state).await;
        Self::redo(state);
    }

    /// 在选中文本前后插入格式 / Insert format around selected text
    pub fn insert_format(state: &mut AppState, prefix: &str, suffix: &str) {
        state.insert_format_around_selection(prefix, suffix);
    }

    /// 在行首插入前缀 / Insert prefix at line start
    pub fn insert_line_prefix(state: &mut AppState, prefix: &str) {
        state.insert_line_prefix(prefix);
    }

    /// 在光标位置插入文本 / Insert text at cursor position
    pub fn insert_text(state: &mut AppState, text: &str) {
        state.insert_at_cursor(text);
    }

    /// 设置字体大小 / Set Font Size
    pub fn set_font_size(state: &mut AppState, size: u32) {
        *state.ui().font_size.write() = size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX);
    }

    /// 设置预览字体大小 / Set Preview Font Size
    pub fn set_preview_font_size(state: &mut AppState, size: u32) {
        *state.ui().preview_font_size.write() = size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX);
    }

    /// 把自动换行推到内核隔间 / Push word wrap into the kernel compartment
    fn notify_word_wrap(enabled: bool) {
        let _ = document::eval(&format!(
            "if(window._mm_setWordWrap) window._mm_setWordWrap({});",
            if enabled { "true" } else { "false" }
        ));
    }

    /// 把行号显示推到内核隔间 / Push line numbers into the kernel compartment
    fn notify_line_numbers(enabled: bool) {
        let _ = document::eval(&format!(
            "if(window._mm_setLineNumbers) window._mm_setLineNumbers({});",
            if enabled { "true" } else { "false" }
        ));
    }

    /// 工作区 Markdown 相对路径，供 `](` 链接补全
    /// Workspace-relative Markdown paths for `](` link completion
    pub fn workspace_completion_paths(state: &AppState) -> Vec<String> {
        let ui = state.ui();
        let root = ui.workspace_root.read().clone();
        let files = ui.file_list.read().clone();
        files
            .iter()
            .filter_map(|path| {
                if let Some(root) = root.as_ref() {
                    path.strip_prefix(root)
                        .ok()
                        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
                } else {
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                }
            })
            .take(400)
            .collect()
    }

    /// 切换自动换行 / Toggle Word Wrap
    pub fn toggle_word_wrap(state: &mut AppState) {
        let mut ui = state.ui();
        let next = !*ui.word_wrap.read();
        *ui.word_wrap.write() = next;
        Self::notify_word_wrap(next);
    }

    /// 设置自动换行 / Set Word Wrap
    pub fn set_word_wrap(state: &mut AppState, wrap: bool) {
        *state.ui().word_wrap.write() = wrap;
        Self::notify_word_wrap(wrap);
    }

    /// 切换行号显示 / Toggle Line Numbers
    pub fn toggle_line_numbers(state: &mut AppState) {
        let mut ui = state.ui();
        let next = !*ui.line_numbers.read();
        *ui.line_numbers.write() = next;
        Self::notify_line_numbers(next);
    }

    /// 设置行号显示 / Set Line Numbers
    pub fn set_line_numbers(state: &mut AppState, show: bool) {
        *state.ui().line_numbers.write() = show;
        Self::notify_line_numbers(show);
    }

    /// 把同步滚动开关推到 JS（全局标志，跨 textarea 重建仍然有效）
    /// Push the sync-scroll flag to JS (global; survives textarea rebuilds)
    fn notify_sync_scroll(enabled: bool) {
        let _ = document::eval(&format!(
            "if(window._mm_setSyncScroll) window._mm_setSyncScroll({});",
            if enabled { "true" } else { "false" }
        ));
    }

    /// 切换同步滚动 / Toggle Sync Scroll
    pub fn toggle_sync_scroll(state: &mut AppState) {
        let mut ui = state.ui();
        let next = !*ui.sync_scroll.read();
        *ui.sync_scroll.write() = next;
        Self::notify_sync_scroll(next);
    }

    /// 设置同步滚动 / Set Sync Scroll
    pub fn set_sync_scroll(state: &mut AppState, sync: bool) {
        *state.ui().sync_scroll.write() = sync;
        Self::notify_sync_scroll(sync);
    }

    // ========== 格式化快捷方法 / Formatting Shortcut Methods ==========

    /// 插入粗体 / Insert Bold
    pub fn insert_bold(state: &mut AppState) {
        Self::insert_format(state, "**", "**");
    }

    /// 插入斜体 / Insert Italic
    pub fn insert_italic(state: &mut AppState) {
        Self::insert_format(state, "*", "*");
    }

    /// 插入代码 / Insert Code
    pub fn insert_code(state: &mut AppState) {
        Self::insert_format(state, "`", "`");
    }

    /// 插入链接 / Insert Link
    pub fn insert_link(state: &mut AppState) {
        Self::insert_format(state, "[", "](url)");
    }

    /// 插入代码块 / Insert Code Block
    pub fn insert_code_block(state: &mut AppState) {
        Self::insert_format(state, "```\n", "\n```\n");
    }

    /// 插入 H1 标题 / Insert H1 Heading
    pub fn insert_h1(state: &mut AppState) {
        Self::insert_line_prefix(state, "# ");
    }

    /// 插入 H2 标题 / Insert H2 Heading
    pub fn insert_h2(state: &mut AppState) {
        Self::insert_line_prefix(state, "## ");
    }

    /// 插入 H3 标题 / Insert H3 Heading
    pub fn insert_h3(state: &mut AppState) {
        Self::insert_line_prefix(state, "### ");
    }

    /// 插入无序列表 / Insert Bullet List
    pub fn insert_bullet_list(state: &mut AppState) {
        Self::insert_line_prefix(state, "- ");
    }

    /// 插入有序列表 / Insert Numbered List
    pub fn insert_numbered_list(state: &mut AppState) {
        Self::insert_line_prefix(state, "1. ");
    }

    /// 插入引用 / Insert Quote
    pub fn insert_quote(state: &mut AppState) {
        Self::insert_line_prefix(state, "> ");
    }

    /// 插入分割线 / Insert Horizontal Rule
    pub fn insert_horizontal_rule(state: &mut AppState) {
        Self::insert_text(state, "\n---\n");
    }

    /// 打开系统打印对话框（可另存为 PDF）
    /// Open the system print dialog (the user can save as PDF)
    pub async fn print_document(state: &mut AppState) {
        Self::flush_from_dom(state).await;
        let content = state.document().content.read().clone();
        let source_dir = state
            .document()
            .current_file
            .read()
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .or_else(|| state.ui().workspace_root.read().clone());
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
        match ExportService::render_html_for_print(&content, source_dir.as_deref(), &options) {
            Ok(html) => {
                let safe = serde_json::to_string(&html).unwrap_or_else(|_| "\"\"".to_string());
                let _ = document::eval(&format!(
                    "if(window._mm_printHtml)window._mm_printHtml({safe})"
                ));
            }
            Err(e) => {
                tracing::error!("Print failed: {e}");
                let lang = *state.ui().language.read();
                let _ = rfd::MessageDialog::new()
                    .set_title(t("print_failed", lang))
                    .set_description(e.to_string())
                    .set_level(rfd::MessageLevel::Error)
                    .show();
            }
        }
    }
}
