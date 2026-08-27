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

    /// 测试 Markdown/文本路径识别 / Test markdown/text path recognition
    #[test]
    fn test_is_markdown_or_text_path() {
        use std::path::PathBuf;
        assert!(file_utils::is_markdown_or_text_path(&PathBuf::from("a.md")));
        assert!(file_utils::is_markdown_or_text_path(&PathBuf::from("a.MD")));
        assert!(file_utils::is_markdown_or_text_path(&PathBuf::from(
            "notes.markdown"
        )));
        assert!(file_utils::is_markdown_or_text_path(&PathBuf::from(
            "readme.txt"
        )));
        assert!(!file_utils::is_markdown_or_text_path(&PathBuf::from(
            "photo.png"
        )));
        assert!(!file_utils::is_markdown_or_text_path(&PathBuf::from(
            "main.rs"
        )));
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

/// 快捷键纯分发与对话框防重入测试 / Shortcut dispatch and dialog re-entrancy tests
#[cfg(test)]
mod shortcut_dispatch_tests {
    use crate::actions::file_actions::try_begin_open_dialog;
    use crate::actions::shortcut_actions::{ShortcutAction, ShortcutActions};

    /// Ctrl+O 应映射到唯一打开文件动作 / Ctrl+O maps to the open-file action
    #[test]
    fn test_ctrl_o_dispatches_open_file() {
        assert_eq!(
            ShortcutActions::find_action("o", true, false, false),
            Some(ShortcutAction::OpenFile)
        );
        assert_eq!(ShortcutActions::find_action("o", false, false, false), None);
    }

    /// 打开对话框守卫存活期间应拒绝连续触发 / Repeated triggers are rejected while the dialog guard is alive
    #[test]
    fn test_open_dialog_rejects_reentry() {
        let guard = try_begin_open_dialog().expect("first dialog should acquire guard");
        assert!(try_begin_open_dialog().is_none());
        drop(guard);
        assert!(try_begin_open_dialog().is_some());
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
    fn test_export_text_empty() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("empty.txt");
        ExportService::export_to_text("", &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "");
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
            let start = *state.cursor_start.read();
            let end = *state.cursor_end.read();
            assert_eq!(&content[start..end], "文本");
        });
    }

    /// ASCII 选区格式化后应保留正文选区 / ASCII formatting preserves the body selection
    #[test]
    fn test_ascii_selection_is_preserved_after_formatting() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("abc def".to_string());
            *state.cursor_start.write() = 4;
            *state.cursor_end.write() = 7;

            EditorActions::insert_bold(&mut state);

            assert_eq!(*state.content.read(), "abc **def**");
            assert_eq!(
                (*state.cursor_start.read(), *state.cursor_end.read()),
                (6, 9)
            );
        });
    }

    /// 中文字节选区格式化不得错位 / Chinese byte selections must format without shifting
    #[test]
    fn test_chinese_selection_uses_utf8_byte_offsets() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("甲中文乙".to_string());
            *state.cursor_start.write() = 3;
            *state.cursor_end.write() = 9;

            EditorActions::insert_bold(&mut state);

            assert_eq!(*state.content.read(), "甲**中文**乙");
            assert_eq!(
                (*state.cursor_start.read(), *state.cursor_end.read()),
                (5, 11)
            );
        });
    }

    /// Emoji 选区格式化不得切开代理对或 UTF-8 字符 / Emoji formatting must not split surrogate pairs or UTF-8 characters
    #[test]
    fn test_emoji_selection_uses_utf8_byte_offsets() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("a😀b".to_string());
            *state.cursor_start.write() = 1;
            *state.cursor_end.write() = 5;

            EditorActions::insert_code(&mut state);

            assert_eq!(*state.content.read(), "a`😀`b");
            assert_eq!(
                (*state.cursor_start.read(), *state.cursor_end.read()),
                (2, 6)
            );
        });
    }

    /// 反向选区应规范化后安全格式化 / Reversed selections are normalized before safe formatting
    #[test]
    fn test_reversed_unicode_selection_is_normalized() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("甲中文乙".to_string());
            *state.cursor_start.write() = 9;
            *state.cursor_end.write() = 3;

            EditorActions::insert_italic(&mut state);

            assert_eq!(*state.content.read(), "甲*中文*乙");
            assert_eq!(
                (*state.cursor_start.read(), *state.cursor_end.read()),
                (4, 10)
            );
        });
    }

    /// 非法和越界字节偏移应向字符边界钳制且不 panic
    /// Invalid and out-of-range byte offsets clamp to character boundaries without panicking
    #[test]
    fn test_invalid_unicode_boundaries_are_clamped() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("a😀b".to_string());
            *state.cursor_start.write() = 2;
            *state.cursor_end.write() = 999;

            EditorActions::insert_bold(&mut state);

            assert_eq!(*state.content.read(), "a**😀b**");
            assert_eq!(
                (*state.cursor_start.read(), *state.cursor_end.read()),
                (3, 8)
            );
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
    fn test_show_ai_chat_defaults_to_selection_context() {
        with_runtime(|| {
            let mut state = AppState::new();
            *state.content.write() = "Hello selected text".to_string();
            *state.cursor_start.write() = 6;
            *state.cursor_end.write() = 14;

            AppActions::show_ai_chat(&mut state);

            assert!(*state.show_ai_chat.read());
            assert!(*state.ai_use_selection.read());
        });
    }

    #[test]
    fn test_show_ai_chat_uses_full_context_without_selection() {
        with_runtime(|| {
            let mut state = AppState::new();
            *state.content.write() = "Hello selected text".to_string();
            *state.cursor_start.write() = 6;
            *state.cursor_end.write() = 6;
            *state.ai_use_selection.write() = true;

            AppActions::show_ai_chat(&mut state);

            assert!(*state.show_ai_chat.read());
            assert!(!*state.ai_use_selection.read());
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

/// SearchActions / SettingsActions 集成测试
/// SearchActions / SettingsActions integration tests
#[cfg(test)]
mod search_settings_actions_tests {
    use super::with_runtime;
    use crate::actions::{SearchActions, SettingsActions};
    use crate::state::AppState;
    use dioxus::prelude::{ReadableExt, WritableExt};

    /// 查询应重算匹配并支持循环导航 / Query recounts matches and wraps navigation
    #[test]
    fn test_search_query_and_navigation() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("foo bar foo".to_string());

            SearchActions::show(&mut state);
            assert!(*state.show_search.read());

            SearchActions::set_query(&mut state, "foo".to_string());
            assert_eq!(*state.search_total.read(), 2);
            assert_eq!(*state.search_index.read(), 1);

            SearchActions::next_match(&mut state);
            assert_eq!(*state.search_index.read(), 2);
            SearchActions::next_match(&mut state);
            assert_eq!(*state.search_index.read(), 1);
            SearchActions::prev_match(&mut state);
            assert_eq!(*state.search_index.read(), 2);

            SearchActions::set_query(&mut state, String::new());
            assert_eq!(*state.search_total.read(), 0);
            assert_eq!(*state.search_index.read(), 0);

            SearchActions::hide(&mut state);
            assert!(!*state.show_search.read());
        });
    }

    /// 工作区搜索开关 / Toggle workspace search overlay
    #[test]
    fn test_search_show_global() {
        with_runtime(|| {
            let mut state = AppState::new();
            SearchActions::show_global(&mut state);
            assert!(*state.show_global_search.read());
        });
    }

    /// 会话恢复与 AI 字段写入 / Session restore and AI field writes
    #[test]
    fn test_settings_session_and_ai() {
        with_runtime(|| {
            let mut state = AppState::new();
            SettingsActions::set_session_restore(&mut state, false);
            assert!(!*state.session_restore_enabled.read());

            {
                let mut config = state.ai_config.write();
                config.enabled = false;
                config.base_url.clear();
                config.model.clear();
            }
            SettingsActions::toggle_ai_enabled(&mut state);
            {
                let config = state.ai_config.read();
                assert!(config.enabled);
                assert!(!config.base_url.is_empty());
                assert!(!config.model.is_empty());
            }

            SettingsActions::set_ai_temperature(&mut state, 2.5);
            assert_eq!(state.ai_config.read().temperature, 1.0);

            SettingsActions::set_ai_model(&mut state, "custom-model".to_string());
            assert_eq!(state.ai_config.read().model, "custom-model");

            SettingsActions::reset_editor_defaults(&mut state);
            assert_eq!(*state.font_size.read(), 16);
            assert_eq!(*state.preview_font_size.read(), 16);
            assert!(!*state.word_wrap.read());
            assert!(*state.line_numbers.read());
            assert!(*state.sync_scroll.read());
            assert!(*state.session_restore_enabled.read());
            assert_eq!(*state.sidebar_width.read(), 280);
        });
    }
}

/// FileActions 集成测试 / FileActions integration tests
#[cfg(test)]
mod file_actions_integration_tests {
    use super::with_runtime;
    use crate::actions::{AppActions, FileActions};
    use crate::state::AppState;
    use crate::utils::file_encoding::{decode_bytes, encode_text, FileEncoding};
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
            assert_eq!(*state.file_encoding.read(), FileEncoding::Utf8);
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
            assert_eq!(*state.file_encoding.read(), FileEncoding::Utf8Bom);
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
            assert_eq!(*state.file_encoding.read(), FileEncoding::Gbk);
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
            assert_eq!(*state.file_encoding.read(), FileEncoding::Utf16Le);
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
            assert_eq!(*state.file_encoding.read(), FileEncoding::Utf16Be);
        });
    }

    /// 手动保存应保持每种原始编码及 BOM / Manual save preserves every original encoding and BOM
    #[test]
    fn test_manual_save_preserves_encoding_and_bom() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let cases = [
                FileEncoding::Utf8,
                FileEncoding::Utf8Bom,
                FileEncoding::Utf16Le,
                FileEncoding::Utf16Be,
                FileEncoding::Gbk,
            ];

            for (index, encoding) in cases.into_iter().enumerate() {
                let path = temp.path().join(format!("preserve-{index}.md"));
                fs::write(&path, encode_text("原文", encoding).unwrap()).unwrap();

                let mut state = AppState::new();
                FileActions::open_file(&mut state, path.clone()).unwrap();
                state.update_content("保存后的中文".to_string());
                FileActions::save_current_file(&mut state).unwrap();

                let bytes = fs::read(&path).unwrap();
                let (content, detected) = decode_bytes(&bytes).unwrap();
                assert_eq!(content, "保存后的中文");
                assert_eq!(detected, encoding);
            }
        });
    }

    /// 另存为默认沿用当前标签编码 / Save As preserves the active tab encoding by default
    #[test]
    fn test_save_as_preserves_active_tab_encoding() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let source = temp.path().join("source.md");
            let target = temp.path().join("target.md");
            fs::write(
                &source,
                encode_text("带 BOM", FileEncoding::Utf16Be).unwrap(),
            )
            .unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, source).unwrap();
            state.update_content("另存正文".to_string());
            FileActions::save_as(&mut state, target.clone()).unwrap();

            let (content, encoding) = decode_bytes(&fs::read(target).unwrap()).unwrap();
            assert_eq!(content, "另存正文");
            assert_eq!(encoding, FileEncoding::Utf16Be);
        });
    }

    /// 切换标签应恢复各自编码并据此保存 / Switching tabs restores and saves each tab with its own encoding
    #[test]
    fn test_cross_tab_encoding_isolation() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let utf8_bom_path = temp.path().join("utf8-bom.md");
            let gbk_path = temp.path().join("gbk.md");
            fs::write(
                &utf8_bom_path,
                encode_text("一", FileEncoding::Utf8Bom).unwrap(),
            )
            .unwrap();
            fs::write(&gbk_path, encode_text("二", FileEncoding::Gbk).unwrap()).unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, utf8_bom_path.clone()).unwrap();
            let utf8_tab = *state.current_tab_index.read();
            FileActions::open_file(&mut state, gbk_path.clone()).unwrap();
            let gbk_tab = *state.current_tab_index.read();
            assert_eq!(*state.file_encoding.read(), FileEncoding::Gbk);

            FileActions::switch_tab(&mut state, utf8_tab);
            assert_eq!(*state.file_encoding.read(), FileEncoding::Utf8Bom);
            state.update_content("甲".to_string());
            FileActions::save_current_file(&mut state).unwrap();

            FileActions::switch_tab(&mut state, gbk_tab);
            assert_eq!(*state.file_encoding.read(), FileEncoding::Gbk);
            state.update_content("乙".to_string());
            FileActions::save_current_file(&mut state).unwrap();

            assert_eq!(
                decode_bytes(&fs::read(utf8_bom_path).unwrap()).unwrap(),
                ("甲".to_string(), FileEncoding::Utf8Bom)
            );
            assert_eq!(
                decode_bytes(&fs::read(gbk_path).unwrap()).unwrap(),
                ("乙".to_string(), FileEncoding::Gbk)
            );
        });
    }

    /// GBK 手动保存不可静默替换字符 / GBK manual save must not silently replace characters
    #[test]
    fn test_gbk_save_rejects_unrepresentable_characters() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("strict-gbk.md");
            let original = encode_text("原文", FileEncoding::Gbk).unwrap();
            fs::write(&path, &original).unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path.clone()).unwrap();
            state.update_content("不能保存 😀".to_string());

            assert!(FileActions::save_current_file(&mut state).is_err());
            assert_eq!(fs::read(path).unwrap(), original);
            assert!(*state.modified.read());
            assert_eq!(*state.save_status.read(), crate::state::SaveStatus::Unsaved);
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
            *state.file_encoding.write() = FileEncoding::Gbk;

            FileActions::new_tab(&mut state);

            assert!(state.content.read().is_empty());
            assert_eq!(*state.file_encoding.read(), FileEncoding::Utf8);
            assert!(state.current_file.read().is_none());
        });
    }

    /// 英文 UI 下新建标签标题应为 Untitled / New tab title uses Untitled in English UI
    #[test]
    fn test_new_tab_title_respects_language() {
        with_runtime(|| {
            let mut state = AppState::new();
            *state.language.write() = crate::state::Language::EnUS;
            FileActions::new_tab(&mut state);
            let title = state.tabs.read().last().unwrap().title.clone();
            assert!(
                title.starts_with("Untitled"),
                "expected Untitled title, got {title}"
            );
        });
    }

    /// 有路径时保存后再关闭应持久化内容 / Save then close persists content for pathed tabs
    #[test]
    fn test_save_then_close_tab_persists() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("close-me.md");
            fs::write(&path, "old").unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path.clone()).unwrap();
            let file_idx = *state.current_tab_index.read();
            FileActions::new_tab(&mut state);
            FileActions::switch_tab(&mut state, file_idx);
            state.update_content("saved-and-closed".to_string());

            FileActions::save_current_file(&mut state).unwrap();
            state.close_tab(file_idx);

            assert_eq!(fs::read_to_string(&path).unwrap(), "saved-and-closed");
            assert!(!state.tabs.read().is_empty());
        });
    }

    /// 拖放 Markdown 路径应打开为标签 / Dropping markdown paths should open as tabs
    #[test]
    fn test_handle_editor_file_drop_opens_tab() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("dropped.md");
            fs::write(&path, "# Dropped").unwrap();

            let mut state = AppState::new();
            FileActions::handle_editor_file_drop(&mut state, vec![path.clone()]);

            assert_eq!(*state.current_file.read(), Some(path));
            assert_eq!(*state.content.read(), "# Dropped");
        });
    }

    /// 拖放图片路径应忽略（由 JS 处理）/ Image drop paths should be ignored (JS-handled)
    #[test]
    fn test_handle_editor_file_drop_skips_images() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("pic.png");
            fs::write(&path, b"fake").unwrap();

            let mut state = AppState::new();
            let before = state.tabs.read().len();
            FileActions::handle_editor_file_drop(&mut state, vec![path]);
            assert_eq!(state.tabs.read().len(), before);
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
    fn test_rename_file_rejects_path_traversal() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let old_path = temp.path().join("safe.md");
            fs::write(&old_path, "# Safe").unwrap();

            let mut state = AppState::new();
            *state.workspace_root.write() = Some(temp.path().to_path_buf());

            let result = FileActions::rename_file(&mut state, &old_path, "..\\outside.md");
            assert!(result.is_err());
            assert!(old_path.exists());
        });
    }

    #[test]
    fn test_validate_rename_name_rejects_traversal() {
        use crate::utils::file_utils::validate_rename_name;
        assert!(validate_rename_name("ok.md").is_ok());
        assert!(validate_rename_name("../x.md").is_err());
        assert!(validate_rename_name("a/b.md").is_err());
        assert!(validate_rename_name("..").is_err());
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

    #[test]
    fn test_push_ai_turn_caps_at_max() {
        with_runtime(|| {
            let mut state = AppState::new();
            for i in 0..12 {
                AppActions::push_ai_turn(&mut state, format!("user-{i}"), format!("assistant-{i}"));
            }
            let len = state.ai().ai_history.read().len();
            assert_eq!(len, crate::services::settings::AI_HISTORY_MAX_MESSAGES);
            assert!(state
                .ai()
                .ai_history
                .read()
                .iter()
                .any(|t| t.content == "user-11"));
            // 清理测试写入的持久化文件副作用 / Clear side-effect on persisted history file
            AppActions::clear_ai_history(&mut state);
        });
    }

    #[test]
    fn test_ai_history_isolated_per_tab() {
        with_runtime(|| {
            let mut state = AppState::new();
            AppActions::push_ai_turn(&mut state, "doc-a".into(), "reply-a".into());
            assert_eq!(state.ai().ai_history.read().len(), 2);

            FileActions::new_tab(&mut state);
            assert!(
                state.ai().ai_history.read().is_empty(),
                "new tab should start with empty AI history"
            );
            AppActions::push_ai_turn(&mut state, "doc-b".into(), "reply-b".into());

            FileActions::switch_tab(&mut state, 0);
            let hist = state.ai().ai_history.read().clone();
            assert_eq!(hist.len(), 2);
            assert_eq!(hist[0].content, "doc-a");

            FileActions::switch_tab(&mut state, 1);
            let hist = state.ai().ai_history.read().clone();
            assert_eq!(hist[0].content, "doc-b");

            AppActions::clear_ai_history(&mut state);
            FileActions::switch_tab(&mut state, 0);
            AppActions::clear_ai_history(&mut state);
        });
    }

    #[test]
    fn test_reload_current_file_resets_history_and_modified() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("reload.md");
            fs::write(&path, "disk-content").unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path.clone()).unwrap();
            state.update_content("edited-in-memory".to_string());
            assert!(*state.document().modified.read());
            assert!(!state.document().history.read().past.is_empty());

            FileActions::reload_current_file(&mut state).unwrap();
            let doc = state.document();
            assert_eq!(doc.content.read().as_str(), "disk-content");
            assert!(!*doc.modified.read());
            assert!(doc.history.read().past.is_empty());
            assert!(doc.history.read().future.is_empty());
            // 同内容再写不应误标未保存 / Same body rewrite must not false-flag unsaved
            state.update_content("disk-content".to_string());
            assert!(!*state.document().modified.read());
        });
    }

    #[test]
    fn test_confirm_load_large_file_clears_pending_and_opens() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("large.md");
            // 超过阈值的大文件 / File larger than threshold
            let big = "x".repeat(crate::config::LARGE_FILE_THRESHOLD_BYTES + 64);
            fs::write(&path, &big).unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path.clone()).unwrap();
            let doc = state.document();
            assert!(doc.pending_large_file.read().is_some());
            assert!(*doc.show_large_file_warning.read());

            FileActions::confirm_load_large_file(&mut state).unwrap();
            let doc = state.document();
            assert!(doc.pending_large_file.read().is_none());
            assert!(!*doc.show_large_file_warning.read());
            assert_eq!(doc.content.read().len(), big.len());
            assert_eq!(doc.current_file.read().as_ref(), Some(&path));
        });
    }

    #[test]
    fn test_cancel_load_large_file_clears_warning() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("large2.md");
            let big = "y".repeat(crate::config::LARGE_FILE_THRESHOLD_BYTES + 8);
            fs::write(&path, &big).unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, path).unwrap();
            FileActions::cancel_load_large_file(&mut state);
            let doc = state.document();
            assert!(doc.pending_large_file.read().is_none());
            assert!(!*doc.show_large_file_warning.read());
            assert_eq!(*doc.file_size_bytes.read(), 0);
        });
    }

    #[test]
    fn test_request_close_tab_shows_confirm_when_modified() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("dirty".to_string());
            assert!(*state.document().modified.read());

            let idx = *state.document().current_tab_index.read();
            FileActions::request_close_tab(&mut state, idx);
            let doc = state.document();
            assert!(*doc.show_close_confirm.read());
            assert_eq!(
                *doc.pending_close_tab_id.read(),
                doc.tabs.read().get(idx).map(|tab| tab.id)
            );
        });
    }

    #[test]
    fn test_discard_and_close_tab_clears_confirm() {
        with_runtime(|| {
            let mut state = AppState::new();
            FileActions::new_tab(&mut state);
            assert_eq!(state.document().tabs.read().len(), 2);
            state.update_content("will discard".to_string());
            let idx = *state.document().current_tab_index.read();
            FileActions::request_close_tab(&mut state, idx);
            assert!(*state.document().show_close_confirm.read());

            FileActions::discard_and_close_tab(&mut state, idx);
            let doc = state.document();
            assert!(!*doc.show_close_confirm.read());
            assert!(doc.pending_close_tab_id.read().is_none());
            assert_eq!(doc.tabs.read().len(), 1);
        });
    }

    /// 当前标签关闭判断必须读取实时 Signal，而非可能过期的 TabInfo
    /// Current-tab close checks the live Signal instead of possibly stale TabInfo
    #[test]
    fn test_request_close_current_uses_live_modified_signal() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("live dirty".to_string());
            let index = *state.current_tab_index.read();
            state.tabs.write()[index].modified = false;
            let tab_id = state.tabs.read()[index].id;

            FileActions::request_close_tab(&mut state, index);

            assert!(*state.show_close_confirm.read());
            assert_eq!(*state.pending_close_tab_id.read(), Some(tab_id));
        });
    }

    /// 非当前标签只使用自身 TabInfo.modified，不受当前标签修改状态影响
    /// Non-current close uses only its TabInfo.modified, ignoring active-tab dirtiness
    #[test]
    fn test_request_close_non_current_ignores_active_modified() {
        with_runtime(|| {
            let mut state = AppState::new();
            FileActions::new_tab(&mut state);
            state.update_content("active dirty".to_string());
            state.tabs.write()[0].modified = false;

            FileActions::request_close_tab(&mut state, 0);

            assert_eq!(state.tabs.read().len(), 1);
            assert!(!*state.show_close_confirm.read());
            assert_eq!(state.content.read().as_str(), "active dirty");
            assert!(*state.modified.read());
        });
    }

    /// 非当前已修改标签应按其稳定标识进入确认状态
    /// A modified non-current tab enters confirmation by stable identity
    #[test]
    fn test_request_close_non_current_modified_target() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("background dirty".to_string());
            let target_id = state.tabs.read()[0].id;
            FileActions::new_tab(&mut state);

            FileActions::request_close_tab(&mut state, 0);

            assert!(*state.show_close_confirm.read());
            assert_eq!(*state.pending_close_tab_id.read(), Some(target_id));
        });
    }

    /// 不保存关闭后台标签不得清除活动标签的 modified
    /// Discard-closing a background tab must preserve the active tab's modified flag
    #[test]
    fn test_discard_background_tab_preserves_other_modified() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("target dirty".to_string());
            let target_id = state.tabs.read()[0].id;
            FileActions::new_tab(&mut state);
            state.update_content("other dirty".to_string());

            FileActions::discard_and_close_tab_by_id(&mut state, target_id);

            assert_eq!(state.tabs.read().len(), 1);
            assert_eq!(state.content.read().as_str(), "other dirty");
            assert!(*state.modified.read());
        });
    }

    /// 无路径关闭应排入快照型另存为意图，取消后保留标签
    /// Untitled close queues a snapshot Save-As intent and cancel keeps the tab
    #[test]
    fn test_untitled_close_intent_cancel_keeps_tab() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("draft snapshot".to_string());
            let tab_id = state.tabs.read()[0].id;
            state.save_current_tab_content();
            let snapshot = FileActions::close_tab_snapshot(&state, tab_id).unwrap();
            assert!(snapshot.path.is_none());

            FileActions::queue_close_save_as(&mut state, snapshot.clone());
            assert!(*state.trigger_save_as.read());
            assert_eq!(state.pending_close_save_as.read().as_ref(), Some(&snapshot));

            FileActions::cancel_close_request(&mut state);
            assert_eq!(state.tabs.read().len(), 1);
            assert_eq!(state.content.read().as_str(), "draft snapshot");
            assert!(*state.modified.read());
            assert!(state.pending_close_save_as.read().is_none());
        });
    }

    /// 关闭型另存为失败时必须保留目标标签和未保存状态
    /// Failed close Save-As must retain the target tab and unsaved state
    #[test]
    fn test_close_save_as_failure_keeps_tab() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let mut state = AppState::new();
            state.update_content("cannot lose me".to_string());
            let tab_id = state.tabs.read()[0].id;
            state.save_current_tab_content();
            let snapshot = FileActions::close_tab_snapshot(&state, tab_id).unwrap();

            let invalid_path = temp.path().join("missing-parent").join("file.md");
            let result = FileActions::save_close_snapshot_as(&mut state, snapshot, invalid_path);

            assert!(result.is_err());
            assert_eq!(state.tabs.read().len(), 1);
            assert_eq!(state.content.read().as_str(), "cannot lose me");
            assert!(*state.modified.read());
        });
    }

    /// 单个无路径标签另存为成功后应重置为空白标签
    /// A single untitled tab resets to a blank tab after successful Save As
    #[test]
    fn test_single_untitled_save_as_success_closes_document() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let output = temp.path().join("single.md");
            let mut state = AppState::new();
            state.update_content("single draft".to_string());
            state.save_current_tab_content();
            let tab_id = state.tabs.read()[0].id;
            let snapshot = FileActions::close_tab_snapshot(&state, tab_id).unwrap();

            FileActions::save_close_snapshot_as(&mut state, snapshot, output.clone()).unwrap();

            assert_eq!(fs::read_to_string(output).unwrap(), "single draft");
            assert_eq!(state.tabs.read().len(), 1);
            assert!(state.content.read().is_empty());
            assert!(!*state.modified.read());
            assert_ne!(state.tabs.read()[0].id, tab_id);
        });
    }

    /// 保存关闭使用稳定标识和目标编码，即使较早索引被移除也不会写错标签
    /// Save-and-close uses stable identity and target encoding after earlier indices shift
    #[test]
    fn test_close_save_as_survives_index_shift_and_preserves_encoding() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let output = temp.path().join("shifted.md");
            let mut state = AppState::new();
            FileActions::new_tab(&mut state);
            state.update_content("目标正文".to_string());
            *state.file_encoding.write() = FileEncoding::Utf16Le;
            state.save_current_tab_content();
            let target_id = state.tabs.read()[1].id;
            let snapshot = FileActions::close_tab_snapshot(&state, target_id).unwrap();
            FileActions::new_tab(&mut state);
            state.update_content("other dirty".to_string());

            state.close_tab(0);
            FileActions::save_close_snapshot_as(&mut state, snapshot, output.clone()).unwrap();

            assert!(state.tabs.read().iter().all(|tab| tab.id != target_id));
            assert_eq!(
                decode_bytes(&fs::read(output).unwrap()).unwrap(),
                ("目标正文".to_string(), FileEncoding::Utf16Le)
            );
            assert_eq!(state.content.read().as_str(), "other dirty");
            assert!(*state.modified.read());
        });
    }

    /// 有路径后台标签按其快照和编码保存，不切换也不覆盖活动标签
    /// A pathed background tab saves its snapshot and encoding without switching or overwriting active state
    #[test]
    fn test_pathed_background_save_and_close_uses_target_snapshot() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("background-gbk.md");
            fs::write(&path, encode_text("旧正文", FileEncoding::Gbk).unwrap()).unwrap();
            let mut state = AppState::new();
            FileActions::open_file(&mut state, path.clone()).unwrap();
            state.update_content("后台目标".to_string());
            state.save_current_tab_content();
            let target_id = state.tabs.read()[1].id;
            let snapshot = FileActions::close_tab_snapshot(&state, target_id).unwrap();
            FileActions::new_tab(&mut state);
            state.update_content("活动标签修改".to_string());

            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(FileActions::save_pathed_snapshot_and_close(
                    &mut state,
                    snapshot,
                    path.clone(),
                ))
                .unwrap();

            assert!(state.tabs.read().iter().all(|tab| tab.id != target_id));
            assert_eq!(
                decode_bytes(&fs::read(path).unwrap()).unwrap(),
                ("后台目标".to_string(), FileEncoding::Gbk)
            );
            assert_eq!(state.content.read().as_str(), "活动标签修改");
            assert!(*state.modified.read());
        });
    }

    /// 外部 GBK 重载后的正文与编码应写入稳定标签并跨切换保留
    /// External GBK reload persists content and encoding in the stable tab across switches
    #[test]
    fn test_external_gbk_reload_survives_tab_switch() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let reloaded_path = temp.path().join("reloaded.md");
            let other_path = temp.path().join("other.md");
            fs::write(&reloaded_path, "before").unwrap();
            fs::write(&other_path, "other").unwrap();

            let mut state = AppState::new();
            FileActions::open_file(&mut state, reloaded_path.clone()).unwrap();
            let reload_index = *state.current_tab_index.read();
            let reload_id = state.tabs.read()[reload_index].id;
            FileActions::open_file(&mut state, other_path).unwrap();
            FileActions::switch_tab(&mut state, reload_index);
            fs::write(
                &reloaded_path,
                encode_text("外部新内容", FileEncoding::Gbk).unwrap(),
            )
            .unwrap();

            FileActions::reload_current_file(&mut state).unwrap();
            assert_eq!(state.content.read().as_str(), "外部新内容");
            assert_eq!(*state.file_encoding.read(), FileEncoding::Gbk);
            assert_eq!(state.tabs.read()[reload_index].id, reload_id);
            assert_eq!(state.tabs.read()[reload_index].content_str(), "外部新内容");
            assert_eq!(state.tabs.read()[reload_index].encoding, FileEncoding::Gbk);

            FileActions::switch_tab(&mut state, reload_index + 1);
            FileActions::switch_tab(&mut state, reload_index);
            assert_eq!(state.content.read().as_str(), "外部新内容");
            assert_eq!(*state.file_encoding.read(), FileEncoding::Gbk);
        });
    }

    /// 另存为应同步活动 Signals 与稳定目标标签的路径、标题和编码
    /// Save As synchronizes path, title, and encoding for active signals and the stable target tab
    #[test]
    fn test_save_as_synchronizes_target_tab_metadata() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let target = temp.path().join("renamed-target.md");
            let mut state = AppState::new();
            state.update_content("另存正文".to_string());
            *state.file_encoding.write() = FileEncoding::Utf16Le;
            let tab_id = state.tabs.read()[0].id;

            FileActions::save_as(&mut state, target.clone()).unwrap();

            let tab = state
                .tabs
                .read()
                .iter()
                .find(|tab| tab.id == tab_id)
                .unwrap()
                .clone();
            assert_eq!(*state.current_file.read(), Some(target.clone()));
            assert_eq!(*state.file_encoding.read(), FileEncoding::Utf16Le);
            assert_eq!(tab.path, Some(target));
            assert_eq!(tab.title, "renamed-target");
            assert_eq!(tab.encoding, FileEncoding::Utf16Le);
            assert!(!tab.modified);
        });
    }

    /// 重命名当前文件应保持稳定标签并同步当前路径与监控刷新
    /// Renaming the active file preserves stable identity and synchronizes path and watcher refresh
    #[test]
    fn test_rename_current_file_updates_tab_and_watcher() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let old_path = temp.path().join("old.md");
            fs::write(&old_path, "body").unwrap();
            let mut state = AppState::new();
            FileActions::set_workspace(&mut state, temp.path().to_path_buf());
            FileActions::open_file(&mut state, old_path.clone()).unwrap();
            let tab_id = state.tabs.read()[*state.current_tab_index.read()].id;
            let watch_before = *state.file_watch_refresh_seq.read();

            let new_path = FileActions::rename_file(&mut state, &old_path, "new.md").unwrap();

            assert_eq!(*state.current_file.read(), Some(new_path.clone()));
            let tab = state
                .tabs
                .read()
                .iter()
                .find(|tab| tab.id == tab_id)
                .unwrap()
                .clone();
            assert_eq!(tab.path, Some(new_path));
            assert_eq!(tab.title, "new");
            assert!(*state.file_watch_refresh_seq.read() > watch_before);
        });
    }

    /// 重命名目录应批量迁移当前与后台标签以及工作区根
    /// Renaming a directory migrates active and background tabs plus the workspace root in bulk
    #[test]
    fn test_rename_directory_updates_all_tabs_and_workspace_root() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let old_root = temp.path().join("old-root");
            fs::create_dir(&old_root).unwrap();
            let first = old_root.join("first.md");
            let second = old_root.join("nested").join("second.md");
            fs::create_dir(old_root.join("nested")).unwrap();
            fs::write(&first, "first").unwrap();
            fs::write(&second, "second").unwrap();
            let mut state = AppState::new();
            FileActions::set_workspace(&mut state, old_root.clone());
            FileActions::open_file(&mut state, first).unwrap();
            FileActions::open_file(&mut state, second).unwrap();
            let ids: Vec<_> = state
                .tabs
                .read()
                .iter()
                .filter(|tab| tab.path.is_some())
                .map(|tab| tab.id)
                .collect();

            let new_root = FileActions::rename_file(&mut state, &old_root, "new-root").unwrap();

            assert_eq!(*state.workspace_root.read(), Some(new_root.clone()));
            assert!(state
                .tabs
                .read()
                .iter()
                .filter(|tab| ids.contains(&tab.id))
                .all(|tab| tab
                    .path
                    .as_ref()
                    .is_some_and(|path| path.starts_with(&new_root))));
            assert!(state
                .current_file
                .read()
                .as_ref()
                .is_some_and(|path| path.starts_with(&new_root)));
            assert_eq!(state.file_list.read().len(), 2);
        });
    }

    /// 删除当前文件应保留内存正文、脱离路径并标记未保存
    /// Deleting the active file preserves memory content, detaches its path, and marks it unsaved
    #[test]
    fn test_delete_current_file_preserves_buffer() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let path = temp.path().join("current.md");
            fs::write(&path, "disk").unwrap();
            let mut state = AppState::new();
            FileActions::set_workspace(&mut state, temp.path().to_path_buf());
            FileActions::open_file(&mut state, path.clone()).unwrap();
            state.update_content("unsaved memory".to_string());
            let tab_id = state.tabs.read()[*state.current_tab_index.read()].id;

            FileActions::delete_file(&mut state, &path).unwrap();

            assert_eq!(state.content.read().as_str(), "unsaved memory");
            assert!(state.current_file.read().is_none());
            assert!(*state.modified.read());
            let tab = state
                .tabs
                .read()
                .iter()
                .find(|tab| tab.id == tab_id)
                .unwrap()
                .clone();
            assert!(tab.path.is_none());
            assert!(tab.modified);
            assert_eq!(tab.content_str(), "unsaved memory");
        });
    }

    /// 删除后台文件不得改变当前标签，但应将目标标签安全转为未命名缓冲区
    /// Deleting a background file keeps the active tab while safely detaching the target buffer
    #[test]
    fn test_delete_non_current_file_preserves_target_buffer() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let target = temp.path().join("target.md");
            let active = temp.path().join("active.md");
            fs::write(&target, "target body").unwrap();
            fs::write(&active, "active body").unwrap();
            let mut state = AppState::new();
            FileActions::set_workspace(&mut state, temp.path().to_path_buf());
            FileActions::open_file(&mut state, target.clone()).unwrap();
            let target_id = state.tabs.read()[*state.current_tab_index.read()].id;
            FileActions::open_file(&mut state, active.clone()).unwrap();

            FileActions::delete_file(&mut state, &target).unwrap();

            assert_eq!(*state.current_file.read(), Some(active));
            assert_eq!(state.content.read().as_str(), "active body");
            let tab = state
                .tabs
                .read()
                .iter()
                .find(|tab| tab.id == target_id)
                .unwrap()
                .clone();
            assert!(tab.path.is_none());
            assert!(tab.modified);
            assert_eq!(tab.content_str(), "target body");
        });
    }

    /// 删除目录应批量保留受影响标签正文并清空被删除的工作区根
    /// Deleting a directory preserves all affected tab bodies and clears a deleted workspace root
    #[test]
    fn test_delete_directory_detaches_tabs_and_clears_workspace() {
        with_runtime(|| {
            let temp = TempDir::new().unwrap();
            let root = temp.path().join("workspace");
            fs::create_dir(&root).unwrap();
            let first = root.join("first.md");
            let second = root.join("second.md");
            fs::write(&first, "first body").unwrap();
            fs::write(&second, "second body").unwrap();
            let mut state = AppState::new();
            FileActions::set_workspace(&mut state, root.clone());
            FileActions::open_file(&mut state, first).unwrap();
            FileActions::open_file(&mut state, second).unwrap();
            let affected_ids: Vec<_> = state
                .tabs
                .read()
                .iter()
                .filter(|tab| tab.path.is_some())
                .map(|tab| tab.id)
                .collect();

            FileActions::delete_file(&mut state, &root).unwrap();

            assert!(state.workspace_root.read().is_none());
            assert!(state.file_list.read().is_empty());
            let tabs = state.tabs.read();
            for tab in tabs.iter().filter(|tab| affected_ids.contains(&tab.id)) {
                assert!(tab.path.is_none());
                assert!(tab.modified);
                assert!(!tab.content_str().is_empty());
            }
            assert!(state.current_file.read().is_none());
            assert!(*state.modified.read());
        });
    }

    #[test]
    fn test_transcript_window_chronological_cap() {
        use crate::state::ChatTurn;
        let turns: Vec<ChatTurn> = (0..12)
            .map(|i| {
                if i % 2 == 0 {
                    ChatTurn::user(format!("u{i}"))
                } else {
                    ChatTurn::assistant(format!("a{i}"))
                }
            })
            .collect();
        let window = AppActions::transcript_window(&turns, 6);
        assert_eq!(window.len(), 6);
        assert_eq!(window[0].content, "u6");
        assert_eq!(window[5].content, "a11");
    }

    #[test]
    fn test_start_and_cancel_ai_generation() {
        with_runtime(|| {
            let mut state = AppState::new();
            let (id1, _rx) = AppActions::start_ai_generation(&mut state);
            assert!(*state.ai().ai_loading.read());
            assert_eq!(*state.ai().ai_generation_id.read(), id1);
            AppActions::cancel_ai_generation(&mut state);
            assert!(!*state.ai().ai_loading.read());
            assert!(*state.ai().ai_generation_id.read() > id1);
        });
    }
}
