//! 导出服务 / Export Service
//!
//! 支持 HTML 与纯文本导出 / HTML and plain-text export

mod html;
mod shared;
mod text;

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
    pub fn export_to_html_with_assets(
        markdown_content: &str,
        output_path: &std::path::Path,
        source_dir: Option<&std::path::Path>,
    ) -> Result<(), ExportError> {
        html::export_to_html_with_assets(markdown_content, output_path, source_dir)
    }

    /// 导出为纯文本 / Export to Plain Text
    pub fn export_to_text(content: &str, output_path: &std::path::Path) -> Result<(), ExportError> {
        text::export_to_text(content, output_path)
    }
}
