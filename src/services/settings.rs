//! 设置持久化服务 / Settings Persistence Service

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
    /// PDF 导出中文字体路径（可选）/ Optional PDF CJK font path
    #[serde(default)]
    pub pdf_cjk_font_path: Option<String>,
    /// 启动时恢复上次会话（标签/工作区）/ Restore last session (tabs/workspace) on launch
    #[serde(default = "default_session_restore_enabled")]
    pub session_restore_enabled: bool,
    /// 窗口宽度 / Window Width
    #[serde(default = "default_window_width")]
    pub window_width: f64,
    /// 窗口高度 / Window Height
    #[serde(default = "default_window_height")]
    pub window_height: f64,
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

fn default_session_restore_enabled() -> bool {
    true
}

fn default_window_width() -> f64 {
    1200.0
}

fn default_window_height() -> f64 {
    800.0
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
            pdf_cjk_font_path: None,
            session_restore_enabled: true,
            window_width: 1200.0,
            window_height: 800.0,
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
            base_url: "https://api.openai.com/v1".to_string(),
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
    pub fn get_config_dir() -> io::Result<PathBuf> {
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

/// 便捷函数：仅保存窗口尺寸（不影响其他设置）/ Save only window size (doesn't affect other settings)
pub fn save_window_size(width: f64, height: f64) -> io::Result<()> {
    let service = SettingsService::new()?;
    let mut settings = service.load().unwrap_or_default();
    settings.window_width = width;
    settings.window_height = height;
    service.save(&settings)
}

/// AI 历史文件名（旧版全局）/ Legacy global AI history filename
const AI_HISTORY_FILENAME: &str = "ai_history.json";
/// 按文档隔离的 AI 历史目录 / Per-document AI history directory
const AI_HISTORY_DIRNAME: &str = "ai_history";
/// 持久化历史上限（与内存一致：10 轮 / 20 条）
/// Persisted history cap (matches in-memory: 10 turns / 20 messages)
pub const AI_HISTORY_MAX_MESSAGES: usize = 20;

/// 按文档键解析历史文件路径 / Resolve history file path for a document key
pub fn ai_history_path_for_key(key: &str) -> io::Result<PathBuf> {
    let dir = SettingsService::get_config_dir()?.join(AI_HISTORY_DIRNAME);
    fs::create_dir_all(&dir)?;
    let safe = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    Ok(dir.join(format!("{safe}.json")))
}

/// 一次性把旧版全局历史迁移到指定文档键 / One-time migrate legacy global history into a document key
fn migrate_legacy_ai_history_once(target_key: &str) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static MIGRATED: AtomicBool = AtomicBool::new(false);
    if MIGRATED.swap(true, Ordering::SeqCst) {
        return;
    }
    let Ok(root) = SettingsService::get_config_dir() else {
        return;
    };
    let legacy = root.join(AI_HISTORY_FILENAME);
    if !legacy.exists() {
        return;
    }
    let Ok(dest) = ai_history_path_for_key(target_key) else {
        return;
    };
    if dest.exists() {
        return;
    }
    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::rename(&legacy, &dest) {
        Ok(()) => tracing::info!("Migrated legacy AI history to {:?}", dest),
        Err(_) => {
            if fs::copy(&legacy, &dest).is_ok() {
                let _ = fs::remove_file(&legacy);
                tracing::info!("Copied legacy AI history to {:?}", dest);
            }
        }
    }
}

/// 加载指定文档的 AI 会话历史 / Load AI conversation history for a document key
pub fn load_ai_history_for_key(key: &str) -> Vec<crate::state::ChatTurn> {
    if key.is_empty() {
        return Vec::new();
    }
    migrate_legacy_ai_history_once(key);
    let Ok(path) = ai_history_path_for_key(key) else {
        return Vec::new();
    };
    load_ai_history_from(&path)
}

/// 保存指定文档的 AI 会话历史 / Save AI conversation history for a document key
pub fn save_ai_history_for_key(key: &str, history: &[crate::state::ChatTurn]) -> io::Result<()> {
    let path = ai_history_path_for_key(key)?;
    save_ai_history_to(&path, history)
}

/// 加载 AI 会话历史（兼容旧 API：无键时读旧全局文件）
/// Load AI history (compat: no key reads legacy global file)
#[allow(dead_code)] // 兼容旧调用与迁移路径 / Kept for legacy callers / migration
pub fn load_ai_history() -> Vec<crate::state::ChatTurn> {
    let Ok(dir) = SettingsService::get_config_dir() else {
        return Vec::new();
    };
    load_ai_history_from(&dir.join(AI_HISTORY_FILENAME))
}

/// 从指定路径加载 AI 历史（测试可注入临时路径）
/// Load AI history from a path (tests may inject a temp path)
pub fn load_ai_history_from(path: &std::path::Path) -> Vec<crate::state::ChatTurn> {
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<Vec<crate::state::ChatTurn>>(&content) {
            Ok(mut turns) => {
                if turns.len() > AI_HISTORY_MAX_MESSAGES {
                    let excess = turns.len() - AI_HISTORY_MAX_MESSAGES;
                    turns.drain(0..excess);
                }
                turns
            }
            Err(e) => {
                tracing::warn!("Failed to parse AI history: {}", e);
                Vec::new()
            }
        },
        Err(e) => {
            tracing::warn!("Failed to read AI history: {}", e);
            Vec::new()
        }
    }
}

/// 保存 AI 会话历史（兼容旧 API）/ Save AI history (legacy API)
#[allow(dead_code)] // 兼容旧调用 / Kept for legacy callers
pub fn save_ai_history(history: &[crate::state::ChatTurn]) -> io::Result<()> {
    let dir = SettingsService::get_config_dir()?;
    fs::create_dir_all(&dir)?;
    save_ai_history_to(&dir.join(AI_HISTORY_FILENAME), history)
}

/// 保存 AI 历史到指定路径 / Save AI history to a specific path
pub fn save_ai_history_to(
    path: &std::path::Path,
    history: &[crate::state::ChatTurn],
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let slice = if history.len() > AI_HISTORY_MAX_MESSAGES {
        &history[history.len() - AI_HISTORY_MAX_MESSAGES..]
    } else {
        history
    };
    let content = serde_json::to_string_pretty(slice)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ChatTurn;

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
        assert!(settings.pdf_cjk_font_path.is_none());
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
                "base_url": "https://api.openai.com/v1",
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
        assert!(settings.pdf_cjk_font_path.is_none());
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

    #[test]
    fn test_ai_history_serde_roundtrip() {
        let turns = vec![
            ChatTurn::user("hello"),
            ChatTurn::assistant("world"),
        ];
        let json = serde_json::to_string(&turns).unwrap();
        let back: Vec<ChatTurn> = serde_json::from_str(&json).unwrap();
        assert_eq!(turns, back);
    }

    #[test]
    fn test_save_ai_history_trims_to_max() {
        let dir = std::env::temp_dir().join(format!(
            "mm_ai_hist_trim_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ai_history.json");

        let mut turns = Vec::new();
        for i in 0..AI_HISTORY_MAX_MESSAGES + 4 {
            turns.push(ChatTurn::user(format!("u{i}")));
        }
        save_ai_history_to(&path, &turns).unwrap();
        let loaded = load_ai_history_from(&path);
        assert_eq!(loaded.len(), AI_HISTORY_MAX_MESSAGES);
        assert_eq!(loaded[0].content, format!("u{}", 4));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_ai_history_corrupt_returns_empty() {
        let dir = std::env::temp_dir().join(format!(
            "mm_ai_hist_bad_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ai_history.json");
        fs::write(&path, "{not-json").unwrap();
        let loaded = load_ai_history_from(&path);
        assert!(loaded.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
