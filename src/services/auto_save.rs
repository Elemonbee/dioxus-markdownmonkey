//! 自动保存服务 / Auto Save Service
//!
//! 由 app.rs 的 use_future 定时调用，定期检查文件修改并保存
//! Called periodically by app.rs use_future to check for modifications and save

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::task;

/// 自动保存服务 / Auto Save Service
pub struct AutoSaveService {
    /// 上次保存时间 / Last Save Time
    last_save: Instant,
    /// 保存间隔（秒）/ Save Interval (seconds)
    interval_secs: u32,
    /// 是否启用 / Is Enabled
    enabled: bool,
}

impl AutoSaveService {
    /// 创建新的自动保存服务 / Create New Auto Save Service
    pub fn new() -> Self {
        Self {
            last_save: Instant::now(),
            interval_secs: 30,
            enabled: false,
        }
    }

    /// 设置启用状态 / Set Enabled Status
    pub fn set_enabled(&mut self, enabled: bool) {
        // 仅在状态发生变化时重置计时器（从禁用→启用时）
        // Only reset timer when actually transitioning from disabled to enabled
        if self.enabled != enabled {
            if enabled {
                self.last_save = Instant::now();
            }
            self.enabled = enabled;
        }
    }

    /// 设置保存间隔 / Set Save Interval
    pub fn set_interval(&mut self, secs: u32) {
        self.interval_secs = secs;
    }

    /// 检查是否需要保存 / Check If Save Is Needed
    pub fn should_save(&self, modified: bool) -> bool {
        if !self.enabled || !modified {
            return false;
        }

        self.last_save.elapsed() >= Duration::from_secs(self.interval_secs as u64)
    }

    /// 更新保存时间 / Update Save Time
    pub fn mark_saved(&mut self) {
        self.last_save = Instant::now();
    }

    /// 获取距离下次保存的剩余秒数 / Get Remaining Seconds Until Next Save
    #[allow(dead_code)]
    pub fn remaining_secs(&self) -> u32 {
        if !self.enabled {
            return 0;
        }

        let elapsed = self.last_save.elapsed().as_secs() as u32;
        self.interval_secs.saturating_sub(elapsed)
    }

    /// 执行自动保存 / Perform Auto Save
    pub async fn auto_save(&mut self, path: Option<&PathBuf>, content: &str) -> Result<(), String> {
        if let Some(path) = path {
            let path = path.clone();
            let content = content.to_string();
            task::spawn_blocking(move || std::fs::write(&path, content))
                .await
                .map_err(|e| format!("自动保存任务失败 / Auto save task failed: {}", e))?
                .map_err(|e| format!("自动保存失败 / Auto save failed: {}", e))?;
            self.mark_saved();
            Ok(())
        } else {
            Err("没有文件路径 / No file path".to_string())
        }
    }
}

impl Default for AutoSaveService {
    fn default() -> Self {
        Self::new()
    }
}
