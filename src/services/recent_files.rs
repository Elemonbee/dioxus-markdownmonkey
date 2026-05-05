//! 最近文件服务 / Recent Files Service
//!
//! 支持 frecency（频率+时效性）评分排序
//! Supports frecency (frequency + recency) scoring and sorting

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 最近文件列表 / Recent Files List
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RecentFiles {
    pub files: Vec<RecentFile>, // 文件列表 / File List
    pub max_count: usize,       // 最大数量 / Max Count
}

/// 最近文件项 / Recent File Item
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: PathBuf,    // 文件路径 / File Path
    pub name: String,     // 文件名 / File Name
    pub last_opened: u64, // 最后打开时间 (Unix 时间戳) / Last Opened Time (Unix Timestamp)
    #[serde(default)]
    pub open_count: u32, // 打开次数 / Open Count
}

impl RecentFile {
    /// 计算 frecency 分数 / Calculate frecency score
    /// 分数 = 打开次数 * 100 + 时间衰减加分
    /// Score = open_count * 100 + time-decay bonus
    pub fn frecency_score(&self) -> f64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let elapsed_secs = now.saturating_sub(self.last_opened);
        let elapsed_hours = elapsed_secs as f64 / 3600.0;
        // 时间衰减：最近打开的文件获得更高加分
        // Time decay: recently opened files get higher bonus
        let recency_bonus = (1000.0 - elapsed_hours * 41.67).max(0.0);
        self.open_count as f64 * 100.0 + recency_bonus
    }
}

impl RecentFiles {
    /// 创建新的最近文件列表 / Create New Recent Files List
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            max_count: 10,
        }
    }

    /// 获取配置文件路径 / Get Config File Path
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("markdownmonkey");

        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }

        config_dir.join("recent_files.json")
    }

    /// 从文件加载 / Load from File
    pub fn load() -> Self {
        let path = Self::config_path();

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(recent) => return recent,
                    Err(e) => {
                        tracing::warn!("Failed to parse recent files: {}", e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read recent files: {}", e);
                }
            }
        }

        Self::new()
    }

    /// 保存到文件 / Save to File
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;

        fs::write(&path, content).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 添加文件到列表 / Add File to List
    /// 累加打开次数并移至列表顶部 / Increments open count and moves to top
    pub fn add(&mut self, path: PathBuf) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 获取文件名 / Get file name
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知文件/Unknown")
            .to_string();

        // 查找已有项并累加打开次数 / Find existing entry and increment open count
        let existing_count = self
            .files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.open_count.saturating_add(1))
            .unwrap_or(1);

        // 移除已存在的项 / Remove existing entry
        self.files.retain(|f| f.path != path);

        // 添加到开头 / Add to beginning
        self.files.insert(
            0,
            RecentFile {
                path,
                name,
                last_opened: now,
                open_count: existing_count,
            },
        );

        // 限制数量 / Limit count
        while self.files.len() > self.max_count {
            self.files.pop();
        }
    }

    /// 移除指定文件 / Remove specific file
    pub fn remove(&mut self, path: &PathBuf) {
        self.files.retain(|f| &f.path != path);
    }

    /// 按 frecency 排序返回 / Return files sorted by frecency score
    pub fn sorted_by_frecency(&self) -> Vec<&RecentFile> {
        let mut files: Vec<&RecentFile> = self.files.iter().collect();
        files.sort_by(|a, b| {
            b.frecency_score()
                .partial_cmp(&a.frecency_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_file(name: &str, last_opened: u64, open_count: u32) -> RecentFile {
        RecentFile {
            path: PathBuf::from(name),
            name: name.to_string(),
            last_opened,
            open_count,
        }
    }

    #[test]
    fn test_add_new_file() {
        let mut rf = RecentFiles::new();
        rf.add(PathBuf::from("/test.md"));
        assert_eq!(rf.files.len(), 1);
        assert_eq!(rf.files[0].name, "test.md");
        assert_eq!(rf.files[0].open_count, 1);
    }

    #[test]
    fn test_add_same_file_increments_count() {
        let mut rf = RecentFiles::new();
        rf.add(PathBuf::from("/test.md"));
        rf.add(PathBuf::from("/test.md"));
        assert_eq!(rf.files.len(), 1);
        assert_eq!(rf.files[0].open_count, 2);
    }

    #[test]
    fn test_add_moves_to_top() {
        let mut rf = RecentFiles::new();
        rf.add(PathBuf::from("/a.md"));
        rf.add(PathBuf::from("/b.md"));
        rf.add(PathBuf::from("/a.md"));
        assert_eq!(rf.files[0].name, "a.md");
        assert_eq!(rf.files[0].open_count, 2);
        assert_eq!(rf.files[1].name, "b.md");
    }

    #[test]
    fn test_max_count_limit() {
        let mut rf = RecentFiles::new();
        rf.max_count = 3;
        rf.add(PathBuf::from("/a.md"));
        rf.add(PathBuf::from("/b.md"));
        rf.add(PathBuf::from("/c.md"));
        rf.add(PathBuf::from("/d.md"));
        assert_eq!(rf.files.len(), 3);
        assert_eq!(rf.files[0].name, "d.md");
    }

    #[test]
    fn test_remove() {
        let mut rf = RecentFiles::new();
        rf.add(PathBuf::from("/a.md"));
        rf.add(PathBuf::from("/b.md"));
        rf.remove(&PathBuf::from("/a.md"));
        assert_eq!(rf.files.len(), 1);
        assert_eq!(rf.files[0].name, "b.md");
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut rf = RecentFiles::new();
        rf.add(PathBuf::from("/a.md"));
        rf.remove(&PathBuf::from("/nonexistent.md"));
        assert_eq!(rf.files.len(), 1);
    }

    #[test]
    fn test_frecency_score_recent_higher() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let recent = make_file("a.md", now, 1);
        let old = make_file("b.md", now - 86400, 1);
        assert!(recent.frecency_score() > old.frecency_score());
    }

    #[test]
    fn test_frecency_score_more_opens_higher() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let frequent = make_file("a.md", now, 5);
        let rare = make_file("b.md", now, 1);
        assert!(frequent.frecency_score() > rare.frecency_score());
    }

    #[test]
    fn test_sorted_by_frecency() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut rf = RecentFiles::new();
        rf.files = vec![
            make_file("old.md", now - 86400, 1),
            make_file("frequent.md", now - 3600, 10),
            make_file("recent.md", now, 1),
        ];
        let sorted = rf.sorted_by_frecency();
        // frequent (10*100 + high recency) should be first, then recent, then old
        assert_eq!(sorted[0].name, "frequent.md");
    }

    #[test]
    fn test_backward_compatible_deserialize() {
        let json =
            r#"{"files":[{"path":"/test.md","name":"test.md","last_opened":1000}],"max_count":10}"#;
        let rf: RecentFiles = serde_json::from_str(json).unwrap();
        assert_eq!(rf.files.len(), 1);
        assert_eq!(rf.files[0].open_count, 0); // default
    }

    #[test]
    fn test_roundtrip_serialize() {
        let mut rf = RecentFiles::new();
        rf.add(PathBuf::from("/test.md"));
        let json = serde_json::to_string(&rf).unwrap();
        let rf2: RecentFiles = serde_json::from_str(&json).unwrap();
        assert_eq!(rf.files[0].name, rf2.files[0].name);
        assert_eq!(rf.files[0].open_count, rf2.files[0].open_count);
    }
}
