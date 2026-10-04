//! 文件监控服务 / File Watcher Service
//!
//! 监视父目录事件，并用 mtime 确认，避免漏检或把自身保存当成外部修改。
//! Watch the parent directory and confirm with mtime so we neither miss edits nor treat our own saves as external.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

/// 监视命令 / Watch command
enum WatchCmd {
    /// 监视这些文件各自的父目录 / Watch the parent directory of each file
    Set(Vec<PathBuf>),
    Clear,
}

/// 把目录事件送到异步循环 / Bridge directory events into the async loop
pub struct FileEventHub {
    cmd_tx: std_mpsc::Sender<WatchCmd>,
    event_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
}

impl FileEventHub {
    /// 启动监视线程；失败时仍可靠 mtime 轮询
    /// Start the watch thread; mtime polling remains if this fails
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = std_mpsc::channel();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        if let Err(e) = thread::Builder::new()
            .name("mm-file-watch".into())
            .spawn(move || watch_thread(cmd_rx, event_tx))
        {
            tracing::warn!("无法启动文件监视线程 / Failed to start file-watch thread: {e}");
        }
        Self { cmd_tx, event_rx }
    }

    /// 监视这些文件的父目录；空列表表示停止
    /// Watch the parent directory of each file; an empty list stops watching
    pub fn watch_files(&self, paths: &[PathBuf]) {
        let cmd = if paths.is_empty() {
            WatchCmd::Clear
        } else {
            WatchCmd::Set(paths.to_vec())
        };
        let _ = self.cmd_tx.send(cmd);
    }

    /// 等到下一次文件系统事件或通道关闭
    /// Wait for the next filesystem event or channel close
    pub async fn recv(&mut self) -> Option<()> {
        self.event_rx.recv().await
    }
}

/// 事件路径是否落在被监视文件的父目录内（含原子替换的临时名）
/// Whether an event path is inside the watched file's parent (including atomic-save temps)
#[cfg(test)]
pub fn event_in_watched_dir(event_path: &Path, watched_file: &Path) -> bool {
    if event_path == watched_file {
        return true;
    }
    let Some(parent) = watched_file.parent() else {
        return false;
    };
    if parent.as_os_str().is_empty() {
        return false;
    }
    event_path.starts_with(parent)
}

/// 监视线程：按命令切换目录，事件只发唤醒信号
/// Watch thread: switch directories on command and only send wake-ups
fn watch_thread(
    cmd_rx: std_mpsc::Receiver<WatchCmd>,
    event_tx: tokio::sync::mpsc::UnboundedSender<()>,
) {
    let (fs_tx, fs_rx) = std_mpsc::channel();
    let mut watcher = match RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if res.is_ok() {
                let _ = fs_tx.send(());
            }
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!(
                "文件系统监视不可用，回退 mtime 轮询 / FS watch unavailable, falling back to mtime: {e}"
            );
            while cmd_rx.recv().is_ok() {}
            return;
        }
    };

    let mut parents: HashSet<PathBuf> = HashSet::new();
    loop {
        let mut cmd = match cmd_rx.try_recv() {
            Ok(c) => Some(c),
            Err(std_mpsc::TryRecvError::Disconnected) => return,
            Err(std_mpsc::TryRecvError::Empty) => None,
        };
        if cmd.is_none() {
            match fs_rx.recv_timeout(Duration::from_millis(200)) {
                Ok(()) => {
                    let _ = event_tx.send(());
                    while fs_rx.try_recv().is_ok() {}
                }
                Err(std_mpsc::RecvTimeoutError::Timeout) => {}
                Err(std_mpsc::RecvTimeoutError::Disconnected) => return,
            }
            cmd = match cmd_rx.try_recv() {
                Ok(c) => Some(c),
                Err(std_mpsc::TryRecvError::Disconnected) => return,
                Err(std_mpsc::TryRecvError::Empty) => None,
            };
        }
        if let Some(c) = cmd {
            apply_watch_cmd(&mut watcher, &mut parents, c);
        }
    }
}

/// 让监视目录集合与当前打开文件的父目录对齐
/// Align the watched directory set with the parents of the open files
fn apply_watch_cmd(
    watcher: &mut RecommendedWatcher,
    parents: &mut HashSet<PathBuf>,
    cmd: WatchCmd,
) {
    let next: HashSet<PathBuf> = match cmd {
        WatchCmd::Clear => HashSet::new(),
        WatchCmd::Set(files) => files
            .into_iter()
            .filter_map(|file| file.parent().map(|parent| parent.to_path_buf()))
            .filter(|parent| parent.is_dir())
            .collect(),
    };
    let stale: Vec<PathBuf> = parents.difference(&next).cloned().collect();
    let fresh: Vec<PathBuf> = next.difference(parents).cloned().collect();
    for old in stale {
        let _ = watcher.unwatch(&old);
        parents.remove(&old);
    }
    for parent in fresh {
        match watcher.watch(&parent, RecursiveMode::NonRecursive) {
            Ok(()) => {
                parents.insert(parent);
            }
            Err(error) => {
                tracing::warn!("监视目录失败 / Failed to watch directory {parent:?}: {error}")
            }
        }
    }
}

/// 文件修改检测（按路径比较 mtime）
/// File modification detection (mtime comparison per path)
pub struct FileModificationChecker {
    baseline: HashMap<PathBuf, SystemTime>,
}

impl FileModificationChecker {
    /// 创建新的检测器 / Create a new checker
    pub fn new() -> Self {
        Self {
            baseline: HashMap::new(),
        }
    }

    /// 设置要监控的单个文件 / Set the single file to watch
    #[cfg(test)]
    pub fn set_file(&mut self, path: &PathBuf) {
        self.sync_files(std::slice::from_ref(path));
    }

    /// 同步监控集合：新文件记下当前 mtime，消失的文件移除，已有文件保留基线
    /// Sync the watched set: stamp new files, drop missing ones, and keep existing baselines
    pub fn sync_files(&mut self, paths: &[PathBuf]) {
        let wanted: HashSet<&PathBuf> = paths.iter().collect();
        self.baseline.retain(|path, _| wanted.contains(path));
        for path in paths {
            self.baseline
                .entry(path.clone())
                .or_insert_with(|| Self::get_modified_time(path).unwrap_or(SystemTime::UNIX_EPOCH));
        }
    }

    /// 清除监控 / Clear the watch
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.baseline.clear();
    }

    /// 是否有任一文件的 mtime 新于基线
    /// Whether any watched file's mtime is newer than its baseline
    #[cfg(test)]
    pub fn check_modified(&mut self) -> bool {
        !self.changed_paths().is_empty()
    }

    /// 返回 mtime 新于基线的路径，不推进基线
    /// Return paths whose mtime is newer than the baseline, without moving the baseline
    pub fn changed_paths(&self) -> Vec<PathBuf> {
        self.baseline
            .iter()
            .filter_map(|(path, last)| {
                let current = Self::get_modified_time(path)?;
                (current > *last).then(|| path.clone())
            })
            .collect()
    }

    /// 把指定路径的基线推到当前 mtime，避免同一轮修改被重复报告
    /// Advance the baseline of these paths to the current mtime so the same edit is not reported again
    pub fn acknowledge(&mut self, paths: &[PathBuf]) {
        for path in paths {
            if let Some(slot) = self.baseline.get_mut(path) {
                if let Some(current) = Self::get_modified_time(path) {
                    *slot = current;
                }
            }
        }
    }

    /// 更新全部基线（在自身保存后调用）
    /// Refresh every baseline (call after our own save)
    pub fn update(&mut self) {
        let paths: Vec<PathBuf> = self.baseline.keys().cloned().collect();
        self.acknowledge(&paths);
    }

    /// 获取文件修改时间 / Get file modification time
    fn get_modified_time(path: &Path) -> Option<SystemTime> {
        std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
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

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        let path = temp_file.path().to_path_buf();

        checker.set_file(&path);
        assert!(!checker.check_modified());

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
        assert!(checker.baseline.contains_key(&path));
        checker.clear();
        assert!(checker.baseline.is_empty());
        assert!(!checker.check_modified());
    }

    #[test]
    fn test_tracks_two_files_independently() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("a.md");
        let second = dir.path().join("b.md");
        std::fs::write(&first, "a").unwrap();
        std::fs::write(&second, "b").unwrap();

        let mut checker = FileModificationChecker::new();
        checker.sync_files(&[first.clone(), second.clone()]);
        assert!(checker.changed_paths().is_empty());

        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&second, "b2").unwrap();
        let changed = checker.changed_paths();
        assert_eq!(changed, vec![second.clone()]);
        checker.acknowledge(&changed);
        assert!(checker.changed_paths().is_empty());
        assert!(!checker.check_modified());
        let _ = first;
    }

    #[test]
    fn test_event_in_watched_dir_matches_file_and_sibling_temp() {
        let dir = tempfile::tempdir().unwrap();
        let watched = dir.path().join("note.md");
        let sibling = dir.path().join("note.md.tmp");
        let other = dir.path().parent().unwrap().join("other-note.md");
        assert!(event_in_watched_dir(&watched, &watched));
        assert!(event_in_watched_dir(&sibling, &watched));
        assert!(!event_in_watched_dir(&other, &watched));
    }
}
