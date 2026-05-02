#![allow(dead_code)]
//! 设置持久化服务 / Settings Persistence Service
//!
//! 注意：部分功能为预留功能，暂未使用
//! Note: Some functions are reserved for future use, not yet used

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// 应用设置 / Application Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 主题 / Theme
    pub theme: String,
    /// 语言 / Language
    pub language: String,
    /// 字体大小 / Font Size
    pub font_size: u32,
    /// 预览字体大小 / Preview Font Size
    pub preview_font_size: u32,
    /// 自动换行 / Word Wrap
    pub word_wrap: bool,
    /// 显示行号 / Show Line Numbers
    pub line_numbers: bool,
    /// 同步滚动 / Sync Scroll
    pub sync_scroll: bool,
    /// 侧边栏可见 / Sidebar Visible
    pub sidebar_visible: bool,
    /// 预览可见 / Preview Visible
    pub show_preview: bool,
    /// 侧边栏宽度 / Sidebar Width
    pub sidebar_width: u32,
    /// 自动保存启用 / Auto Save Enabled
    #[serde(default)]
    pub auto_save_enabled: bool,
    /// 自动保存间隔（秒）/ Auto Save Interval (seconds)
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval: u32,
    /// 拼写检查启用 / Spell Check Enabled
    #[serde(default)]
    pub spell_check_enabled: bool,
    /// AI 配置 / AI Configuration
    pub ai: AISettings,
}

/// AI 设置 / AI Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISettings {
    /// 是否启用 / Is Enabled
    pub enabled: bool,
    /// 提供商 / Provider
    pub provider: String,
    /// 模型 / Model
    pub model: String,
    /// API Key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Base URL
    pub base_url: String,
    /// 系统提示 / System Prompt
    pub system_prompt: String,
    /// 温度 / Temperature
    pub temperature: f32,
}

fn default_auto_save_interval() -> u32 {
    30
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "zh-CN".to_string(),
            font_size: 16,
            preview_font_size: 16,
            word_wrap: false,
            line_numbers: true,
            sync_scroll: true,
            sidebar_visible: true,
            show_preview: true,
            sidebar_width: 280,
            auto_save_enabled: false,
            auto_save_interval: 30,
            spell_check_enabled: false,
            ai: AISettings::default(),
        }
    }
}

impl Default for AISettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: None,
            base_url: "https://api.openai.com".to_string(),
            system_prompt: "You are a helpful assistant for markdown writing.".to_string(),
            temperature: 0.7,
        }
    }
}

/// 设置服务 / Settings Service
pub struct SettingsService {
    config_path: PathBuf, // 配置文件路径 / Config File Path
}

impl SettingsService {
    /// 创建新的设置服务 / Create New Settings Service
    pub fn new() -> io::Result<Self> {
        let config_dir = Self::get_config_dir()?;
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("settings.json");

        Ok(Self { config_path })
    }

    /// 获取配置目录 / Get Config Directory
    fn get_config_dir() -> io::Result<PathBuf> {
        // 尝试使用标准配置目录 / Try to use standard config directory
        if let Some(home) = dirs::config_dir() {
            return Ok(home.join("MarkdownMonkey"));
        }

        // 回退到当前目录 / Fallback to current directory
        Ok(PathBuf::from("."))
    }

    /// 加载设置 / Load Settings
    pub fn load(&self) -> io::Result<AppSettings> {
        if !self.config_path.exists() {
            return Ok(AppSettings::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let settings: AppSettings =
            serde_json::from_str(&content).unwrap_or_else(|_| AppSettings::default());

        Ok(settings)
    }

    /// 保存设置 / Save Settings
    /// 安全措施：保存前清除 api_key 明文，确保 API Key 仅存储在系统密钥环中
    /// Security: clear api_key before saving to ensure API Key is only in system keyring
    pub fn save(&self, settings: &AppSettings) -> io::Result<()> {
        // 创建一个副本用于保存，将 api_key 设为 None
        // Create a copy for saving, set api_key to None
        let mut safe_settings = settings.clone();
        safe_settings.ai.api_key = None;

        let content = serde_json::to_string_pretty(&safe_settings)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// 重置为默认设置 / Reset to Default Settings
    pub fn reset(&self) -> io::Result<AppSettings> {
        let settings = AppSettings::default();
        self.save(&settings)?;
        Ok(settings)
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            config_path: PathBuf::from("settings.json"),
        })
    }
}

/// 便捷函数：加载设置 / Convenience Function: Load Settings
pub fn load_settings() -> AppSettings {
    SettingsService::new()
        .and_then(|s| s.load())
        .unwrap_or_default()
}

/// 便捷函数：保存设置 / Convenience Function: Save Settings
pub fn save_settings(settings: &AppSettings) -> io::Result<()> {
    let service = SettingsService::new()?;
    service.save(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default_values() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.language, "zh-CN");
        assert_eq!(settings.font_size, 16);
        assert!(!settings.auto_save_enabled);
        assert_eq!(settings.auto_save_interval, 30);
        assert!(!settings.word_wrap);
        assert!(settings.line_numbers);
        assert!(settings.sync_scroll);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.theme, deserialized.theme);
        assert_eq!(settings.font_size, deserialized.font_size);
        assert_eq!(settings.auto_save_enabled, deserialized.auto_save_enabled);
        assert_eq!(settings.auto_save_interval, deserialized.auto_save_interval);
    }

    #[test]
    fn test_settings_backward_compatible() {
        // 模拟旧版本设置文件（没有 auto_save 字段）
        let old_json = r#"{
            "theme": "light",
            "language": "en-US",
            "font_size": 14,
            "preview_font_size": 14,
            "word_wrap": true,
            "line_numbers": false,
            "sync_scroll": false,
            "sidebar_visible": true,
            "show_preview": true,
            "sidebar_width": 300,
            "ai": {
                "enabled": false,
                "provider": "openai",
                "model": "gpt-4o-mini",
                "base_url": "https://api.openai.com",
                "system_prompt": "test",
                "temperature": 0.7
            }
        }"#;
        let settings: AppSettings = serde_json::from_str(old_json).unwrap();
        assert_eq!(settings.theme, "light");
        assert_eq!(settings.font_size, 14);
        // 新字段应该使用默认值 / New fields should use defaults
        assert!(!settings.auto_save_enabled);
        assert_eq!(settings.auto_save_interval, 30);
    }

    #[test]
    fn test_ai_settings_default() {
        let ai = AISettings::default();
        assert!(!ai.enabled);
        assert_eq!(ai.provider, "openai");
        assert!(ai.api_key.is_none());
        assert_eq!(ai.temperature, 0.7);
    }

    #[test]
    fn test_ai_settings_no_api_key_in_json() {
        let ai = AISettings::default();
        let json = serde_json::to_string(&ai).unwrap();
        // API Key 为 None 时不应该出现在 JSON 中 / API Key should not appear in JSON when None
        assert!(!json.contains("api_key"));
    }
}
