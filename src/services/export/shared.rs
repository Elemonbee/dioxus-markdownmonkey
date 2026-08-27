//! 导出共享类型和工具函数 / Export shared types and utilities

use std::fmt;
use std::path::{Path, PathBuf};

/// 导出错误类型 / Export error types
#[derive(Debug)]
pub enum ExportError {
    /// 读写失败 / IO failure
    Io(std::io::Error),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO 错误/IO Error: {e}"),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
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
