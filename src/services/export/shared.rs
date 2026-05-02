//! 导出共享类型和工具函数 / Export shared types and utilities

use thiserror::Error;

/// 导出错误类型 / Export Error Types
#[derive(Error, Debug)]
pub enum ExportError {
    #[error("PDF 导出错误/PDF Export Error: {0}")]
    Pdf(String),

    #[error("IO 错误/IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("渲染错误/Render Error: {0}")]
    Render(String),

    #[error("字体错误/Font Error: {0}")]
    Font(String),

    #[error("ZIP 错误/ZIP Error: {0}")]
    Zip(String),
}

impl From<zip::result::ZipError> for ExportError {
    fn from(e: zip::result::ZipError) -> Self {
        ExportError::Zip(e.to_string())
    }
}

impl From<printpdf::Error> for ExportError {
    fn from(e: printpdf::Error) -> Self {
        ExportError::Pdf(e.to_string())
    }
}

/// PDF 导出配置 / PDF Export Configuration
#[derive(Clone, Debug)]
pub struct PdfExportConfig {
    /// 页面宽度 (mm) / Page width (mm)
    pub page_width: f32,
    /// 页面高度 (mm) / Page height (mm)
    pub page_height: f32,
    /// 边距 (mm) / Margin (mm)
    pub margin: f32,
    /// 字体大小 / Font size
    pub font_size: f32,
    /// 行高倍数 / Line height multiplier
    pub line_height_multiplier: f32,
}

impl Default for PdfExportConfig {
    fn default() -> Self {
        Self {
            page_width: 210.0,
            page_height: 297.0,
            margin: 20.0,
            font_size: 11.0,
            line_height_multiplier: 1.4,
        }
    }
}

/// 检查是否为 CJK 字符 / Check if character is CJK
pub fn is_cjk_char(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |
        '\u{3400}'..='\u{4DBF}' |
        '\u{20000}'..='\u{2A6DF}' |
        '\u{F900}'..='\u{FAFF}' |
        '\u{2F800}'..='\u{2FA1F}' |
        '\u{3000}'..='\u{303F}' |
        '\u{FF00}'..='\u{FFEF}'
    )
}

/// 检查文本是否包含 CJK 字符 / Check if text contains CJK characters
pub fn contains_cjk(text: &str) -> bool {
    text.chars().any(is_cjk_char)
}

/// 转义 XML 特殊字符 / Escape XML special characters
pub fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
