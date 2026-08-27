//! 最近文件服务 / Recent Files Service

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
    ///
    /// 优先使用 MarkdownMonkey 配置目录；若仅有旧版 lowercase 路径则迁移一次。
    /// Prefer MarkdownMonkey config dir; one-time migrate from legacy lowercase path.
    fn config_path() -> PathBuf {
        let preferred = crate::services::settings::SettingsService::get_config_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("recent_files.json");

        if preferred.exists() {
            return preferred;
        }

        let legacy = crate::utils::paths::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("markdownmonkey")
            .join("recent_files.json");

        if legacy.exists() {
            if let Some(parent) = preferred.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if fs::copy(&legacy, &preferred).is_ok() {
                tracing::info!("Migrated recent_files.json to {:?}", preferred);
                return preferred;
            }
            return legacy;
        }

        if let Some(parent) = preferred.parent() {
            let _ = fs::create_dir_all(parent);
        }
        preferred
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
    pub fn add(&mut self, path: PathBuf) {
        // 移除已存在的项 / Remove existing entry
        self.files.retain(|f| f.path != path);

        // 获取文件名 / Get file name
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知文件/Unknown")
            .to_string();

        // 添加到开头 / Add to beginning
        self.files.insert(
            0,
            RecentFile {
                path,
                name,
                last_opened: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            },
        );

        // 限制数量 / Limit count
        while self.files.len() > self.max_count {
            self.files.pop();
        }
    }

    /// 清空最近文件列表 / Clear recent files list
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// 仅保留仍然存在的文件 / Keep only paths that still exist on disk
    pub fn prune_missing(&mut self) {
        self.files.retain(|f| f.path.exists());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn clear_empties_list() {
        let mut recent = RecentFiles::new();
        recent.add(PathBuf::from("a.md"));
        recent.add(PathBuf::from("b.md"));
        recent.clear();
        assert!(recent.files.is_empty());
    }

    #[test]
    fn prune_missing_keeps_existing_only() {
        let temp = TempDir::new().unwrap();
        let existing = temp.path().join("keep.md");
        fs::write(&existing, "x").unwrap();
        let mut recent = RecentFiles::new();
        recent.add(existing.clone());
        recent.add(temp.path().join("gone.md"));
        recent.prune_missing();
        assert_eq!(recent.files.len(), 1);
        assert_eq!(recent.files[0].path, existing);
    }
}
