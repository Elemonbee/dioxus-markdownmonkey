//! 编辑器操作 Actions / Editor Operation Actions
//!
//! 处理文本编辑、撤销重做、格式化等操作
//!
//! 注意：部分功能为预留功能，暂未使用
//! Note: Some functions are reserved for future use, not yet used

use crate::state::AppState;
use dioxus::prelude::{ReadableExt, WritableExt};
/// 编辑器 Actions 处理器 / Editor Actions Handler
pub struct EditorActions;

impl EditorActions {
    /// 更新内容 / Update Content
    pub fn update_content(state: &mut AppState, content: String) {
        state.update_content(content);
    }

    /// 撤销 / Undo
    pub fn undo(state: &mut AppState) -> bool {
        state.undo()
    }

    /// 重做 / Redo
    pub fn redo(state: &mut AppState) -> bool {
        state.redo()
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
        *state.font_size.write() = size.clamp(10, 32);
    }

    /// 设置预览字体大小 / Set Preview Font Size
    pub fn set_preview_font_size(state: &mut AppState, size: u32) {
        *state.preview_font_size.write() = size.clamp(10, 32);
    }

    /// 切换自动换行 / Toggle Word Wrap
    pub fn toggle_word_wrap(state: &mut AppState) {
        let current = *state.word_wrap.read();
        *state.word_wrap.write() = !current;
    }

    /// 设置自动换行 / Set Word Wrap
    pub fn set_word_wrap(state: &mut AppState, wrap: bool) {
        *state.word_wrap.write() = wrap;
    }

    /// 切换行号显示 / Toggle Line Numbers
    pub fn toggle_line_numbers(state: &mut AppState) {
        let current = *state.line_numbers.read();
        *state.line_numbers.write() = !current;
    }

    /// 设置行号显示 / Set Line Numbers
    pub fn set_line_numbers(state: &mut AppState, show: bool) {
        *state.line_numbers.write() = show;
    }

    /// 切换同步滚动 / Toggle Sync Scroll
    pub fn toggle_sync_scroll(state: &mut AppState) {
        let current = *state.sync_scroll.read();
        *state.sync_scroll.write() = !current;
    }

    /// 设置同步滚动 / Set Sync Scroll
    pub fn set_sync_scroll(state: &mut AppState, sync: bool) {
        *state.sync_scroll.write() = sync;
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

    // ========== 拼写检查操作 / Spell Check Operations ==========

    /// 切换拼写检查 / Toggle Spell Check
    pub fn toggle_spell_check(state: &mut AppState) {
        let current = *state.spell_check_enabled.read();
        *state.spell_check_enabled.write() = !current;
        if !current {
            // 刚启用，运行检查 / Just enabled, run check
            state.run_spell_check();
        } else {
            // 禁用，清除结果 / Disabled, clear results
            *state.spell_check_results.write() = Vec::new();
        }
    }

    /// 导航到下一个拼写错误 / Navigate to next spell error
    #[allow(dead_code)]
    pub fn next_spell_error(state: &mut AppState) {
        let total = state.spell_check_results.read().len();
        if total == 0 {
            return;
        }
        let current = *state.spell_error_index.read();
        *state.spell_error_index.write() = (current + 1) % total;
    }

    /// 导航到上一个拼写错误 / Navigate to previous spell error
    #[allow(dead_code)]
    pub fn prev_spell_error(state: &mut AppState) {
        let total = state.spell_check_results.read().len();
        if total == 0 {
            return;
        }
        let current = *state.spell_error_index.read();
        *state.spell_error_index.write() = if current == 0 { total - 1 } else { current - 1 };
    }
}
