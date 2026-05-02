#![allow(dead_code)]
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
    pub fn export_to_pdf(content: &str, output_path: &std::path::Path) -> Result<(), ExportError> {
        pdf::export_to_pdf(content, output_path)
    }

    /// 使用自定义配置导出为 PDF / Export to PDF with custom configuration
    pub fn export_to_pdf_with_config(
        content: &str,
        output_path: &std::path::Path,
        config: PdfExportConfig,
    ) -> Result<(), ExportError> {
        pdf::export_to_pdf_with_config(content, output_path, config)
    }

    /// 导出为 HTML / Export to HTML
    pub fn export_to_html(
        markdown_content: &str,
        output_path: &std::path::Path,
    ) -> Result<(), ExportError> {
        html::export_to_html(markdown_content, output_path)
    }

    /// 导出为纯文本 / Export to Plain Text
    pub fn export_to_text(content: &str, output_path: &std::path::Path) -> Result<(), ExportError> {
        text::export_to_text(content, output_path)
    }

    /// 导出为 Word/Docx 格式 / Export to Word/Docx format
    pub fn export_to_docx(content: &str, output_path: &std::path::Path) -> Result<(), ExportError> {
        docx::export_to_docx(content, output_path)
    }
}
