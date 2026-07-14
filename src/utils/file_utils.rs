//! 文件工具函数 / File Utility Functions
//!
//! 提供共享的文件扫描逻辑，避免代码重复
//! Provides shared file scanning logic to avoid code duplication

use std::fs;
use std::path::{Component, Path, PathBuf};

/// 目录扫描最大深度 / Maximum directory scan depth
const MAX_SCAN_DEPTH: usize = 10;
/// 目录扫描最大文件数 / Maximum files per scan
const MAX_SCAN_FILES: usize = 1000;

/// 校验重命名目标名：禁止路径分隔符与 `..` 穿越
/// Validate rename target name: reject separators and `..` traversal
pub fn validate_rename_name(new_name: &str) -> Result<(), String> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err("名称不能为空 / Name cannot be empty".to_string());
    }
    if trimmed == "." || trimmed == ".." {
        return Err("非法名称 / Invalid name".to_string());
    }
    let path = Path::new(trimmed);
    if path.components().count() != 1 {
        return Err("名称不能包含路径分隔符 / Name must be a single path segment".to_string());
    }
    if path
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::RootDir))
    {
        return Err("名称不能包含 '..' / Name cannot contain '..'".to_string());
    }
    if trimmed.contains(['/', '\\']) {
        return Err("名称不能包含路径分隔符 / Name must be a single path segment".to_string());
    }
    Ok(())
}

/// 确保路径位于工作区内（canonicalize 后前缀校验）
/// Ensure path stays inside the workspace (prefix check after canonicalize)
pub fn ensure_within_workspace(path: &Path, workspace: &Path) -> Result<PathBuf, String> {
    let workspace_canon = workspace
        .canonicalize()
        .map_err(|e| format!("无法解析工作区路径 / Cannot resolve workspace path: {}", e))?;

    // 已存在路径直接 canonicalize；否则 canonicalize 父目录再拼接文件名
    // Canonicalize existing paths; otherwise canonicalize parent then join filename
    let candidate = if path.exists() {
        path.canonicalize()
            .map_err(|e| format!("无法解析路径 / Cannot resolve path: {}", e))?
    } else {
        let parent = path
            .parent()
            .ok_or_else(|| "无效路径 / Invalid path".to_string())?;
        let file_name = path
            .file_name()
            .ok_or_else(|| "无效文件名 / Invalid file name".to_string())?;
        let parent_canon = parent
            .canonicalize()
            .map_err(|e| format!("无法解析父目录 / Cannot resolve parent directory: {}", e))?;
        parent_canon.join(file_name)
    };

    if !candidate.starts_with(&workspace_canon) {
        return Err("路径超出工作区范围 / Path escapes the workspace boundary".to_string());
    }
    Ok(candidate)
}

/// 是否为可打开的 Markdown/文本文件
/// Whether the path is an openable markdown/text file
pub fn is_markdown_or_text_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            e.eq_ignore_ascii_case("md")
                || e.eq_ignore_ascii_case("markdown")
                || e.eq_ignore_ascii_case("txt")
        })
        .unwrap_or(false)
}

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
