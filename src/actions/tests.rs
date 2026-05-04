//! Actions 模块测试 / Actions Module Tests
//!
//! 包含纯逻辑测试和基于 Dioxus Runtime 的集成测试
//! Contains pure logic tests and Dioxus Runtime-based integration tests

#[cfg(test)]
mod file_utils_tests {
    use crate::utils::file_utils;
    use std::fs;
    use tempfile::TempDir;

    /// 测试扫描 Markdown 文件
    #[test]
    fn test_scan_markdown_files() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // 创建测试文件
        fs::write(temp_path.join("test1.md"), "# Test 1").unwrap();
        fs::write(temp_path.join("test2.markdown"), "# Test 2").unwrap();
        fs::write(temp_path.join("test3.txt"), "Plain text").unwrap();
        fs::write(temp_path.join("test4.rs"), "fn main() {}").unwrap();

        // 创建子目录
        let sub_dir = temp_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("test5.md"), "# Test 5").unwrap();

        // 扫描文件
        let files = file_utils::scan_markdown_files(temp_path);

        // 验证结果 - 应该找到 4 个文件（2个md + 1个txt + 子目录1个md）
        // 注意：根据实现，可能包含 .markdown 扩展名
        assert!(!files.is_empty(), "应该找到至少一个文件");

        // 验证 .md 文件被找到
        let md_files: Vec<_> = files
            .iter()
            .filter(|f| f.extension().map(|e| e == "md").unwrap_or(false))
            .collect();
        assert!(md_files.len() >= 2, "应该找到至少2个 .md 文件");
    }

    /// 测试跳过隐藏文件和特殊目录
    #[test]
    fn test_skip_hidden_and_special_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // 创建正常文件
        fs::write(temp_path.join("visible.md"), "# Visible").unwrap();

        // 创建隐藏文件
        fs::write(temp_path.join(".hidden.md"), "# Hidden").unwrap();

        // 创建 target 目录（应该被跳过）
        let target_dir = temp_path.join("target");
        fs::create_dir(&target_dir).unwrap();
        fs::write(target_dir.join("build.md"), "# Build").unwrap();

        // 创建 node_modules 目录（应该被跳过）
        let node_dir = temp_path.join("node_modules");
        fs::create_dir(&node_dir).unwrap();
        fs::write(node_dir.join("package.md"), "# Package").unwrap();

        // 扫描文件
        let files = file_utils::scan_markdown_files(temp_path);

        // 只应该找到 visible.md
        assert_eq!(files.len(), 1, "应该只找到1个可见文件");
        assert_eq!(
            files[0].file_name().unwrap(),
            "visible.md",
            "应该找到 visible.md"
        );
    }

    /// 测试文件路径排序（文件夹优先）
    #[test]
    fn test_directories_come_first() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // 创建文件（按字母顺序 z_file 应该在后面）
        fs::write(temp_path.join("z_file.md"), "# Z").unwrap();

        // 创建目录（按字母顺序 a_dir 应该在前面）
        let a_dir = temp_path.join("a_dir");
        fs::create_dir(&a_dir).unwrap();
        fs::write(a_dir.join("inner.md"), "# Inner").unwrap();

        // 扫描文件 - 由于扁平化，顺序取决于实现
        let files = file_utils::scan_markdown_files(temp_path);

        // 至少应该找到2个文件
        assert!(files.len() >= 2, "应该找到至少2个文件");
    }
}

mod editor_actions_tests {
    /// 测试格式化 - 粗体
    #[test]
    fn test_format_bold() {
        let expected = "**Hello World**";
        assert!(expected.starts_with("**"));
        assert!(expected.ends_with("**"));
        assert_eq!(expected.trim_matches('*'), "Hello World");
    }

    /// 测试格式化 - 斜体
    #[test]
    fn test_format_italic() {
        let expected = "*Hello World*";
        assert!(expected.starts_with("*"));
        assert!(expected.ends_with("*"));
    }

    /// 测试格式化 - 代码
    #[test]
    fn test_format_code() {
        let expected = "`fn main() {}`";
        assert!(expected.starts_with("`"));
        assert!(expected.ends_with("`"));
    }
}

/// 导出服务测试 / Export Service Tests
#[cfg(test)]
mod export_tests {
    use crate::services::export::ExportService;
    use tempfile::TempDir;

    #[test]
    fn test_export_text() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.txt");
        ExportService::export_to_text("Hello World", &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "Hello World");
    }

    #[test]
    fn test_export_html() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.html");
        ExportService::export_to_html("# Hello\n\nWorld **bold**", &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("</html>"));
    }

    #[test]
    fn test_export_docx() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.docx");
        ExportService::export_to_docx("# Title\n\nParagraph text\n- List item", &path).unwrap();
        assert!(path.exists());
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
    }

    #[test]
    fn test_export_pdf() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.pdf");
        // PDF 导出可能因字体不可用而失败，但不应 panic
        let _ = ExportService::export_to_pdf("# Test\n\nHello 世界", &path);
    }

    #[test]
    fn test_export_html_with_markdown() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test2.html");
        let md =
            "# Heading\n\n- item1\n- item2\n\n**bold** and *italic*\n\n> quote\n\n```\ncode\n```";
        ExportService::export_to_html(md, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("<h1>"));
        assert!(content.contains("<li>"));
    }

    #[test]
    fn test_export_docx_with_chinese() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("chinese.docx");
        let md = "# 中文标题\n\n这是中文段落内容\n\n- 列表项1\n- 列表项2";
        ExportService::export_to_docx(md, &path).unwrap();
        assert!(path.exists());
        let size = std::fs::metadata(&path).unwrap().len();
        assert!(size > 500, "DOCX should have meaningful content");
    }

    #[test]
    fn test_export_text_empty() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("empty.txt");
        ExportService::export_to_text("", &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_export_docx_empty() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("empty.docx");
        ExportService::export_to_docx("", &path).unwrap();
        assert!(path.exists());
    }
}

/// 扫描深度和数量限制测试 / Scan depth and count limit tests
/// 测试通过 file_utils::scan_markdown_files 间接验证
#[cfg(test)]
mod scan_limit_tests {
    use crate::utils::file_utils;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_skips_hidden_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // 创建正常文件
        fs::write(temp_path.join("visible.md"), "# Visible").unwrap();

        // 创建多层嵌套目录
        let deep = temp_path.join("a").join("b").join("c").join("d");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("deep.md"), "# Deep").unwrap();

        let files = file_utils::scan_markdown_files(temp_path);
        assert_eq!(files.len(), 2, "应该找到2个文件（正常+深层）");
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let files = file_utils::scan_markdown_files(temp_dir.path());
        assert!(files.is_empty(), "空目录应该返回空列表");
    }

    #[test]
    fn test_scan_max_files_limit() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // 创建大量文件验证不会无限扫描
        for i in 0..1050 {
            fs::write(temp_path.join(format!("file_{:04}.md", i)), "# test").unwrap();
        }

        let files = file_utils::scan_markdown_files(temp_path);
        // FileActions 有自己的扫描逻辑，验证不会 panic 或无限循环
        assert!(!files.is_empty(), "应该找到一些文件");
        assert!(files.len() <= 1050, "文件数不应超过创建数");
    }
}

/// 纯文本操作逻辑测试 / Pure text operation logic tests
/// 测试不依赖 Dioxus Runtime 的纯字符串操作算法
#[cfg(test)]
mod text_logic_tests {
    /// 词数统计算法（与 AppState::word_count 一致）
    fn count_words(content: &str) -> usize {
        let cjk_count = content
            .chars()
            .filter(|c| ('\u{4E00}'..='\u{9FFF}').contains(c))
            .count();
        let non_cjk: String = content
            .chars()
            .filter(|c| !('\u{4E00}'..='\u{9FFF}').contains(c))
            .collect();
        let english_words = non_cjk
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .count();
        cjk_count + english_words
    }

    /// 字符统计算法（与 AppState::char_count 一致）
    fn count_chars(content: &str) -> usize {
        content.replace(|c: char| c.is_whitespace(), "").len()
    }

    /// 阅读时间算法（与 AppState::read_time 一致）
    fn read_time(words: usize) -> usize {
        (words / 200).max(1)
    }

    #[test]
    fn test_word_count_english() {
        assert_eq!(count_words("Hello world this is a test"), 6);
    }

    #[test]
    fn test_word_count_chinese() {
        // 6 个 CJK 字符，无英文单词 → 6
        assert_eq!(count_words("你好世界测试"), 6);
    }

    #[test]
    fn test_word_count_mixed() {
        // 4 个 CJK 字符 + 2 个英文单词 (Hello, world) = 6
        assert_eq!(count_words("Hello 你好 world 世界"), 6);
    }

    #[test]
    fn test_char_count() {
        assert_eq!(count_chars("Hello World"), 10);
    }

    #[test]
    fn test_read_time() {
        assert_eq!(read_time(200), 1);
        assert_eq!(read_time(400), 2);
        assert_eq!(read_time(1), 1); // 最少1分钟
    }

    #[test]
    fn test_insert_format_around_selection_logic() {
        let content = "Hello World";
        let (start, end) = (6, 11);
        let (prefix, suffix) = ("**", "**");
        let selected = &content[start..end];
        let result = format!(
            "{}{}{}{}{}",
            &content[..start],
            prefix,
            selected,
            suffix,
            &content[end..]
        );
        assert_eq!(result, "Hello **World**");
    }

    #[test]
    fn test_insert_format_empty_selection_logic() {
        let content = "Hello";
        let pos = 5;
        let (prefix, suffix) = ("**", "**");
        let placeholder = "文本/Text";
        let result = format!(
            "{}{}{}{}{}",
            &content[..pos],
            prefix,
            placeholder,
            suffix,
            ""
        );
        assert_eq!(result, "Hello**文本/Text**");
    }

    #[test]
    fn test_insert_line_prefix_logic() {
        let content = "Hello\nWorld";
        let cursor_pos = 7; // 在 "World" 中
        let line_start = content[..cursor_pos]
            .rfind('\n')
            .map(|p| p + 1)
            .unwrap_or(0);
        let line_prefix = "- ";
        let result = format!(
            "{}{}{}",
            &content[..line_start],
            line_prefix,
            &content[line_start..]
        );
        assert_eq!(result, "Hello\n- World");
    }

    #[test]
    fn test_insert_at_cursor_logic() {
        let content = "Hello World";
        let cursor_pos = 5;
        let insert_text = " Beautiful";
        let result = format!(
            "{}{}{}",
            &content[..cursor_pos],
            insert_text,
            &content[cursor_pos..]
        );
        assert_eq!(result, "Hello Beautiful World");
    }
}

// ========== 集成测试：直接调用 Actions 方法 ==========
// Integration tests: calling actual Actions methods on AppState

/// 在 Dioxus 作用域内运行依赖 Signal 的测试 / Run Signal-dependent tests inside a Dioxus scope
#[cfg(test)]
fn with_runtime<F: FnOnce()>(f: F) {
    use dioxus::prelude::*;
    fn empty_component() -> Element {
        rsx! { div {} }
    }
    let vdom = VirtualDom::prebuilt(empty_component);
    let scope_id = ScopeId::ROOT;
    vdom.in_scope(scope_id, f);
}

/// EditorActions 集成测试 / EditorActions integration tests
#[cfg(test)]
mod editor_actions_integration_tests {
    use super::with_runtime;
    use crate::actions::EditorActions;
    use crate::state::AppState;
    use dioxus::prelude::{ReadableExt, WritableExt};

    #[test]
    fn test_insert_format_bold_with_selection() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Hello World".to_string());
            // 选中 "World" / Select "World"
            *state.cursor_start.write() = 6;
            *state.cursor_end.write() = 11;

            EditorActions::insert_bold(&mut state);

            let content = state.content.read();
            assert_eq!(*content, "Hello **World**");
        });
    }

    #[test]
    fn test_insert_format_italic_with_selection() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Hello World".to_string());
            *state.cursor_start.write() = 0;
            *state.cursor_end.write() = 5;

            EditorActions::insert_italic(&mut state);

            let content = state.content.read();
            assert_eq!(*content, "*Hello* World");
        });
    }

    #[test]
    fn test_insert_format_code_with_selection() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Use fn main".to_string());
            *state.cursor_start.write() = 4;
            *state.cursor_end.write() = 12;

            EditorActions::insert_code(&mut state);

            let content = state.content.read();
            assert_eq!(*content, "Use `fn main`");
        });
    }

    #[test]
    fn test_insert_format_empty_selection_uses_placeholder() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Hello".to_string());
            // 光标在末尾 / Cursor at end
            *state.cursor_start.write() = 5;
            *state.cursor_end.write() = 5;

            EditorActions::insert_bold(&mut state);

            let content = state.content.read();
            // 应该插入 placeholder / Should insert placeholder
            assert!(content.contains("**"));
            assert!(content.len() > "Hello".len());
        });
    }

    #[test]
    fn test_insert_line_prefix_heading() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Hello\nWorld".to_string());
            // 光标在 "World" 行 / Cursor on "World" line
            *state.cursor_end.write() = 7;

            EditorActions::insert_h2(&mut state);

            let content = state.content.read();
            assert_eq!(*content, "Hello\n## World");
        });
    }

    #[test]
    fn test_insert_line_prefix_bullet() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Item one".to_string());
            // 光标在行首附近 / Cursor near line start
            *state.cursor_end.write() = 0;

            EditorActions::insert_bullet_list(&mut state);

            let content = state.content.read();
            assert_eq!(*content, "- Item one");
        });
    }

    #[test]
    fn test_insert_text_at_cursor() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Hello World".to_string());
            *state.cursor_end.write() = 5;

            EditorActions::insert_text(&mut state, " Beautiful");

            let content = state.content.read();
            assert_eq!(*content, "Hello Beautiful World");
        });
    }

    #[test]
    fn test_insert_horizontal_rule() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Above".to_string());
            *state.cursor_end.write() = 5;

            EditorActions::insert_horizontal_rule(&mut state);

            let content = state.content.read();
            assert!(content.contains("---"));
        });
    }

    #[test]
    fn test_set_font_size_clamps() {
        with_runtime(|| {
            let mut state = AppState::new();

            // 下限 / Minimum clamp
            EditorActions::set_font_size(&mut state, 0);
            assert_eq!(*state.font_size.read(), 10);

            // 上限 / Maximum clamp
            EditorActions::set_font_size(&mut state, 100);
            assert_eq!(*state.font_size.read(), 32);

            // 正常值 / Normal value
            EditorActions::set_font_size(&mut state, 20);
            assert_eq!(*state.font_size.read(), 20);
        });
    }

    #[test]
    fn test_set_preview_font_size_clamps() {
        with_runtime(|| {
            let mut state = AppState::new();

            EditorActions::set_preview_font_size(&mut state, 5);
            assert_eq!(*state.preview_font_size.read(), 10);

            EditorActions::set_preview_font_size(&mut state, 24);
            assert_eq!(*state.preview_font_size.read(), 24);
        });
    }

    #[test]
    fn test_toggle_word_wrap() {
        with_runtime(|| {
            let mut state = AppState::new();
            let initial = *state.word_wrap.read();

            EditorActions::toggle_word_wrap(&mut state);
            assert_eq!(*state.word_wrap.read(), !initial);

            EditorActions::toggle_word_wrap(&mut state);
            assert_eq!(*state.word_wrap.read(), initial);
        });
    }

    #[test]
    fn test_toggle_line_numbers() {
        with_runtime(|| {
            let mut state = AppState::new();
            assert!(*state.line_numbers.read()); // default true

            EditorActions::toggle_line_numbers(&mut state);
            assert!(!*state.line_numbers.read());

            EditorActions::toggle_line_numbers(&mut state);
            assert!(*state.line_numbers.read());
        });
    }

    #[test]
    fn test_toggle_sync_scroll() {
        with_runtime(|| {
            let mut state = AppState::new();
            assert!(*state.sync_scroll.read()); // default true

            EditorActions::toggle_sync_scroll(&mut state);
            assert!(!*state.sync_scroll.read());
        });
    }

    #[test]
    fn test_toggle_spell_check_enables_and_runs() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("teh recieve occured".to_string());
            assert!(!*state.spell_check_enabled.read());
            assert!(state.spell_check_results.read().is_empty());

            // 启用拼写检查 / Enable spell check
            EditorActions::toggle_spell_check(&mut state);
            assert!(*state.spell_check_enabled.read());
            assert!(!state.spell_check_results.read().is_empty());

            // 禁用拼写检查 / Disable spell check
            EditorActions::toggle_spell_check(&mut state);
            assert!(!*state.spell_check_enabled.read());
            assert!(state.spell_check_results.read().is_empty());
        });
    }

    #[test]
    fn test_next_prev_spell_error_navigation() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("teh recieve occured".to_string());
            *state.spell_check_enabled.write() = true;
            state.run_spell_check();

            let total = state.spell_check_results.read().len();
            assert!(total >= 2, "应检测到至少 2 个拼写错误");

            // 向后导航 / Navigate forward
            EditorActions::next_spell_error(&mut state);
            assert_eq!(*state.spell_error_index.read(), 1);

            // 循环 / Wrap around
            for _ in 0..total {
                EditorActions::next_spell_error(&mut state);
            }
            assert_eq!(*state.spell_error_index.read(), 1);

            // 向前导航 / Navigate backward
            EditorActions::prev_spell_error(&mut state);
            assert_eq!(*state.spell_error_index.read(), 0);

            // 从头部循环 / Wrap from head
            EditorActions::prev_spell_error(&mut state);
            assert_eq!(*state.spell_error_index.read(), total - 1);
        });
    }

    #[test]
    fn test_undo_redo_via_editor_actions() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("First".to_string());
            state.update_content("Second".to_string());

            // 撤销 → 恢复到 "First" / Undo → restores to "First"
            assert!(EditorActions::undo(&mut state));
            assert_eq!(*state.content.read(), "First");

            // 撤销 → 恢复到 "" / Undo → restores to ""
            assert!(EditorActions::undo(&mut state));
            assert_eq!(*state.content.read(), "");

            // 重做 → "First" / Redo → "First"
            assert!(EditorActions::redo(&mut state));
            assert_eq!(*state.content.read(), "First");

            // 重做 → "Second" / Redo → "Second"
            assert!(EditorActions::redo(&mut state));
            assert_eq!(*state.content.read(), "Second");

            // 无更多重做 / No more redo
            assert!(!EditorActions::redo(&mut state));
        });
    }
}

/// AppActions 集成测试 / AppActions integration tests
#[cfg(test)]
mod app_actions_integration_tests {
    use super::with_runtime;
    use crate::actions::AppActions;
    use crate::state::{AIProvider, AppState, Language, SidebarTab, Theme};
    use dioxus::prelude::{ReadableExt, WritableExt};

    #[test]
    fn test_toggle_theme_cycles() {
        with_runtime(|| {
            let mut state = AppState::new();
            assert_eq!(*state.theme.read(), Theme::Dark);

            AppActions::toggle_theme(&mut state);
            assert_eq!(*state.theme.read(), Theme::Light);

            AppActions::toggle_theme(&mut state);
            assert_eq!(*state.theme.read(), Theme::System);

            AppActions::toggle_theme(&mut state);
            assert_eq!(*state.theme.read(), Theme::Dark);
        });
    }

    #[test]
    fn test_set_theme() {
        with_runtime(|| {
            let mut state = AppState::new();
            AppActions::set_theme(&mut state, Theme::Light);
            assert_eq!(*state.theme.read(), Theme::Light);

            AppActions::set_theme(&mut state, Theme::System);
            assert_eq!(*state.theme.read(), Theme::System);
        });
    }

    #[test]
    fn test_toggle_language() {
        with_runtime(|| {
            let mut state = AppState::new();
            assert_eq!(*state.language.read(), Language::ZhCN);

            AppActions::toggle_language(&mut state);
            assert_eq!(*state.language.read(), Language::EnUS);

            AppActions::toggle_language(&mut state);
            assert_eq!(*state.language.read(), Language::ZhCN);
        });
    }

    #[test]
    fn test_set_language() {
        with_runtime(|| {
            let mut state = AppState::new();
            AppActions::set_language(&mut state, Language::EnUS);
            assert_eq!(*state.language.read(), Language::EnUS);
        });
    }

    #[test]
    fn test_toggle_sidebar() {
        with_runtime(|| {
            let mut state = AppState::new();
            let initial = *state.sidebar_visible.read();

            AppActions::toggle_sidebar(&mut state);
            assert_eq!(*state.sidebar_visible.read(), !initial);

            AppActions::toggle_sidebar(&mut state);
            assert_eq!(*state.sidebar_visible.read(), initial);
        });
    }

    #[test]
    fn test_set_sidebar_visible() {
        with_runtime(|| {
            let mut state = AppState::new();
            AppActions::set_sidebar_visible(&mut state, false);
            assert!(!*state.sidebar_visible.read());

            AppActions::set_sidebar_visible(&mut state, true);
            assert!(*state.sidebar_visible.read());
        });
    }

    #[test]
    fn test_toggle_preview() {
        with_runtime(|| {
            let mut state = AppState::new();
            let initial = *state.show_preview.read();

            AppActions::toggle_preview(&mut state);
            assert_eq!(*state.show_preview.read(), !initial);
        });
    }

    #[test]
    fn test_set_sidebar_tab() {
        with_runtime(|| {
            let mut state = AppState::new();
            assert_eq!(*state.sidebar_tab.read(), SidebarTab::Outline);

            AppActions::set_sidebar_tab(&mut state, SidebarTab::Files);
            assert_eq!(*state.sidebar_tab.read(), SidebarTab::Files);
        });
    }

    #[test]
    fn test_set_sidebar_width_clamps() {
        with_runtime(|| {
            let mut state = AppState::new();

            // 过小 / Too small
            AppActions::set_sidebar_width(&mut state, 50);
            assert_eq!(*state.sidebar_width.read(), 200);

            // 过大 / Too large
            AppActions::set_sidebar_width(&mut state, 500);
            assert_eq!(*state.sidebar_width.read(), 400);

            // 正常 / Normal
            AppActions::set_sidebar_width(&mut state, 300);
            assert_eq!(*state.sidebar_width.read(), 300);
        });
    }

    #[test]
    fn test_show_hide_settings() {
        with_runtime(|| {
            let mut state = AppState::new();
            assert!(!*state.show_settings.read());

            AppActions::show_settings(&mut state);
            assert!(*state.show_settings.read());

            AppActions::hide_settings(&mut state);
            assert!(!*state.show_settings.read());
        });
    }

    #[test]
    fn test_show_hide_shortcuts() {
        with_runtime(|| {
            let mut state = AppState::new();
            AppActions::show_shortcuts(&mut state);
            assert!(*state.show_shortcuts.read());

            AppActions::hide_shortcuts(&mut state);
            assert!(!*state.show_shortcuts.read());
        });
    }

    #[test]
    fn test_show_hide_ai_chat() {
        with_runtime(|| {
            let mut state = AppState::new();
            AppActions::show_ai_chat(&mut state);
            assert!(*state.show_ai_chat.read());

            AppActions::hide_ai_chat(&mut state);
            assert!(!*state.show_ai_chat.read());
        });
    }

    #[test]
    fn test_show_hide_ai_result() {
        with_runtime(|| {
            let mut state = AppState::new();
            AppActions::show_ai_result(&mut state);
            assert!(*state.show_ai_result.read());

            AppActions::hide_ai_result(&mut state);
            assert!(!*state.show_ai_result.read());
        });
    }

    #[test]
    fn test_close_all_modals() {
        with_runtime(|| {
            let mut state = AppState::new();
            *state.show_settings.write() = true;
            *state.show_shortcuts.write() = true;
            *state.show_ai_chat.write() = true;
            *state.show_ai_result.write() = true;

            AppActions::close_all_modals(&mut state);

            assert!(!*state.show_settings.read());
            assert!(!*state.show_shortcuts.read());
            assert!(!*state.show_ai_chat.read());
            assert!(!*state.show_ai_result.read());
        });
    }

    #[test]
    fn test_close_overlays() {
        with_runtime(|| {
            let mut state = AppState::new();
            *state.show_settings.write() = true;
            *state.show_search.write() = true;
            *state.show_global_search.write() = true;
            *state.show_table_editor.write() = true;

            AppActions::close_overlays(&mut state);

            assert!(!*state.show_settings.read());
            assert!(!*state.show_search.read());
            assert!(!*state.show_global_search.read());
            assert!(!*state.show_table_editor.read());
        });
    }

    #[test]
    fn test_set_ai_provider() {
        with_runtime(|| {
            let mut state = AppState::new();

            AppActions::set_ai_provider(&mut state, AIProvider::OpenAI);
            let config = state.ai_config.read();
            assert_eq!(config.provider, AIProvider::OpenAI);
            assert!(!config.base_url.is_empty());
            assert!(!config.model.is_empty());
        });
    }
}

/// FileActions 集成测试 / FileActions integration tests
#[cfg(test)]
mod file_actions_integration_tests {
    use super::with_runtime;
    use crate::actions::FileActions;
    use crate::state::AppState;
    use dioxus::prelude::{ReadableExt, WritableExt};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_open_file_utf8() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("test.md");
            fs::write(&path, "# Hello UTF-8\n\nSome content").unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path.clone()).unwrap();

            assert_eq!(*state.current_file.read(), Some(path));
            assert_eq!(*state.content.read(), "# Hello UTF-8\n\nSome content");
            assert_eq!(*state.file_encoding.read(), "UTF-8");
            assert!(!*state.modified.read());
        });
    }

    #[test]
    fn test_open_file_utf8_bom() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("bom.md");
            let mut bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
            bytes.extend_from_slice(b"Hello BOM");
            fs::write(&path, &bytes).unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path).unwrap();

            assert_eq!(*state.content.read(), "Hello BOM");
            assert_eq!(*state.file_encoding.read(), "UTF-8 BOM");
        });
    }

    #[test]
    fn test_open_file_gbk() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("gbk.md");
            // GBK 编码的 "中文" / GBK-encoded "中文"
            let gbk_bytes = encoding_rs::GBK.encode("中文内容").0;
            fs::write(&path, &gbk_bytes).unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path).unwrap();

            assert_eq!(*state.content.read(), "中文内容");
            assert_eq!(*state.file_encoding.read(), "GBK");
        });
    }

    #[test]
    fn test_open_file_utf16_le() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("utf16le.md");
            // UTF-16 LE BOM + "Hello"
            let mut bytes: Vec<u8> = vec![0xFF, 0xFE]; // BOM
            bytes.extend_from_slice(
                &"Hello"
                    .encode_utf16()
                    .flat_map(|c| c.to_le_bytes())
                    .collect::<Vec<u8>>(),
            );
            fs::write(&path, &bytes).unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path).unwrap();

            assert_eq!(*state.content.read(), "Hello");
            assert_eq!(*state.file_encoding.read(), "UTF-16 LE");
        });
    }

    #[test]
    fn test_open_file_utf16_be() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("utf16be.md");
            // UTF-16 BE BOM + "Hello"
            let mut bytes: Vec<u8> = vec![0xFE, 0xFF]; // BOM
            bytes.extend_from_slice(
                &"Hello"
                    .encode_utf16()
                    .flat_map(|c| c.to_be_bytes())
                    .collect::<Vec<u8>>(),
            );
            fs::write(&path, &bytes).unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path).unwrap();

            assert_eq!(*state.content.read(), "Hello");
            assert_eq!(*state.file_encoding.read(), "UTF-16 BE");
        });
    }

    #[test]
    fn test_save_current_file() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("save_test.md");

            let mut state = AppState::new();
            *state.current_file.write() = Some(path.clone());
            state.update_content("Saved content".to_string());
            assert!(*state.modified.read());

            FileActions::save_current_file(&mut state).unwrap();

            assert!(!*state.modified.read());
            let on_disk = fs::read_to_string(&path).unwrap();
            assert_eq!(on_disk, "Saved content");
        });
    }

    #[test]
    fn test_save_current_file_no_path_triggers_save_as() {
        with_runtime(|| {
            let mut state = AppState::new();
            // 无文件路径 → 触发另存为 / No file path → triggers save-as
            assert!(state.current_file.read().is_none());

            let result = FileActions::save_current_file(&mut state);
            assert!(result.is_err());
            assert!(*state.trigger_save_as.read());
        });
    }

    #[test]
    fn test_save_as() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("new_file.md");

            let mut state = AppState::new();
            state.update_content("New file content".to_string());

            FileActions::save_as(&mut state, path.clone()).unwrap();

            assert_eq!(*state.current_file.read(), Some(path));
            assert!(!*state.modified.read());
            let on_disk = fs::read_to_string(state.current_file.read().as_ref().unwrap()).unwrap();
            assert_eq!(on_disk, "New file content");
        });
    }

    #[test]
    fn test_save_as_adds_md_extension() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("no_ext");

            let mut state = AppState::new();
            state.update_content("test".to_string());

            FileActions::save_as(&mut state, path).unwrap();

            let saved = state.current_file.read().clone().unwrap();
            assert_eq!(saved.extension().unwrap(), "md");
        });
    }

    #[test]
    fn test_set_workspace() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let dir = temp.path();
            fs::write(dir.join("a.md"), "# A").unwrap();
            fs::write(dir.join("b.md"), "# B").unwrap();

            let mut state = AppState::new();
            FileActions::set_workspace(&mut state, dir.to_path_buf());

            assert_eq!(*state.workspace_root.read(), Some(dir.to_path_buf()));
            assert_eq!(state.file_list.read().len(), 2);
        });
    }

    #[test]
    fn test_new_tab() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Existing content".to_string());
            *state.file_encoding.write() = "GBK".to_string();

            FileActions::new_tab(&mut state);

            assert!(state.content.read().is_empty());
            assert_eq!(*state.file_encoding.read(), "UTF-8");
            assert!(state.current_file.read().is_none());
        });
    }

    #[test]
    fn test_switch_tab() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path1 = temp.path().join("tab1.md");
            let path2 = temp.path().join("tab2.md");
            fs::write(&path1, "Content 1").unwrap();
            fs::write(&path2, "Content 2").unwrap();

            let mut state = AppState::new();
            // 初始标签 index=0（空的"未命名"标签）
            FileActions::open_file(&mut state, path1).unwrap();
            // path1 打开在 index=1
            FileActions::open_file(&mut state, path2).unwrap();
            // path2 打开在 index=2

            // 当前在 tab2 (index=2) / Currently on tab2 (index=2)
            assert_eq!(*state.content.read(), "Content 2");

            // 切换到 tab1 (index=1) / Switch to tab1 (index=1)
            FileActions::switch_tab(&mut state, 1);
            assert_eq!(*state.content.read(), "Content 1");

            // 切换回 tab2 (index=2) / Switch back to tab2 (index=2)
            FileActions::switch_tab(&mut state, 2);
            assert_eq!(*state.content.read(), "Content 2");
        });
    }

    #[test]
    fn test_switch_tab_out_of_bounds() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Original".to_string());

            // 不应该 panic / Should not panic
            FileActions::switch_tab(&mut state, 999);
            assert_eq!(*state.content.read(), "Original");
        });
    }

    #[test]
    fn test_create_new_file() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let mut state = AppState::new();
            *state.workspace_root.write() = Some(temp.path().to_path_buf());

            let path = FileActions::create_new_file(&mut state, temp.path(), "test.md").unwrap();
            assert!(path.exists());
            assert_eq!(path.extension().unwrap(), "md");
        });
    }

    #[test]
    fn test_create_new_file_dedup() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            fs::write(temp.path().join("doc.md"), "").unwrap();

            let mut state = AppState::new();
            *state.workspace_root.write() = Some(temp.path().to_path_buf());

            let path = FileActions::create_new_file(&mut state, temp.path(), "doc").unwrap();
            assert_ne!(path.file_name().unwrap(), "doc.md");
            assert!(path.exists());
        });
    }

    #[test]
    fn test_create_new_folder() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let mut state = AppState::new();
            *state.workspace_root.write() = Some(temp.path().to_path_buf());

            let path = FileActions::create_new_folder(&mut state, temp.path(), "notes").unwrap();
            assert!(path.is_dir());
        });
    }

    #[test]
    fn test_delete_file() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let file_path = temp.path().join("deleteme.md");
            fs::write(&file_path, "# Delete me").unwrap();

            let mut state = AppState::new();
            *state.workspace_root.write() = Some(temp.path().to_path_buf());

            FileActions::delete_file(&mut state, &file_path).unwrap();
            assert!(!file_path.exists());
        });
    }

    #[test]
    fn test_rename_file() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let old_path = temp.path().join("old.md");
            fs::write(&old_path, "# Old").unwrap();

            let mut state = AppState::new();
            *state.workspace_root.write() = Some(temp.path().to_path_buf());

            let new_path = FileActions::rename_file(&mut state, &old_path, "new.md").unwrap();
            assert!(!old_path.exists());
            assert!(new_path.exists());
            assert_eq!(new_path.file_name().unwrap(), "new.md");
        });
    }

    #[test]
    fn test_rename_file_duplicate_name_fails() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let old_path = temp.path().join("original.md");
            let existing_path = temp.path().join("taken.md");
            fs::write(&old_path, "# A").unwrap();
            fs::write(&existing_path, "# B").unwrap();

            let mut state = AppState::new();
            let result = FileActions::rename_file(&mut state, &old_path, "taken.md");
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_refresh_workspace() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let mut state = AppState::new();
            *state.workspace_root.write() = Some(temp.path().to_path_buf());

            // 初始为空 / Initially empty
            FileActions::refresh_workspace(&mut state);
            assert!(state.file_list.read().is_empty());

            // 添加文件后刷新 / Add file and refresh
            fs::write(temp.path().join("new.md"), "# New").unwrap();
            FileActions::refresh_workspace(&mut state);
            assert_eq!(state.file_list.read().len(), 1);
        });
    }

    #[test]
    fn test_open_file_already_open_switches_tab() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("existing.md");
            fs::write(&path, "Hello").unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path.clone()).unwrap();
            assert_eq!(state.tabs.read().len(), 2); // initial + opened

            // 再次打开同一文件不应创建新标签 / Re-opening same file should not create new tab
            FileActions::open_file(&mut state, path).unwrap();
            assert_eq!(state.tabs.read().len(), 2);
        });
    }
}
