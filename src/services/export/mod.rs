//! 导出服务 / Export Service
//!
//! 支持 HTML 与纯文本导出 / HTML and plain-text export

mod html;
mod shared;
mod text;

pub use html::HtmlExportOptions;
pub use shared::ExportError;

/// 导出服务 / Export Service
pub struct ExportService;

impl ExportService {
    /// 导出为 HTML / Export to HTML
    #[allow(dead_code)] // 测试与无资源目录场景 / Tests and no-assets export path
    pub fn export_to_html(
        markdown_content: &str,
        output_path: &std::path::Path,
    ) -> Result<(), ExportError> {
        html::export_to_html(markdown_content, output_path)
    }

    /// 导出 HTML 并打包本地图片 / Export HTML and bundle local images
    #[allow(dead_code)] // 测试与无外观选项场景 / Tests and default-style export path
    pub fn export_to_html_with_assets(
        markdown_content: &str,
        output_path: &std::path::Path,
        source_dir: Option<&std::path::Path>,
    ) -> Result<(), ExportError> {
        html::export_to_html_with_assets(markdown_content, output_path, source_dir)
    }

    /// 按语言与主题导出 HTML 并打包本地图片
    /// Export HTML with language/theme and bundle local images
    pub fn export_to_html_with_options(
        markdown_content: &str,
        output_path: &std::path::Path,
        source_dir: Option<&std::path::Path>,
        options: &HtmlExportOptions,
    ) -> Result<(), ExportError> {
        html::export_to_html_with_options(markdown_content, output_path, source_dir, options)
    }

    /// 导出为纯文本 / Export to Plain Text
    pub fn export_to_text(content: &str, output_path: &std::path::Path) -> Result<(), ExportError> {
        text::export_to_text(content, output_path)
    }

    /// 生成打印用 HTML（本地图嵌 data URI，不写盘）
    /// Build print HTML with local images as data URIs, without writing a file
    pub fn render_html_for_print(
        markdown_content: &str,
        source_dir: Option<&std::path::Path>,
        options: &HtmlExportOptions,
    ) -> Result<String, ExportError> {
        html::render_html_for_print(markdown_content, source_dir, options)
    }
}
