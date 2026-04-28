//! 文件监控服务 / File Watcher Service
//!
//! 监控文件外部修改，当文件被其他程序修改时通知用户
//! Monitors external file changes, notifies user when file is modified by other programs

use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

/// 文件修改检测（优先使用文件系统事件，回退到时间戳）
/// File modification detection (filesystem events first, timestamp fallback)
pub struct FileModificationChecker {
    path: Option<PathBuf>,
    last_modified: Option<std::time::SystemTime>,
    last_event_check: Option<std::time::SystemTime>,
    watcher: Option<RecommendedWatcher>,
    receiver: Option<Receiver<notify::Result<Event>>>,
    pending_external_change: bool,
}

impl FileModificationChecker {
    /// 创建新的检测器 / Create new checker
    pub fn new() -> Self {
        Self {
            path: None,
            last_modified: None,
            last_event_check: None,
            watcher: None,
            receiver: None,
            pending_external_change: false,
        }
    }

    /// 设置要监控的文件 / Set file to watch
    pub fn set_file(&mut self, path: &PathBuf) {
        self.clear();
        self.path = Some(path.clone());
        // 初始化时记录，避免因首次检查产生误报
        // Record at init time to avoid false positive on first check
        self.last_modified = Self::get_modified_time(path);
        self.last_event_check = self.last_modified;
        self.pending_external_change = false;

        let watch_target = path
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| path.clone());

        let (tx, rx) = std::sync::mpsc::channel();
        match recommended_watcher(tx) {
            Ok(mut watcher) => {
                if let Err(err) = watcher.watch(&watch_target, RecursiveMode::NonRecursive) {
                    tracing::warn!(
                        "文件监控事件注册失败，回退到时间戳轮询: {:?} - {}",
                        watch_target,
                        err
                    );
                } else {
                    self.receiver = Some(rx);
                    self.watcher = Some(watcher);
                }
            }
            Err(err) => {
                tracing::warn!("无法创建文件系统监控器，回退到时间戳轮询: {}", err);
            }
        }
    }

    /// 清除监控 / Clear watch
    pub fn clear(&mut self) {
        self.path = None;
        self.last_modified = None;
        self.last_event_check = None;
        self.watcher = None;
        self.receiver = None;
        self.pending_external_change = false;
    }

    /// 检查文件是否被外部修改（忽略自身保存导致的修改）
    /// Check if file was externally modified (ignores changes caused by own save)
    pub fn check_modified(&mut self) -> bool {
        self.drain_events();

        if let Some(ref path) = self.path {
            if let Some(current) = Self::get_modified_time(path) {
                if self.pending_external_change {
                    let baseline = self.last_event_check.or(self.last_modified);
                    if baseline.is_none_or(|last| current > last) {
                        return true;
                    }
                    self.pending_external_change = false;
                    self.last_event_check = Some(current);
                }

                if let Some(last) = self.last_modified {
                    // 仅当修改时间大于上次记录时视为外部修改
                    // Only treat as external when modification time is strictly newer
                    if current > last {
                        self.last_event_check = Some(current);
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 更新最后修改时间（在自身保存后调用）
    /// Update last modified time (call after own save to avoid false positives)
    pub fn update(&mut self) {
        self.pending_external_change = false;
        self.drain_events();
        if let Some(ref path) = self.path {
            self.last_modified = Self::get_modified_time(path);
            self.last_event_check = self.last_modified;
        }
    }

    /// 获取文件修改时间 / Get file modification time
    fn get_modified_time(path: &PathBuf) -> Option<std::time::SystemTime> {
        std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
    }

    fn drain_events(&mut self) {
        let Some(target_path) = self.path.clone() else {
            return;
        };

        let Some(receiver) = self.receiver.as_ref() else {
            return;
        };

        while let Ok(event) = receiver.try_recv() {
            match event {
                Ok(event) => {
                    if Self::event_matches_target(&target_path, &event)
                        && Self::is_relevant_event_kind(&event.kind)
                    {
                        self.pending_external_change = true;
                    }
                }
                Err(err) => {
                    tracing::warn!("文件监控事件读取失败 / File watch event error: {}", err);
                }
            }
        }
    }

    fn event_matches_target(target_path: &std::path::Path, event: &Event) -> bool {
        event.paths.iter().any(|event_path| {
            event_path == target_path
                || (event_path.file_name() == target_path.file_name()
                    && event_path.parent() == target_path.parent())
        })
    }

    fn is_relevant_event_kind(kind: &EventKind) -> bool {
        matches!(
            kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) | EventKind::Any
        )
    }
}

impl Default for FileModificationChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_modification_checker() {
        let mut checker = FileModificationChecker::new();

        // 创建临时文件 / Create temp file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        let path = temp_file.path().to_path_buf();

        checker.set_file(&path);
        assert!(!checker.check_modified());

        // 修改文件 / Modify file
        std::thread::sleep(std::time::Duration::from_millis(10));
        writeln!(temp_file, "more content").unwrap();

        assert!(checker.check_modified());
        checker.update();
        assert!(!checker.check_modified());
    }

    #[test]
    fn test_clear() {
        let mut checker = FileModificationChecker::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        checker.set_file(&path);
        assert!(checker.path.is_some());
        checker.clear();
        assert!(checker.path.is_none());
        assert!(!checker.check_modified());
    }
}
