//! 会话持久化：标签页、工作区与未保存草稿
//! Session persistence: tabs, workspace, and unsaved drafts

use crate::config::LARGE_FILE_THRESHOLD_BYTES;
use crate::services::settings::SettingsService;
use crate::state::{AppState, SidebarTab, TabInfo};
use dioxus::prelude::{ReadableExt, WritableExt};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 会话文件版本 / Session file schema version
pub const SESSION_VERSION: u32 = 1;
/// 会话元数据文件名 / Session metadata filename
const SESSION_FILENAME: &str = "session.json";
/// 草稿子目录名 / Draft subdirectory name
const DRAFTS_DIRNAME: &str = "session_drafts";

/// 磁盘上的会话快照 / On-disk session snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSnapshot {
    /// Schema 版本 / Schema version
    pub version: u32,
    /// 工作区根目录 / Workspace root
    pub workspace_root: Option<PathBuf>,
    /// 活动标签下标 / Active tab index
    pub active_tab_index: usize,
    /// 侧栏页签：outline | files / Sidebar tab key
    pub sidebar_tab: String,
    /// 标签条目 / Tab entries
    pub tabs: Vec<SessionTabEntry>,
}

/// 单个标签的持久化描述 / Persisted description of one tab
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionTabEntry {
    /// 磁盘路径（已保存文件）/ Disk path for saved files
    pub path: Option<PathBuf>,
    /// 标签标题 / Tab title
    pub title: String,
    /// 是否有未保存修改 / Whether content has unsaved edits
    pub modified: bool,
    /// 草稿文件 ID（有则从 sidecar 读正文）/ Draft id when body lives in a sidecar
    pub draft_id: Option<String>,
}

/// 恢复结果统计 / Restore outcome statistics
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RestoreReport {
    /// 成功恢复的标签数 / Successfully restored tabs
    pub restored: usize,
    /// 因缺失而跳过的路径数 / Skipped missing paths
    pub missing: usize,
    /// 因过大而跳过的条目数 / Skipped oversized entries
    pub skipped_large: usize,
}

impl Default for SessionSnapshot {
    fn default() -> Self {
        Self {
            version: SESSION_VERSION,
            workspace_root: None,
            active_tab_index: 0,
            sidebar_tab: "outline".to_string(),
            tabs: Vec::new(),
        }
    }
}

/// 会话服务 / Session service
pub struct SessionService;

impl SessionService {
    /// 会话元数据路径 / Path to session.json
    ///
    /// 获取配置目录下的 session.json 路径
    /// Resolve `{config}/session.json`
    pub fn session_path() -> io::Result<PathBuf> {
        Ok(SettingsService::get_config_dir()?.join(SESSION_FILENAME))
    }

    /// 草稿目录路径 / Path to draft directory
    ///
    /// 获取草稿 sidecar 目录
    /// Resolve `{config}/session_drafts/`
    pub fn drafts_dir() -> io::Result<PathBuf> {
        Ok(SettingsService::get_config_dir()?.join(DRAFTS_DIRNAME))
    }

    /// 将快照与草稿写入磁盘（可注入根目录便于测试）
    /// Persist snapshot + drafts to disk (injectable root for tests)
    pub fn save_to(root: &Path, state: &AppState) -> io::Result<SessionSnapshot> {
        let mut state = *state;
        state.save_current_tab_content();

        let drafts = root.join(DRAFTS_DIRNAME);
        fs::create_dir_all(&drafts)?;
        // 清理旧草稿，避免堆积 / Clear old drafts to avoid buildup
        clear_dir_contents(&drafts)?;

        let doc = state.document();
        let ui = state.ui();
        let tabs = doc.tabs.read().clone();
        let active = *doc.current_tab_index.read();
        let workspace = ui.workspace_root.read().clone();
        let sidebar = match *ui.sidebar_tab.read() {
            SidebarTab::Files => "files",
            SidebarTab::Outline => "outline",
        }
        .to_string();

        let mut entries = Vec::new();
        let mut compact_active = 0usize;
        let mut seen = 0usize;

        for (i, tab) in tabs.iter().enumerate() {
            let content = if i == active {
                doc.content.read().clone()
            } else {
                tab.content
                    .as_ref()
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            };

            let blank_untitled = tab.path.is_none() && !tab.modified && content.trim().is_empty();
            if blank_untitled {
                continue;
            }

            if i == active {
                compact_active = seen;
            }

            let needs_draft = tab.modified || (tab.path.is_none() && !content.trim().is_empty());
            let mut draft_id = None;

            if needs_draft {
                if content.len() > LARGE_FILE_THRESHOLD_BYTES {
                    tracing::warn!(
                        "Session draft skipped (too large): {} bytes, title={}",
                        content.len(),
                        tab.title
                    );
                    // 已保存路径仍可按路径恢复（丢失未保存修改）
                    // Path-only restore still possible (unsaved edits lost)
                    if tab.path.is_none() {
                        continue;
                    }
                } else {
                    let id = format!("tab_{}_{}", seen, unix_millis());
                    let draft_path = drafts.join(format!("{id}.md"));
                    fs::write(&draft_path, &content)?;
                    draft_id = Some(id);
                }
            }

            entries.push(SessionTabEntry {
                path: tab.path.clone(),
                title: tab.title.clone(),
                modified: draft_id.is_some(),
                draft_id,
            });
            seen += 1;
        }

        let snapshot = SessionSnapshot {
            version: SESSION_VERSION,
            workspace_root: workspace,
            active_tab_index: if entries.is_empty() {
                0
            } else {
                compact_active.min(entries.len() - 1)
            },
            sidebar_tab: sidebar,
            tabs: entries,
        };

        let path = root.join(SESSION_FILENAME);
        let tmp = root.join(format!("{SESSION_FILENAME}.tmp"));
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&tmp, json)?;
        fs::rename(&tmp, &path)?;
        Ok(snapshot)
    }

    /// 保存当前会话到默认配置目录 / Save current session to default config dir
    pub fn save(state: &AppState) -> io::Result<SessionSnapshot> {
        let root = SettingsService::get_config_dir()?;
        fs::create_dir_all(&root)?;
        Self::save_to(&root, state)
    }

    /// 从默认路径加载会话 / Load session from default path
    pub fn load() -> Option<SessionSnapshot> {
        let path = Self::session_path().ok()?;
        Self::load_from(&path)
    }

    /// 从指定路径加载会话 / Load session from a path
    pub fn load_from(path: &Path) -> Option<SessionSnapshot> {
        if !path.exists() {
            return None;
        }
        match fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str::<SessionSnapshot>(&text) {
                Ok(snap) => Some(snap),
                Err(e) => {
                    tracing::warn!("Failed to parse session.json: {}", e);
                    None
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read session.json: {}", e);
                None
            }
        }
    }

    /// 将快照恢复到应用状态（可注入草稿根目录）
    /// Apply snapshot into app state (injectable drafts root)
    pub fn restore_into(
        state: &mut AppState,
        snapshot: &SessionSnapshot,
        drafts_root: &Path,
    ) -> RestoreReport {
        let mut report = RestoreReport::default();
        if snapshot.tabs.is_empty() {
            return report;
        }

        let mut built: Vec<TabInfo> = Vec::new();

        for entry in &snapshot.tabs {
            if let Some(ref draft_id) = entry.draft_id {
                let draft_path = drafts_root.join(format!("{draft_id}.md"));
                match fs::read_to_string(&draft_path) {
                    Ok(content) => {
                        if content.len() > LARGE_FILE_THRESHOLD_BYTES {
                            report.skipped_large += 1;
                            continue;
                        }
                        let mut tab = if let Some(ref path) = entry.path {
                            let encoding =
                                crate::actions::FileActions::read_file_with_encoding(path)
                                    .map(|(_, encoding)| encoding)
                                    .unwrap_or_default();
                            TabInfo::from_file_with_encoding(path.clone(), &content, encoding)
                        } else {
                            TabInfo::new(&entry.title)
                        };
                        tab.title = entry.title.clone();
                        tab.modified = true;
                        tab.set_content_arc(std::sync::Arc::<str>::from(content.as_str()));
                        built.push(tab);
                        report.restored += 1;
                    }
                    Err(_) => {
                        // 草稿丢失：若有路径则退回路径恢复
                        // Draft missing: fall back to path if present
                        if let Some(ref path) = entry.path {
                            if let Some(tab) = try_load_path_tab(path, &mut report) {
                                built.push(tab);
                                report.restored += 1;
                            }
                        } else {
                            report.missing += 1;
                        }
                    }
                }
                continue;
            }

            if let Some(ref path) = entry.path {
                if let Some(tab) = try_load_path_tab(path, &mut report) {
                    built.push(tab);
                    report.restored += 1;
                }
            }
        }

        if built.is_empty() {
            return report;
        }

        let active = snapshot.active_tab_index.min(built.len().saturating_sub(1));

        // 工作区 / Workspace
        if let Some(ref root) = snapshot.workspace_root {
            if root.is_dir() {
                crate::actions::FileActions::set_workspace(state, root.clone());
            }
        }

        // 侧栏 / Sidebar
        {
            let mut ui = state.ui();
            *ui.sidebar_tab.write() = match snapshot.sidebar_tab.as_str() {
                "files" => SidebarTab::Files,
                _ => SidebarTab::Outline,
            };
        }

        state.apply_restored_tabs(built, active);
        report
    }

    /// 从默认配置目录恢复会话 / Restore session from default config dir
    pub fn restore(state: &mut AppState, snapshot: &SessionSnapshot) -> RestoreReport {
        let drafts = Self::drafts_dir().unwrap_or_else(|_| PathBuf::from(DRAFTS_DIRNAME));
        Self::restore_into(state, snapshot, &drafts)
    }
}

/// 尝试从磁盘路径加载标签（过大则跳过）
/// Try loading a tab from disk (skip if oversized)
fn try_load_path_tab(path: &Path, report: &mut RestoreReport) -> Option<TabInfo> {
    if !path.exists() {
        report.missing += 1;
        return None;
    }
    let meta = fs::metadata(path).ok()?;
    if meta.len() as usize > LARGE_FILE_THRESHOLD_BYTES {
        report.skipped_large += 1;
        return None;
    }
    match crate::actions::FileActions::read_file_with_encoding(path) {
        Ok((content, encoding)) => Some(TabInfo::from_file_with_encoding(
            path.to_path_buf(),
            &content,
            encoding,
        )),
        Err(e) => {
            tracing::warn!("Session restore failed to read {:?}: {}", path, e);
            report.missing += 1;
            None
        }
    }
}

/// 清空目录内容（保留目录本身）/ Clear directory contents (keep the directory)
fn clear_dir_contents(dir: &Path) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// 当前 Unix 毫秒时间戳 / Current Unix time in milliseconds
fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use dioxus::prelude::ReadableExt;
    use tempfile::TempDir;

    /// 在 Dioxus 作用域内运行 Signal 测试 / Run Signal tests inside a Dioxus scope
    fn with_runtime<F: FnOnce()>(f: F) {
        use dioxus::prelude::*;
        fn empty_component() -> Element {
            rsx! { div {} }
        }
        let vdom = VirtualDom::prebuilt(empty_component);
        vdom.in_scope(ScopeId::ROOT, f);
    }

    #[test]
    fn test_session_snapshot_roundtrip_json() {
        let snap = SessionSnapshot {
            version: 1,
            workspace_root: Some(PathBuf::from("/tmp/ws")),
            active_tab_index: 1,
            sidebar_tab: "files".to_string(),
            tabs: vec![SessionTabEntry {
                path: Some(PathBuf::from("/tmp/ws/a.md")),
                title: "a".to_string(),
                modified: false,
                draft_id: None,
            }],
        };
        let json = serde_json::to_string(&snap).unwrap();
        let back: SessionSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, back);
    }

    #[test]
    fn test_save_and_restore_path_tabs() {
        with_runtime(|| {
            let dir = TempDir::new().unwrap();
            let root = dir.path();
            let file_a = root.join("a.md");
            let file_b = root.join("b.md");
            fs::write(&file_a, "# A\n").unwrap();
            fs::write(&file_b, "# B\nhello").unwrap();

            let mut state = AppState::new();
            state.open_file_in_tab(file_a.clone(), "# A\n".to_string());
            state.open_file_in_tab(file_b.clone(), "# B\nhello".to_string());

            let snap = SessionService::save_to(root, &state).unwrap();
            assert_eq!(snap.tabs.len(), 2);
            assert!(snap.tabs.iter().all(|t| t.draft_id.is_none()));

            let mut state2 = AppState::new();
            let report =
                SessionService::restore_into(&mut state2, &snap, &root.join(DRAFTS_DIRNAME));
            assert_eq!(report.restored, 2);
            assert_eq!(state2.document().tabs.read().len(), 2);
            assert!(state2
                .document()
                .tabs
                .read()
                .iter()
                .any(|t| t.path.as_ref() == Some(&file_b)));
        });
    }

    #[test]
    fn test_save_and_restore_untitled_draft() {
        with_runtime(|| {
            let dir = TempDir::new().unwrap();
            let root = dir.path();

            let mut state = AppState::new();
            state.update_content("# draft note\n".to_string());
            assert!(*state.document().modified.read());

            let snap = SessionService::save_to(root, &state).unwrap();
            assert_eq!(snap.tabs.len(), 1);
            assert!(snap.tabs[0].draft_id.is_some());
            assert!(snap.tabs[0].path.is_none());

            let mut state2 = AppState::new();
            let report =
                SessionService::restore_into(&mut state2, &snap, &root.join(DRAFTS_DIRNAME));
            assert_eq!(report.restored, 1);
            assert!(state2.document().content.read().contains("draft note"));
            assert!(*state2.document().modified.read());
        });
    }

    #[test]
    fn test_restore_skips_missing_path() {
        with_runtime(|| {
            let dir = TempDir::new().unwrap();
            let drafts = dir.path().join(DRAFTS_DIRNAME);
            fs::create_dir_all(&drafts).unwrap();

            let snap = SessionSnapshot {
                version: 1,
                workspace_root: None,
                active_tab_index: 0,
                sidebar_tab: "outline".to_string(),
                tabs: vec![SessionTabEntry {
                    path: Some(dir.path().join("gone.md")),
                    title: "gone".to_string(),
                    modified: false,
                    draft_id: None,
                }],
            };

            let mut state = AppState::new();
            let report = SessionService::restore_into(&mut state, &snap, &drafts);
            assert_eq!(report.missing, 1);
            assert_eq!(report.restored, 0);
            // 默认空白标签仍在 / Default blank tab remains
            assert_eq!(state.document().tabs.read().len(), 1);
        });
    }
}
