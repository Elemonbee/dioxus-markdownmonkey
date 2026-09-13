//! 文件监控服务 / File Watcher Service
//!
//! 监视父目录事件，并用 mtime 确认，避免漏检或把自身保存当成外部修改。
//! Watch the parent directory and confirm with mtime so we neither miss edits nor treat our own saves as external.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use std::thread;
use std::time::Duration;

/// 监视命令 / Watch command
enum WatchCmd {
    Set(PathBuf),
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

    /// 监视该文件的父目录；`None` 表示停止
    /// Watch the file's parent directory; `None` stops watching
    pub fn watch_file(&self, path: Option<&Path>) {
        let cmd = match path {
            Some(p) => WatchCmd::Set(p.to_path_buf()),
            None => WatchCmd::Clear,
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

    let mut parent: Option<PathBuf> = None;
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
            apply_watch_cmd(&mut watcher, &mut parent, c);
        }
    }
}

/// 切换或拆除当前监视目录 / Switch or tear down the watched directory
fn apply_watch_cmd(watcher: &mut RecommendedWatcher, parent: &mut Option<PathBuf>, cmd: WatchCmd) {
    let next = match cmd {
        WatchCmd::Clear => None,
        WatchCmd::Set(file) => file.parent().map(|p| p.to_path_buf()),
    };
    if next.as_ref() == parent.as_ref() {
        return;
    }
    if let Some(old) = parent.take() {
        let _ = watcher.unwatch(&old);
    }
    if let Some(p) = next {
        if p.is_dir() {
            match watcher.watch(&p, RecursiveMode::NonRecursive) {
                Ok(()) => *parent = Some(p),
                Err(e) => tracing::warn!("监视目录失败 / Failed to watch directory {p:?}: {e}"),
            }
        }
    }
}

/// 文件修改检测（比较 mtime）
/// File modification detection (mtime comparison)
pub struct FileModificationChecker {
    path: Option<PathBuf>,
    last_modified: Option<std::time::SystemTime>,
}

impl FileModificationChecker {
    /// 创建新的检测器 / Create a new checker
    pub fn new() -> Self {
        Self {
            path: None,
            last_modified: None,
        }
    }

    /// 设置要监控的文件 / Set the file to watch
    pub fn set_file(&mut self, path: &PathBuf) {
        self.clear();
        self.path = Some(path.clone());
        // 初始化时记录，避免因首次检查产生误报
        // Record at init time to avoid a false positive on first check
        self.last_modified = Self::get_modified_time(path);
    }

    /// 清除监控 / Clear the watch
    pub fn clear(&mut self) {
        self.path = None;
        self.last_modified = None;
    }

    /// 检查文件是否被外部修改（忽略自身保存导致的修改）
    /// Check if the file was modified externally (ignores our own saves after `update`)
    pub fn check_modified(&mut self) -> bool {
        let Some(ref path) = self.path else {
            return false;
        };
        let Some(current) = Self::get_modified_time(path) else {
            return false;
        };
        if let Some(last) = self.last_modified {
            if current > last {
                return true;
            }
        }
        false
    }

    /// 更新最后修改时间（在自身保存后调用）
    /// Update last modified time (call after our own save)
    pub fn update(&mut self) {
        if let Some(ref path) = self.path {
            self.last_modified = Self::get_modified_time(path);
        }
    }

    /// 获取文件修改时间 / Get file modification time
    fn get_modified_time(path: &PathBuf) -> Option<std::time::SystemTime> {
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
        assert!(checker.path.is_some());
        checker.clear();
        assert!(checker.path.is_none());
        assert!(!checker.check_modified());
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
