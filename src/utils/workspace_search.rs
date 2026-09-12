//! 工作区搜索与替换辅助 / Workspace search & replace helpers

use crate::config::{
    WORKSPACE_SEARCH_MAX_DEPTH, WORKSPACE_SEARCH_MAX_FILES, WORKSPACE_SEARCH_MAX_FILE_BYTES,
    WORKSPACE_SEARCH_MAX_RESULTS,
};
use crate::state::AppState;
use crate::utils::file_encoding::{read_file, write_file, FileEncoding};
use crate::utils::file_utils::is_markdown_or_text_path;
use crate::utils::replace::{count_matches, replace_all_in_text};
use dioxus::prelude::ReadableExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 工作区替换报告 / Workspace replace report
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceReplaceReport {
    pub files_touched: usize,
    pub matches_total: usize,
    pub errors: Vec<String>,
}

/// 搜索结果项 / Search result item
#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceSearchHit {
    pub path: PathBuf,
    pub line: usize,
    pub content: String,
    pub start: usize,
    pub end: usize,
}

/// 工作区文件快照（含写回编码）/ Workspace file snapshot (with write-back encoding)
#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceFile {
    pub path: PathBuf,
    pub content: String,
    pub encoding: FileEncoding,
}

/// 按编码读取工作区文本；失败时返回 None
/// Read workspace text with encoding detection; return None on failure
fn read_workspace_text(path: &Path) -> Option<(String, FileEncoding)> {
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > WORKSPACE_SEARCH_MAX_FILE_BYTES {
            return None;
        }
    }
    read_file(path).ok()
}

/// 从应用状态收集打开标签的路径→正文覆盖表
/// Collect path→body overrides from open tabs
///
/// 已被驱逐的非活动标签不写入覆盖表，搜索时回退到磁盘内容。
/// Evicted inactive tabs are omitted so search falls back to disk.
pub fn collect_open_buffer_overrides(state: &AppState) -> HashMap<PathBuf, String> {
    let doc = state.document();
    let active = *doc.current_tab_index.read();
    let tabs = doc.tabs.read();
    let mut map = HashMap::new();
    for (i, tab) in tabs.iter().enumerate() {
        let Some(ref path) = tab.path else {
            continue;
        };
        let body = if i == active {
            Some(doc.content.read().clone())
        } else {
            tab.content.as_ref().map(|c| c.to_string())
        };
        if let Some(body) = body {
            map.insert(path.clone(), body);
        }
    }
    map
}

/// 解析文件正文：优先打开标签缓冲，否则按编码读盘
/// Resolve file body: prefer an open-tab buffer, otherwise read disk with encoding detection
fn resolve_file_body(
    path: &Path,
    open_overrides: &HashMap<PathBuf, String>,
) -> Option<(String, FileEncoding)> {
    if let Some(content) = open_overrides.get(path) {
        let encoding = read_file(path)
            .map(|(_, encoding)| encoding)
            .unwrap_or(FileEncoding::Utf8);
        return Some((content.clone(), encoding));
    }
    read_workspace_text(path)
}

/// 是否应跳过该目录名 / Whether this directory name should be skipped
fn should_skip_name(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules"
}

/// 在目录中搜索，默认可忽略大小写
/// Search in directory; case-insensitive by default
#[cfg(test)]
pub fn search_in_directory(
    dir: &Path,
    query: &str,
    open_overrides: &HashMap<PathBuf, String>,
) -> Vec<WorkspaceSearchHit> {
    search_in_directory_with_options(dir, query, open_overrides, true)
}

/// 在目录中搜索，可指定是否忽略大小写
/// Search in directory with an explicit case-sensitivity option
pub fn search_in_directory_with_options(
    dir: &Path,
    query: &str,
    open_overrides: &HashMap<PathBuf, String>,
    case_insensitive: bool,
) -> Vec<WorkspaceSearchHit> {
    let mut results = Vec::new();
    let mut visited: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    let mut files_scanned = 0usize;

    walk_search(
        dir,
        0,
        &mut WalkSearchCtx {
            query,
            case_insensitive,
            open_overrides,
            results: &mut results,
            visited: &mut visited,
            files_scanned: &mut files_scanned,
        },
    );

    for (path, content) in open_overrides {
        if results.len() >= WORKSPACE_SEARCH_MAX_RESULTS {
            break;
        }
        if visited.contains(path) {
            continue;
        }
        results.extend(search_in_content(path, content, query, case_insensitive));
    }

    results.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    results.truncate(WORKSPACE_SEARCH_MAX_RESULTS);
    results
}

/// 工作区目录扫描上下文 / Workspace directory walk context
struct WalkSearchCtx<'a> {
    query: &'a str,
    case_insensitive: bool,
    open_overrides: &'a HashMap<PathBuf, String>,
    results: &'a mut Vec<WorkspaceSearchHit>,
    visited: &'a mut std::collections::HashSet<PathBuf>,
    files_scanned: &'a mut usize,
}

/// 递归扫描目录并收集搜索命中 / Recursively scan directories and collect search hits
fn walk_search(dir: &Path, depth: usize, ctx: &mut WalkSearchCtx<'_>) {
    if depth > WORKSPACE_SEARCH_MAX_DEPTH
        || ctx.results.len() >= WORKSPACE_SEARCH_MAX_RESULTS
        || *ctx.files_scanned >= WORKSPACE_SEARCH_MAX_FILES
    {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if ctx.results.len() >= WORKSPACE_SEARCH_MAX_RESULTS
            || *ctx.files_scanned >= WORKSPACE_SEARCH_MAX_FILES
        {
            return;
        }
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if should_skip_name(name) {
            continue;
        }
        if path.is_dir() {
            walk_search(&path, depth + 1, ctx);
        } else if path.is_file() && is_markdown_or_text_path(&path) {
            *ctx.files_scanned += 1;
            ctx.visited.insert(path.clone());
            if let Some((content, _)) = resolve_file_body(&path, ctx.open_overrides) {
                ctx.results.extend(search_in_content(
                    &path,
                    &content,
                    ctx.query,
                    ctx.case_insensitive,
                ));
            }
        }
    }
}

/// 在内容中搜索 / Search in content
///
/// `query` 为原始查询；仅在 `case_insensitive` 为 true 时转为小写再匹配。
/// `query` is the original needle; it is lowercased only when `case_insensitive` is true.
pub fn search_in_content(
    path: &Path,
    content: &str,
    query: &str,
    case_insensitive: bool,
) -> Vec<WorkspaceSearchHit> {
    let mut results = Vec::new();
    if query.is_empty() {
        return results;
    }
    let path_buf = path.to_path_buf();
    let needle = if case_insensitive {
        query.to_lowercase()
    } else {
        query.to_string()
    };

    for (line_idx, line) in content.lines().enumerate() {
        let search_in = if case_insensitive {
            line.to_lowercase()
        } else {
            line.to_string()
        };

        let mut start = 0;
        while let Some(pos) = search_in[start..].find(needle.as_str()) {
            let abs_pos = start + pos;
            results.push(WorkspaceSearchHit {
                path: path_buf.clone(),
                line: line_idx,
                content: line.to_string(),
                start: abs_pos,
                end: abs_pos + needle.len(),
            });
            start = abs_pos + needle.len();
            if start >= search_in.len() {
                break;
            }
        }
    }

    results.truncate(50);
    results
}

/// 收集工作区可替换文件列表 / Collect workspace files eligible for replace
pub fn collect_workspace_files(
    root: &Path,
    overrides: &HashMap<PathBuf, String>,
) -> Vec<(PathBuf, String)> {
    collect_workspace_files_with_encoding(root, overrides)
        .into_iter()
        .map(|file| (file.path, file.content))
        .collect()
}

/// 收集工作区文件及写回编码 / Collect workspace files together with write-back encodings
pub fn collect_workspace_files_with_encoding(
    root: &Path,
    overrides: &HashMap<PathBuf, String>,
) -> Vec<WorkspaceFile> {
    let mut files: Vec<WorkspaceFile> = Vec::new();
    let mut files_scanned = 0usize;
    walk_collect(root, 0, overrides, &mut files, &mut files_scanned);
    for (path, content) in overrides {
        if !files.iter().any(|file| &file.path == path) {
            let encoding = read_file(path)
                .map(|(_, encoding)| encoding)
                .unwrap_or(FileEncoding::Utf8);
            files.push(WorkspaceFile {
                path: path.clone(),
                content: content.clone(),
                encoding,
            });
        }
    }
    files
}

/// 递归收集可替换文件 / Recursively collect replaceable files
fn walk_collect(
    dir: &Path,
    depth: usize,
    overrides: &HashMap<PathBuf, String>,
    out: &mut Vec<WorkspaceFile>,
    files_scanned: &mut usize,
) {
    if depth > WORKSPACE_SEARCH_MAX_DEPTH || *files_scanned >= WORKSPACE_SEARCH_MAX_FILES {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if *files_scanned >= WORKSPACE_SEARCH_MAX_FILES {
            return;
        }
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if should_skip_name(name) {
            continue;
        }
        if path.is_dir() {
            walk_collect(&path, depth + 1, overrides, out, files_scanned);
        } else if is_markdown_or_text_path(&path) {
            *files_scanned += 1;
            if let Some((content, encoding)) = resolve_file_body(&path, overrides) {
                out.push(WorkspaceFile {
                    path,
                    content,
                    encoding,
                });
            }
        }
    }
}

/// 对文件列表执行字面全部替换
/// Apply literal replace-all across a file list
#[cfg(test)]
pub fn apply_workspace_replace(
    files: &[(PathBuf, String)],
    query: &str,
    replacement: &str,
) -> (Vec<(PathBuf, String)>, WorkspaceReplaceReport) {
    apply_workspace_replace_with_options(files, query, replacement, true)
}

/// 对文件列表执行字面全部替换，可指定是否忽略大小写
/// Apply literal replace-all with an explicit case-sensitivity option
pub fn apply_workspace_replace_with_options(
    files: &[(PathBuf, String)],
    query: &str,
    replacement: &str,
    case_insensitive: bool,
) -> (Vec<(PathBuf, String)>, WorkspaceReplaceReport) {
    let mut report = WorkspaceReplaceReport::default();
    let mut changed = Vec::new();
    for (path, content) in files {
        let n = count_matches(content, query, case_insensitive, false);
        if n == 0 {
            continue;
        }
        let next = replace_all_in_text(content, query, replacement, case_insensitive, false);
        if next != *content {
            report.files_touched += 1;
            report.matches_total += n;
            changed.push((path.clone(), next));
        }
    }
    (changed, report)
}

/// 按编码写回工作区替换结果 / Write workspace replace results back using each file's encoding
pub fn write_workspace_replacements(
    files: &[WorkspaceFile],
    changed: &[(PathBuf, String)],
) -> Vec<String> {
    let mut errors = Vec::new();
    for (path, new_content) in changed {
        let encoding = files
            .iter()
            .find(|file| &file.path == path)
            .map(|file| file.encoding)
            .unwrap_or(FileEncoding::Utf8);
        if let Err(error) = write_file(path, new_content, encoding) {
            errors.push(format!("{}: {}", path.display(), error));
        }
    }
    errors
}

/// 预览工作区替换影响范围，可指定是否忽略大小写
/// Preview replace impact with an explicit case-sensitivity option
pub fn preview_workspace_replace_counts_with_options(
    files: &[(PathBuf, String)],
    query: &str,
    case_insensitive: bool,
) -> (usize, usize) {
    let mut files_touched = 0usize;
    let mut matches_total = 0usize;
    if query.is_empty() {
        return (0, 0);
    }
    for (_path, content) in files {
        let n = count_matches(content, query, case_insensitive, false);
        if n > 0 {
            files_touched += 1;
            matches_total += n;
        }
    }
    (files_touched, matches_total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::file_encoding::encode_text;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_search_prefers_open_override() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a.md");
        fs::write(&path, "disk-only-token").unwrap();

        let mut overrides = HashMap::new();
        overrides.insert(path.clone(), "buffer-token hello".to_string());

        let found = search_in_directory(dir.path(), "buffer-token", &overrides);
        assert_eq!(found.len(), 1);
        assert!(found[0].content.contains("buffer-token"));

        let disk_only = search_in_directory(dir.path(), "disk-only-token", &overrides);
        assert!(disk_only.is_empty());
    }

    #[test]
    fn test_search_falls_back_to_disk_when_tab_evicted() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a.md");
        fs::write(&path, "disk-token remains").unwrap();

        let overrides = HashMap::new();
        let found = search_in_directory(dir.path(), "disk-token", &overrides);
        assert_eq!(found.len(), 1);
        assert!(found[0].content.contains("disk-token"));
    }

    #[test]
    fn test_search_is_case_sensitive_when_requested() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a.md");
        fs::write(&path, "Hello World").unwrap();

        let overrides = HashMap::new();
        let insensitive = search_in_directory_with_options(dir.path(), "hello", &overrides, true);
        assert_eq!(insensitive.len(), 1);

        let sensitive = search_in_directory_with_options(dir.path(), "hello", &overrides, false);
        assert!(sensitive.is_empty());

        let exact = search_in_directory_with_options(dir.path(), "Hello", &overrides, false);
        assert_eq!(exact.len(), 1);
    }

    #[test]
    fn test_search_reads_gbk_files() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("gbk.md");
        let bytes = encode_text("你好世界 GBKtoken", FileEncoding::Gbk).unwrap();
        fs::write(&path, bytes).unwrap();

        let found = search_in_directory(dir.path(), "GBKtoken", &HashMap::new());
        assert_eq!(found.len(), 1);
        assert!(found[0].content.contains("你好世界"));
    }

    #[test]
    fn test_search_in_content_respects_case_flag() {
        let path = PathBuf::from("note.md");
        let hits = search_in_content(&path, "Hello HELLO", "hello", true);
        assert_eq!(hits.len(), 2);

        let sensitive = search_in_content(&path, "Hello HELLO", "hello", false);
        assert!(sensitive.is_empty());
    }

    #[test]
    fn test_apply_workspace_replace() {
        let files = vec![
            (PathBuf::from("a.md"), "foo bar foo".to_string()),
            (PathBuf::from("b.md"), "nothing".to_string()),
        ];
        let (changed, report) = apply_workspace_replace(&files, "foo", "baz");
        assert_eq!(report.files_touched, 1);
        assert_eq!(report.matches_total, 2);
        assert_eq!(changed[0].1, "baz bar baz");
    }
}
