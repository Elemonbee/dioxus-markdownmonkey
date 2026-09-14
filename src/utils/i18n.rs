//! 国际化支持 / Internationalization Support
//!
//! 提供中英文语言切换的文本映射
//! Provides text mapping for Chinese and English language switching

use crate::state::Language;
use std::collections::HashMap;

/// 国际化文本 / Internationalized Text
pub struct I18n {
    texts: HashMap<&'static str, HashMap<Language, &'static str>>,
}

impl I18n {
    /// 创建新的国际化实例 / Create New I18n Instance
    pub fn new() -> Self {
        let mut texts: HashMap<&'static str, HashMap<Language, &'static str>> = HashMap::new();

        // 文件菜单 / File Menu
        texts.insert("file", lang_map("文件", "File"));
        texts.insert("new", lang_map("新建", "New"));
        texts.insert("open", lang_map("打开", "Open"));
        texts.insert("save", lang_map("保存", "Save"));
        texts.insert("save_as", lang_map("另存为", "Save As"));

        // 编辑菜单 / Edit Menu
        texts.insert("edit", lang_map("编辑", "Edit"));
        texts.insert("undo", lang_map("撤销", "Undo"));
        texts.insert("redo", lang_map("重做", "Redo"));
        texts.insert("cut", lang_map("剪切", "Cut"));
        texts.insert("copy", lang_map("复制", "Copy"));
        texts.insert("paste", lang_map("粘贴", "Paste"));

        // 视图 / View
        texts.insert("view", lang_map("视图", "View"));
        texts.insert("toggle_sidebar", lang_map("切换侧边栏", "Toggle Sidebar"));
        texts.insert("toggle_preview", lang_map("切换预览", "Toggle Preview"));
        texts.insert("outline", lang_map("大纲", "Outline"));
        texts.insert("files", lang_map("文件", "Files"));

        // 设置 / Settings
        texts.insert("settings", lang_map("设置", "Settings"));
        texts.insert("theme", lang_map("主题", "Theme"));
        texts.insert("provider", lang_map("提供商", "Provider"));
        texts.insert("dark", lang_map("深色", "Dark"));
        texts.insert("light", lang_map("浅色", "Light"));
        texts.insert("language", lang_map("语言", "Language"));

        // AI / AI Assistant
        texts.insert("ai_assistant", lang_map("AI 助手", "AI Assistant"));
        texts.insert("ai_continue", lang_map("续写", "Continue"));
        texts.insert(
            "ai_continue_need_selection",
            lang_map(
                "请先选中要续写的段落，避免长文截断后内容对不齐。",
                "Select the passage to continue first, so a truncated long document cannot leave a gap.",
            ),
        );
        texts.insert("ai_improve", lang_map("优化", "Improve"));
        texts.insert("ai_outline", lang_map("大纲", "Outline"));
        texts.insert("ai_translate", lang_map("翻译", "Translate"));
        texts.insert("ai_fix_grammar", lang_map("修正语法", "Fix Grammar"));
        texts.insert(
            "ai_need_selection",
            lang_map("请先选中文本。", "Select text first."),
        );
        texts.insert(
            "ai_translate_to_en",
            lang_map("译成英文", "Translate to English"),
        );
        texts.insert(
            "ai_translate_to_zh",
            lang_map("译成中文", "Translate to Chinese"),
        );
        texts.insert(
            "ai_thinking",
            lang_map("AI 正在思考...", "AI is thinking..."),
        );
        texts.insert(
            "ai_not_enabled",
            lang_map("AI 助手未启用", "AI Assistant is not enabled"),
        );
        texts.insert(
            "ai_configure",
            lang_map(
                "请在设置中配置 API Key",
                "Please configure API Key in settings",
            ),
        );
        texts.insert("ai_clear_history", lang_map("清空会话", "Clear Chat"));
        texts.insert("ai_follow_up", lang_map("继续提问", "Follow Up"));
        texts.insert("ai_history_turns", lang_map("{n} 轮对话", "{n} turns"));
        texts.insert("ai_transcript_user", lang_map("你", "You"));
        texts.insert("ai_transcript_assistant", lang_map("助手", "Assistant"));
        texts.insert("ai_transcript_copy", lang_map("复制", "Copy"));
        texts.insert("ai_transcript_copied", lang_map("已复制", "Copied"));
        texts.insert("ai_stop", lang_map("停止生成", "Stop"));
        texts.insert("ai_generating", lang_map("正在生成…", "Generating…"));
        texts.insert("ai_retry", lang_map("重试", "Retry"));
        texts.insert(
            "ai_replace_selection",
            lang_map("替换选区", "Replace Selection"),
        );
        texts.insert("ai_insert_cursor", lang_map("插入光标", "Insert at Cursor"));
        texts.insert(
            "ai_insert_after_selection",
            lang_map("插入选区后", "Insert After Selection"),
        );
        texts.insert("ai_compare_original", lang_map("原文", "Original"));
        texts.insert("ai_compare_result", lang_map("结果", "Result"));
        texts.insert("ai_translate_into", lang_map("译成", "Translate into"));
        texts.insert("ai_translate_en", lang_map("英文", "English"));
        texts.insert("ai_translate_zh", lang_map("中文", "Chinese"));
        texts.insert(
            "ai_context_warn_title",
            lang_map("上下文较长", "Long Context"),
        );
        texts.insert(
            "ai_context_warn_msg",
            lang_map(
                "发送给 AI 的文本较长，可能较慢或超出模型限制。是否继续？",
                "The text sent to the AI is long and may be slow or exceed the model limit. Continue?",
            ),
        );
        texts.insert(
            "ai_context_truncated_title",
            lang_map("上下文已截断", "Context Truncated"),
        );
        texts.insert(
            "ai_context_truncated_msg",
            lang_map(
                "文本超过上限，已截断后再发送。",
                "The text exceeded the limit and was truncated before sending.",
            ),
        );

        // 状态 / Status
        texts.insert("saved", lang_map("已保存", "Saved"));
        texts.insert("unsaved", lang_map("未保存", "Unsaved"));
        texts.insert("modified", lang_map("已修改", "Modified"));
        texts.insert("chars", lang_map("字符", "characters"));
        texts.insert("words", lang_map("词", "words"));

        // 快捷键 / Shortcuts
        texts.insert("shortcuts", lang_map("快捷键", "Shortcuts"));
        texts.insert("close", lang_map("关闭", "Close"));
        texts.insert("cancel", lang_map("取消", "Cancel"));
        texts.insert("clear", lang_map("清空", "Clear"));
        texts.insert("send", lang_map("发送", "Send"));

        // 文件 / Files
        texts.insert("untitled", lang_map("未命名", "Untitled"));
        texts.insert("aria_toolbar", lang_map("编辑器工具栏", "Editor toolbar"));
        texts.insert(
            "aria_editor",
            lang_map("Markdown 编辑器", "Markdown editor"),
        );
        texts.insert("aria_preview", lang_map("预览", "Preview"));
        texts.insert("aria_sidebar", lang_map("侧边栏", "Sidebar"));
        texts.insert(
            "aria_sidebar_content",
            lang_map("侧边栏内容", "Sidebar content"),
        );
        texts.insert("aria_open_tabs", lang_map("打开的标签", "Open tabs"));
        texts.insert("aria_file_tree", lang_map("文件树", "File tree"));
        texts.insert(
            "aria_ai_transcript",
            lang_map("对话历史", "Conversation history"),
        );
        texts.insert("chars_abbr", lang_map("字", "chars"));
        texts.insert("export_failed", lang_map("导出失败", "Export Failed"));
        texts.insert("export", lang_map("导出", "Export"));
        texts.insert("preview_rendering", lang_map("渲染中…", "Rendering…"));
        texts.insert("open_file", lang_map("打开文件", "Open File"));
        texts.insert("save_file", lang_map("保存文件", "Save File"));

        // 占位符 / Placeholders
        texts.insert("placeholder_text", lang_map("文本", "Text"));
        texts.insert(
            "placeholder_input",
            lang_map(
                "开始输入 Markdown 内容...",
                "Start typing Markdown content...",
            ),
        );

        // 文件修改 / File Modified
        texts.insert("file_modified", lang_map("文件已修改", "File Modified"));
        texts.insert(
            "file_modified_msg",
            lang_map(
                "文件已被其他程序修改。",
                "The file has been modified by another program.",
            ),
        );
        texts.insert("reload", lang_map("重新加载", "Reload"));
        texts.insert("ignore", lang_map("忽略", "Ignore"));

        // 全局搜索 / Global Search
        texts.insert("global_search", lang_map("全局搜索", "Global Search"));
        texts.insert(
            "search_placeholder",
            lang_map("搜索所有 Markdown 文件...", "Search all Markdown files..."),
        );
        texts.insert("no_results", lang_map("未找到结果", "No results found"));

        // 搜索替换 / Search & Replace
        texts.insert("find", lang_map("查找", "Find"));
        texts.insert("case_sensitive", lang_map("区分大小写", "Case Sensitive"));
        texts.insert("regex", lang_map("正则表达式", "Regex"));
        texts.insert("previous", lang_map("上一个", "Previous"));
        texts.insert("next", lang_map("下一个", "Next"));
        texts.insert("replace_btn", lang_map("替换", "Replace"));
        texts.insert("replace", lang_map("替换", "Replace"));
        texts.insert("replace_all", lang_map("全部替换", "Replace All"));
        texts.insert(
            "replace_all_workspace",
            lang_map("工作区全部替换", "Replace All in Workspace"),
        );
        texts.insert("replacing", lang_map("替换中…", "Replacing…"));
        texts.insert(
            "replace_workspace_done",
            lang_map("工作区替换完成", "Workspace replace done"),
        );
        texts.insert(
            "replace_workspace_none",
            lang_map("未找到可替换项", "Nothing to replace"),
        );
        texts.insert(
            "replace_workspace_confirm_title",
            lang_map("确认工作区替换", "Confirm Workspace Replace"),
        );
        texts.insert(
            "replace_workspace_confirm_msg",
            lang_map(
                "将在 {files} 个文件中替换 {matches} 处匹配，此操作会直接写盘。是否继续？",
                "Replace {matches} match(es) in {files} file(s). This writes to disk. Continue?",
            ),
        );

        // 表格 / Table
        texts.insert("table_editor", lang_map("表格编辑器", "Table Editor"));
        texts.insert("add_row", lang_map("添加行", "Add Row"));
        texts.insert("insert_table", lang_map("插入表格", "Insert Table"));

        // 大文件警告 / Large File Warning
        texts.insert(
            "large_file_warning",
            lang_map("大文件警告", "Large File Warning"),
        );
        texts.insert(
            "large_file_msg1",
            lang_map(
                "您正在打开一个大文件，可能会影响编辑器性能。",
                "You are opening a large file, which may affect editor performance.",
            ),
        );
        texts.insert("large_file_msg2", lang_map("建议：大文件可能编辑较慢，实时预览可能会被自动关闭。", "Tip: Large files may be slower to edit, live preview may be automatically disabled."));
        texts.insert("continue_edit", lang_map("继续编辑", "Continue Editing"));

        // 文件操作 / File Operations
        texts.insert("delete_file", lang_map("删除文件", "Delete File"));
        texts.insert("rename_file", lang_map("重命名文件", "Rename File"));
        texts.insert("copy_path", lang_map("复制路径", "Copy Path"));
        texts.insert(
            "reveal_in_explorer",
            lang_map("在文件管理器中显示", "Reveal in File Explorer"),
        );
        texts.insert("confirm_delete", lang_map("确认删除？", "Confirm Delete?"));
        texts.insert("close_tab", lang_map("关闭标签", "Close Tab"));
        texts.insert(
            "close_confirm_title",
            lang_map("关闭确认", "Close Confirmation"),
        );
        texts.insert(
            "close_confirm_msg",
            lang_map(
                "此文件有未保存的更改，确定要关闭吗？",
                "This file has unsaved changes. Are you sure you want to close it?",
            ),
        );
        texts.insert("dont_save", lang_map("不保存并关闭", "Don't Save & Close"));

        // 工具栏 / Toolbar
        texts.insert("new_file", lang_map("新建文件", "New File"));
        texts.insert("export_html", lang_map("导出 HTML", "Export HTML"));
        texts.insert("export_text", lang_map("导出纯文本", "Export Plain Text"));
        texts.insert("export_print", lang_map("打印 / PDF", "Print / PDF"));
        texts.insert("print_failed", lang_map("打印失败", "Print Failed"));
        texts.insert("bold", lang_map("粗体", "Bold"));
        texts.insert("italic", lang_map("斜体", "Italic"));
        texts.insert("code", lang_map("代码", "Code"));
        texts.insert("link", lang_map("链接", "Link"));
        texts.insert("heading_1", lang_map("一级标题", "Heading 1"));
        texts.insert("heading_2", lang_map("二级标题", "Heading 2"));
        texts.insert("heading_3", lang_map("三级标题", "Heading 3"));
        texts.insert("bullet_list", lang_map("无序列表", "Bullet List"));
        texts.insert("numbered_list", lang_map("有序列表", "Numbered List"));
        texts.insert("quote", lang_map("引用", "Quote"));
        texts.insert("code_block", lang_map("代码块", "Code Block"));
        texts.insert("horizontal_rule", lang_map("分割线", "Horizontal Rule"));
        texts.insert("image", lang_map("图片", "Image"));
        texts.insert(
            "auto_save_on",
            lang_map("自动保存已启用", "Auto Save Enabled"),
        );
        texts.insert(
            "auto_save_off",
            lang_map("自动保存已禁用", "Auto Save Disabled"),
        );
        texts.insert("table_editor_btn", lang_map("表格编辑器", "Table Editor"));

        // 状态栏 / Status Bar
        texts.insert("saving", lang_map("保存中...", "Saving..."));
        texts.insert("system", lang_map("系统", "System"));
        texts.insert("read", lang_map("阅读", "Read"));
        texts.insert("min", lang_map("分钟", "min"));

        // 侧边栏 / Sidebar
        texts.insert("no_outline", lang_map("暂无大纲", "No Outline"));
        texts.insert(
            "add_headings",
            lang_map("添加标题以生成大纲", "Add headings to generate outline"),
        );

        // AI 结果 / AI Results
        texts.insert("copy", lang_map("复制", "Copy"));
        texts.insert("append", lang_map("追加到文档", "Append"));
        texts.insert("replace_doc", lang_map("替换全文", "Replace Document"));
        texts.insert(
            "ai_continue_result",
            lang_map("AI 续写结果", "AI Continue Result"),
        );
        texts.insert(
            "ai_improve_result",
            lang_map("AI 优化结果", "AI Improve Result"),
        );
        texts.insert("ai_outline_result", lang_map("AI 生成大纲", "AI Outline"));
        texts.insert(
            "ai_translate_result",
            lang_map("AI 翻译结果", "AI Translation"),
        );
        texts.insert(
            "ai_grammar_result",
            lang_map("AI 语法修正", "AI Grammar Fix"),
        );
        texts.insert("ai_response", lang_map("AI 回复", "AI Response"));
        texts.insert("ai_error", lang_map("AI 错误", "AI Error"));
        texts.insert("error", lang_map("错误", "Error"));
        texts.insert("open_settings_btn", lang_map("打开设置", "Open Settings"));
        texts.insert(
            "custom_input_placeholder",
            lang_map("输入问题…", "Ask a question…"),
        );
        texts.insert(
            "ai_chat_empty",
            lang_map(
                "直接提问即可。会带上当前选区，没有选区则带光标附近的文字。",
                "Ask a question. The current selection is attached, or a window around the cursor if nothing is selected.",
            ),
        );
        texts.insert(
            "ai_chat_context_attached",
            lang_map("附带 {n} / {max} 字上下文", "Attaching {n} / {max} chars"),
        );
        texts.insert(
            "ai_chat_context_chars",
            lang_map("聊天上下文上限", "Chat context limit"),
        );
        texts.insert(
            "ai_chat_context_chars_hint",
            lang_map(
                "发送聊天时附带的文档字数。优先当前选区，否则取光标附近。",
                "Characters of the document attached to chat. Prefers the selection, otherwise a window around the cursor.",
            ),
        );
        texts.insert("ai_context", lang_map("上下文", "Context"));
        texts.insert("ai_context_full", lang_map("全文", "Document"));
        texts.insert("ai_context_selection", lang_map("选区", "Selection"));
        texts.insert(
            "ai_context_full_hint",
            lang_map("将整篇文档发送给 AI", "Sending full document to AI"),
        );
        texts.insert(
            "ai_context_selection_hint",
            lang_map("仅将当前选区发送给 AI", "Sending selected text to AI"),
        );

        // 全局搜索 / Global Search
        texts.insert("searching", lang_map("搜索中...", "Searching..."));
        texts.insert(
            "search_workspace",
            lang_map("搜索工作区中的文本...", "Search text in workspace..."),
        );

        // 标签栏 / Tab Bar
        texts.insert("new_tab", lang_map("新建标签", "New Tab"));

        // 表格列名 / Table Column
        texts.insert("column", lang_map("列", "Column"));

        // 快捷键弹窗 / Shortcuts Modal
        texts.insert("insert_link", lang_map("插入链接", "Insert Link"));
        texts.insert("show_shortcuts", lang_map("显示快捷键", "Show Shortcuts"));
        texts.insert("close_modal", lang_map("关闭弹窗", "Close Modal"));
        texts.insert(
            "shortcut_search",
            lang_map("文档内搜索替换", "In-document Search & Replace"),
        );
        texts.insert(
            "shortcut_global_search",
            lang_map("工作区全局搜索 / 替换", "Workspace Search / Replace"),
        );
        texts.insert(
            "shortcut_print",
            lang_map("打印 / 另存 PDF", "Print / Save as PDF"),
        );

        // 设置弹窗 / Settings Modal
        texts.insert("editor", lang_map("编辑器", "Editor"));
        texts.insert("font_size", lang_map("字体大小", "Font Size"));
        texts.insert(
            "preview_font_size",
            lang_map("预览字体大小", "Preview Font Size"),
        );
        texts.insert("word_wrap", lang_map("自动换行", "Word Wrap"));
        texts.insert("line_numbers", lang_map("显示行号", "Line Numbers"));
        texts.insert("sync_scroll", lang_map("同步滚动", "Sync Scroll"));
        texts.insert("auto_save", lang_map("自动保存", "Auto Save"));
        texts.insert(
            "auto_save_interval",
            lang_map("自动保存间隔（秒）", "Auto Save Interval (sec)"),
        );
        texts.insert(
            "session_restore",
            lang_map("启动时恢复上次会话", "Restore last session on startup"),
        );
        texts.insert("appearance", lang_map("外观", "Appearance"));
        texts.insert("sidebar_width", lang_map("侧边栏宽度", "Sidebar Width"));
        texts.insert(
            "sidebar_resize",
            lang_map("拖拽调整侧边栏宽度", "Drag to resize sidebar"),
        );
        texts.insert("enable_ai", lang_map("启用 AI", "Enable AI"));
        texts.insert("enter_api_key", lang_map("输入 API Key", "Enter API Key"));
        texts.insert("model_name", lang_map("模型名称", "Model Name"));
        texts.insert("temperature", lang_map("温度 (0-1)", "Temperature"));
        texts.insert("system_prompt", lang_map("系统提示词", "System Prompt"));
        texts.insert(
            "system_prompt_hint",
            lang_map(
                "作为所有 AI 任务提示词的全局前缀",
                "Global prefix for every AI task prompt",
            ),
        );
        texts.insert("reset_default", lang_map("重置默认", "Reset"));
        texts.insert("save_close", lang_map("保存并关闭", "Save & Close"));
        texts.insert("follow_system", lang_map("跟随系统", "System"));

        // 预览 / Preview
        texts.insert("preview", lang_map("预览", "Preview"));
        texts.insert("sync_scroll_toggle", lang_map("同步滚动", "Sync Scroll"));

        // 文件树 / File Tree
        texts.insert("new_file_btn", lang_map("新建文件", "New File"));
        texts.insert("new_folder", lang_map("新建文件夹", "New Folder"));
        texts.insert("select_folder", lang_map("选择文件夹", "Select Folder"));
        texts.insert("refresh", lang_map("刷新", "Refresh"));
        texts.insert("search_files", lang_map("搜索文件...", "Search files..."));
        texts.insert("files_found", lang_map("个文件", "files found"));
        texts.insert(
            "no_matching_files",
            lang_map("未找到匹配文件", "No matching files found"),
        );
        texts.insert(
            "click_to_select",
            lang_map("点击选择文件夹", "Click to select folder"),
        );
        texts.insert("recent_files", lang_map("最近打开", "Recent Files"));
        texts.insert(
            "no_recent_files",
            lang_map("暂无最近文件", "No recent files"),
        );
        texts.insert(
            "aria_recent_files",
            lang_map("最近打开的文件", "Recently opened files"),
        );
        texts.insert("clear_recent", lang_map("清空最近", "Clear Recent"));
        texts.insert("line", lang_map("行", "Line"));
        texts.insert("current_file", lang_map("当前文件", "Current File"));
        texts.insert(
            "navigate_open",
            lang_map(
                "↑↓ 导航 • Enter 打开 • Esc 关闭",
                "↑↓ Navigate • Enter Open • Esc Close",
            ),
        );

        // 状态栏补充 / Status Bar additions
        texts.insert("file_type_markdown", lang_map("Markdown", "Markdown"));

        // 设置弹窗补充 / Settings Modal additions
        texts.insert("api_key_label", lang_map("API Key", "API Key"));
        texts.insert(
            "api_base_url_label",
            lang_map("API Base URL", "API Base URL"),
        );
        texts.insert(
            "api_base_url_placeholder",
            lang_map("https://api.openai.com", "https://api.openai.com"),
        );
        texts.insert("provider_openai", lang_map("OpenAI", "OpenAI"));
        texts.insert("provider_claude", lang_map("Claude", "Claude"));
        texts.insert("provider_ollama", lang_map("Ollama", "Ollama"));
        texts.insert("provider_deepseek", lang_map("DeepSeek", "DeepSeek"));
        texts.insert("provider_kimi", lang_map("Kimi", "Kimi"));
        texts.insert("provider_openrouter", lang_map("OpenRouter", "OpenRouter"));
        texts.insert(
            "loading_models",
            lang_map("加载模型中...", "Loading models..."),
        );
        texts.insert("fetch_models", lang_map("获取模型", "Fetch Models"));
        texts.insert("custom_model", lang_map("自定义模型", "Custom Model"));
        texts.insert("no_models_found", lang_map("未找到模型", "No models found"));
        texts.insert("select_model", lang_map("选择模型", "Select Model"));
        texts.insert(
            "large_file_threshold",
            lang_map("大文件提醒阈值", "Large File Warning Threshold"),
        );
        texts.insert(
            "ai_clear_history_confirm",
            lang_map(
                "确定清空当前 AI 会话历史？",
                "Clear the current AI conversation history?",
            ),
        );

        Self { texts }
    }

    /// 获取文本 / Get Text
    pub fn get<'a>(&self, key: &'a str, lang: Language) -> &'a str {
        self.texts
            .get(key)
            .and_then(|m| m.get(&lang))
            .copied()
            .unwrap_or(key)
    }
}

/// 创建语言映射 / Create Language Mapping
fn lang_map(zh: &'static str, en: &'static str) -> HashMap<Language, &'static str> {
    let mut map = HashMap::new();
    map.insert(Language::ZhCN, zh);
    map.insert(Language::EnUS, en);
    map
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局国际化实例 / Global I18n Instance
pub fn t(key: &str, lang: Language) -> String {
    thread_local! {
        static I18N: I18n = I18n::new();
    }

    I18N.with(|i18n| i18n.get(key, lang).to_string())
}

/// 未命名标签标题 / Untitled tab title
pub fn untitled_tab_title(lang: Language) -> String {
    t("untitled", lang)
}

/// 带序号的未命名标签标题 / Numbered untitled tab title
pub fn untitled_tab_title_n(lang: Language, n: usize) -> String {
    format!("{} {}", t("untitled", lang), n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_zh_cn() {
        let result = t("file", Language::ZhCN);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_i18n_en() {
        let result = t("file", Language::EnUS);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_i18n_missing_key_returns_key() {
        let result = t("nonexistent_key_12345", Language::EnUS);
        assert_eq!(result, "nonexistent_key_12345");
    }

    #[test]
    fn test_i18n_different_languages() {
        let zh = t("save", Language::ZhCN);
        let en = t("save", Language::EnUS);
        // They should both be non-empty
        assert!(!zh.is_empty());
        assert!(!en.is_empty());
    }

    #[test]
    fn test_i18n_untitled_helpers() {
        assert_eq!(untitled_tab_title(Language::ZhCN), "未命名");
        assert_eq!(untitled_tab_title(Language::EnUS), "Untitled");
        assert_eq!(untitled_tab_title_n(Language::EnUS, 2), "Untitled 2");
    }

    #[test]
    fn test_ai_apply_i18n_keys() {
        for key in [
            "ai_retry",
            "ai_replace_selection",
            "ai_insert_cursor",
            "ai_insert_after_selection",
            "ai_continue_need_selection",
            "ai_need_selection",
            "ai_translate_into",
            "ai_translate_to_en",
            "ai_chat_context_chars",
            "ai_chat_context_attached",
            "ai_chat_empty",
            "ai_context_warn_title",
            "ai_context_truncated_msg",
        ] {
            assert_ne!(t(key, Language::ZhCN), key);
            assert_ne!(t(key, Language::EnUS), key);
        }
        assert_eq!(t("ai_replace_selection", Language::ZhCN), "替换选区");
        assert_eq!(t("ai_insert_cursor", Language::EnUS), "Insert at Cursor");
    }

    /// 关闭确认按钮应同时提供中英文文案 / Close-confirm buttons provide both locales
    #[test]
    fn test_close_confirm_action_i18n() {
        assert_eq!(t("save_close", Language::ZhCN), "保存并关闭");
        assert_eq!(t("save_close", Language::EnUS), "Save & Close");
        assert_eq!(t("dont_save", Language::ZhCN), "不保存并关闭");
        assert_eq!(t("dont_save", Language::EnUS), "Don't Save & Close");
    }
}
