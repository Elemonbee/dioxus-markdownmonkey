//! 文件工具函数 / File Utility Functions
//!
//! 提供共享的文件扫描逻辑，避免代码重复
//! Provides shared file scanning logic to avoid code duplication

use std::fs;
use std::path::{Path, PathBuf};

/// 目录扫描最大深度 / Maximum directory scan depth
const MAX_SCAN_DEPTH: usize = 10;
/// 目录扫描最大文件数 / Maximum files per scan
const MAX_SCAN_FILES: usize = 1000;

/// 扫描目录中的 Markdown 文件 / Scan Markdown files in directory
pub fn scan_markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    scan_dir_recursive(dir, &mut files, 0);
    files
}

/// 递归扫描目录（带深度和数量限制）/ Recursively scan directory (with depth and count limits)
fn scan_dir_recursive(dir: &Path, files: &mut Vec<PathBuf>, depth: usize) {
    // 超过最大深度或最大文件数时停止 / Stop when exceeding limits
    if depth >= MAX_SCAN_DEPTH || files.len() >= MAX_SCAN_FILES {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        // 文件夹优先 / Directories first
        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            b_is_dir.cmp(&a_is_dir)
        });

        for entry in entries {
            // 超过最大文件数时停止 / Stop when exceeding max file count
            if files.len() >= MAX_SCAN_FILES {
                break;
            }

            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // 跳过隐藏文件和特殊目录 / Skip hidden files and special directories
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }

            // 跳过符号链接以防止循环引用导致无限递归
            // Skip symlinks to prevent infinite recursion from circular references
            if path.is_dir() && !path.is_symlink() {
                scan_dir_recursive(&path, files, depth + 1);
            } else if let Some(ext) = path.extension() {
                if ext == "md" || ext == "markdown" || ext == "txt" {
                    files.push(path);
                }
            }
        }
    }
}
