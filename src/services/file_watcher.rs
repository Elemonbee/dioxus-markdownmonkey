//! 文件监控服务 / File Watcher Service
//!
//! 用修改时间轮询检测外部修改，避免引入文件系统事件 crate
//! Detect external edits by polling mtime, without a filesystem-event crate

use std::path::PathBuf;

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
}
