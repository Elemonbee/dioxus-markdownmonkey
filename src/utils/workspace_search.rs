//! 工作区搜索与替换辅助 / Workspace search & replace helpers

use crate::state::AppState;
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

/// 从应用状态收集打开标签的路径→正文覆盖表
/// Collect path→body overrides from open tabs
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
            doc.content.read().clone()
        } else {
            tab.content
                .as_ref()
                .map(|c| c.to_string())
                .unwrap_or_default()
        };
        map.insert(path.clone(), body);
    }
    map
}

/// 在目录中搜索（磁盘 + 打开标签覆盖）
/// Search in directory (disk + open-tab overrides)
pub fn search_in_directory(
    dir: &Path,
    query: &str,
    open_overrides: &HashMap<PathBuf, String>,
) -> Vec<WorkspaceSearchHit> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();
    let mut visited: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    fn walk(
        dir: &Path,
        query_lower: &str,
        open_overrides: &HashMap<PathBuf, String>,
        results: &mut Vec<WorkspaceSearchHit>,
        visited: &mut std::collections::HashSet<PathBuf>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                if path.is_dir() {
                    walk(&path, query_lower, open_overrides, results, visited);
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "md" || ext == "markdown" || ext == "txt" {
                            visited.insert(path.clone());
                            let content = open_overrides
                                .get(&path)
                                .cloned()
                                .or_else(|| std::fs::read_to_string(&path).ok());
                            if let Some(content) = content {
                                results.extend(search_in_content(
                                    &path,
                                    &content,
                                    query_lower,
                                    true,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    walk(
        dir,
        &query_lower,
        open_overrides,
        &mut results,
        &mut visited,
    );

    for (path, content) in open_overrides {
        if visited.contains(path) {
            continue;
        }
        results.extend(search_in_content(path, content, &query_lower, true));
    }

    results.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    results.truncate(100);
    results
}

/// 在内容中搜索 / Search in content
pub fn search_in_content(
    path: &Path,
    content: &str,
    query_lower: &str,
    case_insensitive: bool,
) -> Vec<WorkspaceSearchHit> {
    let mut results = Vec::new();
    let path_buf = path.to_path_buf();

    for (line_idx, line) in content.lines().enumerate() {
        let search_in = if case_insensitive {
            line.to_lowercase()
        } else {
            line.to_string()
        };

        let mut start = 0;
        while let Some(pos) = search_in[start..].find(query_lower) {
            let abs_pos = start + pos;
            results.push(WorkspaceSearchHit {
                path: path_buf.clone(),
                line: line_idx,
                content: line.to_string(),
                start: abs_pos,
                end: abs_pos + query_lower.len(),
            });
            start = abs_pos + query_lower.len();
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
    let mut files: Vec<(PathBuf, String)> = Vec::new();

    fn walk(
        dir: &Path,
        overrides: &HashMap<PathBuf, String>,
        out: &mut Vec<(PathBuf, String)>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                if path.is_dir() {
                    walk(&path, overrides, out);
                } else if let Some(ext) = path.extension() {
                    if ext == "md" || ext == "markdown" || ext == "txt" {
                        let content = overrides
                            .get(&path)
                            .cloned()
                            .or_else(|| std::fs::read_to_string(&path).ok());
                        if let Some(content) = content {
                            out.push((path, content));
                        }
                    }
                }
            }
        }
    }

    walk(root, overrides, &mut files);
    for (path, content) in overrides {
        if !files.iter().any(|(p, _)| p == path) {
            files.push((path.clone(), content.clone()));
        }
    }
    files
}

/// 对文件列表执行字面大小写不敏感全部替换
/// Apply literal case-insensitive replace-all across a file list
pub fn apply_workspace_replace(
    files: &[(PathBuf, String)],
    query: &str,
    replacement: &str,
) -> (Vec<(PathBuf, String)>, WorkspaceReplaceReport) {
    let mut report = WorkspaceReplaceReport::default();
    let mut changed = Vec::new();
    for (path, content) in files {
        let n = count_matches(content, query, true, false);
        if n == 0 {
            continue;
        }
        let next = replace_all_in_text(content, query, replacement, true, false);
        if next != *content {
            report.files_touched += 1;
            report.matches_total += n;
            changed.push((path.clone(), next));
        }
    }
    (changed, report)
}

/// 预览工作区替换将影响的文件数与匹配数 / Preview files and matches a workspace replace would touch
pub fn preview_workspace_replace_counts(
    files: &[(PathBuf, String)],
    query: &str,
) -> (usize, usize) {
    let mut files_touched = 0usize;
    let mut matches_total = 0usize;
    if query.is_empty() {
        return (0, 0);
    }
    for (_path, content) in files {
        let n = count_matches(content, query, true, false);
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
