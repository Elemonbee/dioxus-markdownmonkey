//! 导出服务 / Export Service
//!
//! 支持 PDF、HTML、DOCX、纯文本导出
//! Support PDF, HTML, DOCX, plain text export
//!
//! PDF 导出使用系统字体支持中文 / PDF export uses system fonts for Chinese support

mod docx;
mod html;
mod pdf;
mod shared;
mod text;

pub use shared::{ExportError, PdfExportConfig};

/// 导出服务 / Export Service
pub struct ExportService;

impl ExportService {
    /// 导出为 PDF / Export to PDF
    #[allow(dead_code)] // 测试与无配置默认导出 / Used by tests and default no-config export
    pub fn export_to_pdf(content: &str, output_path: &std::path::Path) -> Result<(), ExportError> {
        pdf::export_to_pdf(content, output_path)
    }

    /// 使用自定义配置导出为 PDF / Export to PDF with custom configuration
    #[allow(dead_code)] // 无资源目录场景与测试入口 / No-assets path and test entry
    pub fn export_to_pdf_with_config(
        content: &str,
        output_path: &std::path::Path,
        config: PdfExportConfig,
    ) -> Result<(), ExportError> {
        pdf::export_to_pdf_with_config(content, output_path, config)
    }

    /// 导出 PDF 并嵌入本地图片 / Export PDF and embed local images
    pub fn export_to_pdf_with_assets(
        content: &str,
        output_path: &std::path::Path,
        config: PdfExportConfig,
        source_dir: Option<&std::path::Path>,
    ) -> Result<(), ExportError> {
        pdf::export_to_pdf_with_assets(content, output_path, config, source_dir)
    }

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

    /// 导出为 Word/Docx 格式 / Export to Word/Docx format
    #[allow(dead_code)] // 测试与无资源目录场景 / Tests and no-assets path
    pub fn export_to_docx(content: &str, output_path: &std::path::Path) -> Result<(), ExportError> {
        docx::export_to_docx(content, output_path)
    }

    /// 导出 DOCX 并嵌入本地图片 / Export DOCX and embed local images
    pub fn export_to_docx_with_assets(
        content: &str,
        output_path: &std::path::Path,
        source_dir: Option<&std::path::Path>,
    ) -> Result<(), ExportError> {
        docx::export_to_docx_with_assets(content, output_path, source_dir)
    }
}
