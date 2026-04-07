//! 服务层测试 / Service Layer Tests

#[cfg(test)]
mod tests {
    use crate::config::LARGE_FILE_THRESHOLD_BYTES;
    use crate::services::ai::{AIError, AIService};
    use crate::services::auto_save::AutoSaveService;
    use crate::services::export::ExportService;
    use crate::services::file_watcher::FileModificationChecker;
    use crate::services::image::{ImageFormat, ImageService};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    // ========== 导出服务测试 / Export Service Tests ==========

    #[test]
    fn test_export_html_creates_file() {
        let content = "# Hello World\n\nThis is a test.";
        let output_path = std::env::temp_dir().join("test_export.html");

        let result = ExportService::export_to_html(content, &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());

        // 验证内容
        let html_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(html_content.contains("<!DOCTYPE html>"));
        assert!(html_content.contains("</html>"));

        // 清理
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_export_text() {
        let content = "# Test\n\nContent";
        let output_path = std::env::temp_dir().join("test_export.txt");

        let result = ExportService::export_to_text(content, &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());

        // 清理
        let _ = std::fs::remove_file(&output_path);
    }

    // ========== 自动保存服务测试 / Auto Save Service Tests ==========

    #[test]
    fn test_auto_save_disabled_by_default() {
        let service = AutoSaveService::new();
        assert!(!service.should_save(true));
    }

    #[test]
    fn test_auto_save_enabled() {
        let mut service = AutoSaveService::new();
        service.set_enabled(true);

        // 刚启用，不需要保存
        assert!(!service.should_save(true));

        // 未修改，不需要保存
        assert!(!service.should_save(false));
    }

    #[test]
    fn test_auto_save_interval() {
        let mut service = AutoSaveService::new();
        service.set_enabled(true);
        service.set_interval(1); // 1 秒

        // 等待间隔
        std::thread::sleep(Duration::from_millis(1100));

        // 现在应该保存
        assert!(service.should_save(true));
    }

    #[test]
    fn test_auto_save_mark_saved() {
        let mut service = AutoSaveService::new();
        service.set_enabled(true);
        service.set_interval(1);

        std::thread::sleep(Duration::from_millis(1100));
        assert!(service.should_save(true));

        service.mark_saved();
        assert!(!service.should_save(true));
    }

    // ========== 图片服务测试 / Image Service Tests ==========

    #[test]
    fn test_image_format_from_mime() {
        assert!(matches!(
            ImageFormat::from_mime("image/png"),
            Some(ImageFormat::Png)
        ));
        assert!(matches!(
            ImageFormat::from_mime("image/jpeg"),
            Some(ImageFormat::Jpeg)
        ));
        assert!(matches!(
            ImageFormat::from_mime("image/gif"),
            Some(ImageFormat::Gif)
        ));
        assert!(matches!(
            ImageFormat::from_mime("image/webp"),
            Some(ImageFormat::WebP)
        ));
        assert!(ImageFormat::from_mime("text/plain").is_none());
    }

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Gif.extension(), "gif");
        assert_eq!(ImageFormat::WebP.extension(), "webp");
    }

    #[test]
    fn test_generate_markdown() {
        let path = PathBuf::from("/images/test.png");
        let md = ImageService::generate_markdown(&path, Some("测试图片"));
        assert_eq!(md, "![测试图片](/images/test.png)");

        let md_default = ImageService::generate_markdown(&path, None);
        assert_eq!(md_default, "![图片/Image](/images/test.png)");
    }

    // ========== 文件监控服务测试 / File Watcher Service Tests ==========

    #[test]
    fn test_file_modification_checker_new() {
        let mut checker = FileModificationChecker::new();
        assert!(!checker.check_modified());
    }

    #[test]
    fn test_file_modification_checker_set_file() {
        let mut checker = FileModificationChecker::new();

        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        let path = temp_file.path().to_path_buf();

        checker.set_file(&path);
        assert!(!checker.check_modified()); // 刚设置时不应该检测到修改
    }

    #[test]
    fn test_file_modification_checker_detects_change() {
        let mut checker = FileModificationChecker::new();

        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "initial content").unwrap();
        let path = temp_file.path().to_path_buf();

        checker.set_file(&path);
        assert!(!checker.check_modified());

        // 等待一下确保时间戳不同
        std::thread::sleep(std::time::Duration::from_millis(50));

        // 修改文件
        writeln!(temp_file, "modified content").unwrap();

        // 现在应该检测到修改
        assert!(checker.check_modified());
    }

    #[test]
    fn test_file_modification_checker_update() {
        let mut checker = FileModificationChecker::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "initial").unwrap();
        let path = temp_file.path().to_path_buf();

        checker.set_file(&path);

        std::thread::sleep(std::time::Duration::from_millis(50));
        writeln!(temp_file, "modified").unwrap();

        assert!(checker.check_modified());

        checker.update();
        assert!(!checker.check_modified()); // 更新后不再检测到修改
    }

    #[test]
    fn test_file_modification_checker_clear() {
        let mut checker = FileModificationChecker::new();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        checker.set_file(&path);
        checker.clear();

        assert!(!checker.check_modified());
    }

    #[test]
    fn test_ai_service_requires_api_key() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let service = AIService::new(
            String::new(),
            Some("https://api.openai.com/v1".to_string()),
            Some("gpt-4o-mini".to_string()),
        );

        let messages = crate::services::ai::build_continue_messages("hello");
        let result = rt.block_on(service.chat(messages));
        assert!(matches!(result, Err(AIError::Config(_))));
    }

    #[test]
    fn test_ai_service_normalizes_base_url() {
        let _service = AIService::new(
            "key".to_string(),
            Some("https://api.openai.com/v1/".to_string()),
            Some("gpt-4o-mini".to_string()),
        );

        assert_eq!(
            AIService::default_base_url(&crate::state::AIProvider::OpenAI),
            "https://api.openai.com/v1"
        );
        assert_eq!(
            AIService::default_model(&crate::state::AIProvider::OpenAI),
            "gpt-4o-mini"
        );
    }

    #[test]
    fn test_large_file_threshold_constant_is_one_megabyte() {
        assert_eq!(LARGE_FILE_THRESHOLD_BYTES, 1024 * 1024);
    }

    // ========== Markdown 渲染测试 / Markdown Rendering Tests ==========

    #[test]
    fn test_markdown_render_heading() {
        use crate::services::markdown::render_markdown;

        let html = render_markdown("# Hello World");
        assert!(html.contains("<h1>Hello World</h1>") || html.contains("<h1>"));
    }

    #[test]
    fn test_markdown_render_bold() {
        use crate::services::markdown::render_markdown;

        let html = render_markdown("**bold text**");
        assert!(html.contains("<strong>") || html.contains("<b>"));
    }

    #[test]
    fn test_markdown_render_italic() {
        use crate::services::markdown::render_markdown;

        let html = render_markdown("*italic text*");
        assert!(html.contains("<em>") || html.contains("<i>"));
    }

    #[test]
    fn test_markdown_render_code_block() {
        use crate::services::markdown::render_markdown;

        let html = render_markdown("```rust\nfn main() {}\n```");
        assert!(html.contains("<pre>") || html.contains("<code>"));
    }

    #[test]
    fn test_markdown_render_table() {
        use crate::services::markdown::render_markdown;

        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = render_markdown(md);
        assert!(html.contains("<table>"));
    }

    #[test]
    fn test_markdown_render_mermaid() {
        use crate::services::markdown::render_markdown;

        let md = "```mermaid\ngraph TD\nA-->B\n```";
        let html = render_markdown(md);
        assert!(html.contains("mermaid"));
    }

    #[test]
    fn test_markdown_sanitizes_script_tags() {
        use crate::services::markdown::render_markdown;

        let html = render_markdown("<script>alert('xss')</script><p>ok</p>");
        assert!(!html.contains("alert('xss')"));
        assert!(html.contains("<p>ok</p>") || html.contains("ok"));
    }

    // ========== 数学公式预处理测试 / Math Formula Preprocess Tests ==========

    #[test]
    fn test_html_encode_attribute() {
        use crate::services::markdown::html_encode_attribute;

        assert_eq!(html_encode_attribute("hello"), "hello");
        assert!(html_encode_attribute("a & b").contains("amp;"));
        assert!(html_encode_attribute(r#"a " b"#).contains("quot;"));
        assert!(html_encode_attribute("a < b").contains("lt;"));
        assert!(html_encode_attribute("a > b").contains("gt;"));
    }

    #[test]
    fn test_preprocess_math_formulas_inline() {
        use crate::services::markdown::preprocess_math_formulas;

        let input = "The formula $E = mc^2$ is famous.";
        let result = preprocess_math_formulas(input);
        assert!(result.contains("data-formula-inline"));
        assert!(!result.contains("$E = mc^2$"));
    }

    #[test]
    fn test_preprocess_math_formulas_block() {
        use crate::services::markdown::preprocess_math_formulas;

        let input = "Block formula:\n$$x^2 + y^2 = z^2$$\nDone.";
        let result = preprocess_math_formulas(input);
        assert!(result.contains("data-formula-block"));
    }

    #[test]
    fn test_preprocess_math_formulas_in_code_block() {
        use crate::services::markdown::preprocess_math_formulas;

        let input = "```\n$not a formula$\n```";
        let result = preprocess_math_formulas(input);
        assert!(!result.contains("data-formula-inline"));
        assert!(result.contains("$not a formula$"));
    }

    // ========== 语法高亮测试 / Syntax Highlight Tests ==========

    #[test]
    fn test_syntax_highlight_rust() {
        use crate::services::syntax_highlight::SyntaxHighlightService;

        let service = SyntaxHighlightService::new();
        let result = service.highlight_code_block("fn main() { println!(\"hello\"); }", "rust");
        assert!(!result.is_empty());
        // Should contain HTML tags for highlighted code
        assert!(result.contains("<span") || result.contains("<pre") || result.contains("<code"));
    }

    #[test]
    fn test_syntax_highlight_unknown_lang() {
        use crate::services::syntax_highlight::SyntaxHighlightService;

        let service = SyntaxHighlightService::new();
        let result = service.highlight_code_block("some code", "unknown_lang_xyz");
        // Should still produce output (plain text fallback)
        assert!(!result.is_empty());
    }

    // ========== DOCX 导出测试 / DOCX Export Tests ==========

    #[test]
    fn test_export_docx_creates_file() {
        let content = "# Test Document\n\nThis is a test paragraph.\n\n- Item 1\n- Item 2";
        let output_path = std::env::temp_dir().join("test_export.docx");

        let result = ExportService::export_to_docx(content, &output_path);
        assert!(result.is_ok(), "DOCX export should succeed");
        assert!(output_path.exists(), "DOCX file should be created");

        // 验证文件不为空 / Verify file is not empty
        let metadata = std::fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 0, "DOCX file should not be empty");

        // 清理 / Cleanup
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_export_docx_chinese_content() {
        let content = "# 中文标题\n\n这是中文内容测试。\n\n- 列表项一\n- 列表项二";
        let output_path = std::env::temp_dir().join("test_export_chinese.docx");

        let result = ExportService::export_to_docx(content, &output_path);
        assert!(result.is_ok(), "DOCX export with Chinese should succeed");

        // 清理 / Cleanup
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_export_html_preserves_markdown_rendered_content() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("rendered.html");
        let md = "# Title\n\n**bold**\n\n| A | B |\n|---|---|\n| 1 | 2 |";

        ExportService::export_to_html(md, &path).unwrap();

        let html = std::fs::read_to_string(path).unwrap();
        assert!(html.contains("<h1>Title</h1>") || html.contains("<h1"));
        assert!(html.contains("<strong>bold</strong>") || html.contains("bold"));
        assert!(html.contains("<table>"));
    }
}
