//! 导出共享类型和工具函数 / Export shared types and utilities

use std::path::{Path, PathBuf};
use thiserror::Error;

/// 导出错误类型 / Export Error Types
#[derive(Error, Debug)]
#[allow(dead_code)] // 完整错误面：部分变体供 PDF/字体路径使用 / Full error surface for PDF/font paths
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
    /// 优先使用的 CJK 字体路径 / Preferred CJK font path
    pub cjk_font_path: Option<String>,
}

impl Default for PdfExportConfig {
    fn default() -> Self {
        Self {
            page_width: 210.0,
            page_height: 297.0,
            margin: 20.0,
            font_size: 11.0,
            line_height_multiplier: 1.4,
            cjk_font_path: None,
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

/// 判断是否为应跳过的远程/内联资源 / Whether a URL should be left untouched
pub fn is_remote_or_data_url(src: &str) -> bool {
    let lower = src.trim().to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("blob:")
        || lower.starts_with("//")
}

/// 相对路径相对文档目录解析；绝对路径原样使用
/// Resolve relative paths against the document directory; keep absolute paths
pub fn resolve_image_path(base: &Path, src: &str) -> Option<PathBuf> {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return None;
    }
    let cleaned = trimmed.split(['?', '#']).next().unwrap_or(trimmed);
    let path = PathBuf::from(cleaned);
    if path.is_absolute() {
        Some(path)
    } else {
        Some(base.join(path))
    }
}

/// 由扩展名推断图片 MIME / Infer image MIME from file extension
pub fn image_content_type(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => Some("image/png"),
        Some("jpg") | Some("jpeg") => Some("image/jpeg"),
        Some("gif") => Some("image/gif"),
        Some("bmp") => Some("image/bmp"),
        _ => None,
    }
}

/// 从 PNG IHDR 读取宽高 / Read PNG width/height from IHDR
pub fn png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() > 24 && data.starts_with(b"\x89PNG\r\n\x1a\n") {
        let w = u32::from_be_bytes(data[16..20].try_into().ok()?);
        let h = u32::from_be_bytes(data[20..24].try_into().ok()?);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }
    None
}

/// 解析整行独立图片语法 `![alt](path)`（远程 URL 返回 None）
/// Parse a standalone `![alt](path)` line (returns None for remote URLs)
pub fn parse_standalone_image(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with("![") {
        return None;
    }
    let rest = &trimmed[2..];
    let bracket_end = rest.find(']')?;
    let alt = rest[..bracket_end].to_string();
    let after = rest[bracket_end + 1..].strip_prefix('(')?;
    let paren_end = after.find(')')?;
    if !after[paren_end + 1..].trim().is_empty() {
        return None;
    }
    let path = after[..paren_end].trim().to_string();
    if path.is_empty() || is_remote_or_data_url(&path) {
        return None;
    }
    Some((alt, path))
}

/// 将像素尺寸转为 EMU（最大宽度约 6 英寸）/ Convert pixel size to EMUs (max ~6 inches wide)
pub fn image_size_emus(width_px: u32, height_px: u32) -> (i64, i64) {
    const MAX_CX: i64 = 5486400; // 6 inches
    let w = width_px.max(1) as i64;
    let h = height_px.max(1) as i64;
    // 96 DPI → EMU = px * 9525
    let cx = w * 9525;
    let cy = h * 9525;
    if cx <= MAX_CX {
        (cx, cy)
    } else {
        let scale = MAX_CX as f64 / cx as f64;
        (MAX_CX, ((cy as f64) * scale) as i64)
    }
}
