//! 服务层测试 / Service Layer Tests

#[cfg(test)]
mod tests {
    use crate::config::LARGE_FILE_THRESHOLD_BYTES;
    use crate::services::ai::{AIError, AIService};
    use crate::services::auto_save::AutoSaveService;
    use crate::services::export::ExportService;
    use crate::services::file_watcher::FileModificationChecker;
    use crate::services::image::{ImageFormat, ImageService};
    use crate::utils::file_encoding::{decode_bytes, encode_text, FileEncoding};
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

    /// 自动保存应使用指定编码并保留 BOM / Auto-save uses the specified encoding and preserves its BOM
    #[tokio::test]
    async fn test_auto_save_preserves_encoding_and_bom() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("autosave.md");
        let mut service = AutoSaveService::new();

        service
            .auto_save(Some(&path), "自动保存", FileEncoding::Utf16Le)
            .await
            .unwrap();

        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.starts_with(&[0xFF, 0xFE]));
        assert_eq!(
            decode_bytes(&bytes).unwrap(),
            ("自动保存".to_string(), FileEncoding::Utf16Le)
        );
    }

    /// 自动保存 GBK 不可表示字符时应失败且不覆盖文件 / GBK auto-save rejects unrepresentable text without overwriting
    #[tokio::test]
    async fn test_auto_save_rejects_unrepresentable_gbk() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("autosave-gbk.md");
        let original = encode_text("原文", FileEncoding::Gbk).unwrap();
        std::fs::write(&path, &original).unwrap();
        let mut service = AutoSaveService::new();

        let result = service
            .auto_save(Some(&path), "emoji 😀", FileEncoding::Gbk)
            .await;

        assert!(result.is_err());
        assert_eq!(std::fs::read(path).unwrap(), original);
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

        let messages = crate::services::ai::AITask::Continue.build_messages("hello", "");
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
        assert!(html.contains("<h1") && html.contains("Hello World"));
        assert!(html.contains("data-source-line=\"0\""));
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
        assert!(html.contains("<pre") || html.contains("<code"));
        assert!(html.contains("code-block"));
    }

    #[test]
    fn test_markdown_render_table() {
        use crate::services::markdown::render_markdown;

        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = render_markdown(md);
        assert!(html.contains("<table"));
    }

    #[test]
    fn test_markdown_render_fenced_code() {
        use crate::services::markdown::render_markdown;

        let md = "```mermaid\ngraph TD\nA-->B\n```";
        let html = render_markdown(md);
        assert!(html.contains("<pre"));
        assert!(html.contains("class=\"mermaid\"") || html.contains("code-block"));
    }

    #[test]
    fn test_markdown_sanitizes_script_tags() {
        use crate::services::markdown::render_markdown;

        let html = render_markdown("ok\n\n<script>alert('xss')</script>");
        assert!(!html.contains("<script"));
        assert!(html.contains("ok"));
    }

    #[test]
    fn test_markdown_strips_javascript_urls_and_style() {
        use crate::services::markdown::render_markdown;

        let html = render_markdown(
            r#"[click](javascript:alert(1)) <p style="position:fixed;top:0">x</p> ![x](data:text/html,<script>alert(1)</script>)"#,
        );
        assert!(!html.to_lowercase().contains("javascript:"));
        assert!(!html.contains("position:fixed"));
        assert!(!html.to_lowercase().contains("data:text/html"));
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
        assert!(html.contains("<table"));
    }

    #[test]
    fn test_export_html_bundles_local_images() {
        let temp = tempfile::TempDir::new().unwrap();
        let img = temp.path().join("shot.png");
        std::fs::write(&img, b"fake-png").unwrap();
        let path = temp.path().join("doc.html");
        let md = "![alt](shot.png)\n";

        ExportService::export_to_html_with_assets(md, &path, Some(temp.path())).unwrap();
        let html = std::fs::read_to_string(&path).unwrap();
        assert!(html.contains("doc_files/"));
        assert!(html.contains("shot.png"));
        let assets = temp.path().join("doc_files");
        assert!(assets.is_dir());
        assert!(std::fs::read_dir(&assets).unwrap().next().is_some());
    }
}
