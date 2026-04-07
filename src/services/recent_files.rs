//! 最近文件服务 / Recent Files Service
//!
//! 注意：部分功能为预留功能，暂未使用
//! Note: Some functions are reserved for future use, not yet used

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
}
