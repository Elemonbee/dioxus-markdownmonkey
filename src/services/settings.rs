//! 设置持久化服务 / Settings Persistence Service

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use dioxus::prelude::ReadableExt;

use crate::state::{AppState, Language, Theme};

/// 进程内设置写锁 / Process-wide settings write lock
static SETTINGS_WRITE_LOCK: Mutex<()> = Mutex::new(());

/// 应用设置 / Application Settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            word_wrap: true,
            line_numbers: true,
            sync_scroll: true,
            sidebar_visible: true,
            show_preview: true,
            sidebar_width: 280,
            auto_save_enabled: false,
            auto_save_interval: 30,
            session_restore_enabled: true,
            window_width: 1200.0,
            window_height: 800.0,
            ai: AISettings::default(),
        }
    }
}

impl AppSettings {
    /// 从当前应用状态生成完整设置快照，并保留调用方提供的窗口尺寸
    /// Build a complete settings snapshot from app state while preserving supplied window dimensions
    pub fn from_state(state: &AppState, window_width: f64, window_height: f64) -> Self {
        let ui = state.ui();
        let ai = state.ai();
        let config = ai.ai_config.read();

        let settings = Self {
            theme: match *ui.theme.read() {
                Theme::Dark => "dark",
                Theme::Light => "light",
                Theme::System => "system",
            }
            .to_string(),
            language: match *ui.language.read() {
                Language::ZhCN => "zh-CN",
                Language::EnUS => "en-US",
            }
            .to_string(),
            font_size: *ui.font_size.read(),
            preview_font_size: *ui.preview_font_size.read(),
            word_wrap: *ui.word_wrap.read(),
            line_numbers: *ui.line_numbers.read(),
            sync_scroll: *ui.sync_scroll.read(),
            sidebar_visible: *ui.sidebar_visible.read(),
            show_preview: *ui.show_preview.read(),
            sidebar_width: *ui.sidebar_width.read(),
            auto_save_enabled: *ui.auto_save_enabled.read(),
            auto_save_interval: *ui.auto_save_interval.read(),
            session_restore_enabled: *ui.session_restore_enabled.read(),
            window_width,
            window_height,
            ai: AISettings {
                enabled: config.enabled,
                provider: config.provider.as_str().to_string(),
                model: config.model.clone(),
                api_key: None,
                base_url: config.base_url.clone(),
                system_prompt: config.system_prompt.clone(),
                temperature: config.temperature,
            },
        };
        settings
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

    /// 使用指定路径创建设置服务，便于测试隔离
    /// Create a settings service for an explicit path to isolate tests
    #[cfg(test)]
    fn from_path(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// 获取配置目录 / Get Config Directory
    pub fn get_config_dir() -> io::Result<PathBuf> {
        // 尝试使用标准配置目录 / Try to use standard config directory
        if let Some(home) = crate::utils::paths::config_dir() {
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
        let _guard = SETTINGS_WRITE_LOCK
            .lock()
            .map_err(|_| io::Error::other("settings write lock poisoned"))?;
        let mut merged = settings.clone();
        if self.config_path.exists() {
            if let Ok(current) = self.load() {
                merged.window_width = current.window_width;
                merged.window_height = current.window_height;
            }
        }
        self.save_locked(&merged)
    }

    /// 在已持有全局写锁时安全保存设置
    /// Save settings safely while the global write lock is held
    fn save_locked(&self, settings: &AppSettings) -> io::Result<()> {
        let mut safe_settings = settings.clone();
        safe_settings.ai.api_key = None;
        let content = serde_json::to_vec_pretty(&safe_settings)?;
        atomic_write(&self.config_path, &content)
    }

    /// 若本文件仍有明文 API Key，立刻重写配置去掉它
    /// Rewrite this settings file immediately when a plaintext API key is still present
    pub fn strip_plaintext_api_key_if_present(&self) -> io::Result<bool> {
        let settings = self.load()?;
        if settings
            .ai
            .api_key
            .as_deref()
            .is_some_and(|key| !key.is_empty())
        {
            self.save(&settings)?;
            tracing::info!(
                "已从 settings.json 清除明文 API Key / Cleared plaintext API Key from settings.json"
            );
            return Ok(true);
        }
        Ok(false)
    }

    /// 在同一串行化临界区内更新窗口尺寸
    /// Update window dimensions inside the same serialized critical section
    fn save_window_size(&self, width: f64, height: f64) -> io::Result<()> {
        let _guard = SETTINGS_WRITE_LOCK
            .lock()
            .map_err(|_| io::Error::other("settings write lock poisoned"))?;
        let mut settings = self.load().unwrap_or_default();
        settings.window_width = width;
        settings.window_height = height;
        self.save_locked(&settings)
    }
}

/// 使用同目录临时文件写入并原子替换目标文件
/// Write through a same-directory temporary file and atomically replace the destination
fn atomic_write(path: &Path, content: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let stem = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("settings.json");
    let mut attempt = 0_u32;
    let temp_path = loop {
        let candidate = parent.join(format!(".{stem}.{}.{}.tmp", std::process::id(), attempt));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                if let Err(error) = file.write_all(content).and_then(|_| file.sync_all()) {
                    let _ = fs::remove_file(&candidate);
                    return Err(error);
                }
                break candidate;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                attempt = attempt.wrapping_add(1);
            }
            Err(error) => return Err(error),
        }
    };

    if let Err(error) = atomic_replace(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }
    Ok(())
}

/// 在 Unix 上通过 rename 原子替换目标文件
/// Atomically replace the destination with rename on Unix
#[cfg(not(windows))]
fn atomic_replace(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

/// 在 Windows 上通过 ReplaceFileW 原子替换现有文件
/// Atomically replace an existing file with ReplaceFileW on Windows
#[cfg(windows)]
fn atomic_replace(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    if !destination.exists() {
        return fs::rename(source, destination);
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn ReplaceFileW(
            replaced_file_name: *const u16,
            replacement_file_name: *const u16,
            backup_file_name: *const u16,
            replace_flags: u32,
            exclude: *mut std::ffi::c_void,
            reserved: *mut std::ffi::c_void,
        ) -> i32;
    }

    let destination_wide: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let source_wide: Vec<u16> = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let result = unsafe {
        ReplaceFileW(
            destination_wide.as_ptr(),
            source_wide.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
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

/// 若磁盘上仍有明文 API Key，立刻重写配置去掉它
/// Rewrite settings immediately when a plaintext API key is still on disk
pub fn strip_plaintext_api_key_if_present() -> io::Result<bool> {
    SettingsService::new()?.strip_plaintext_api_key_if_present()
}

/// 便捷函数：仅保存窗口尺寸（不影响其他设置）/ Save only window size (doesn't affect other settings)
pub fn save_window_size(width: f64, height: f64) -> io::Result<()> {
    let service = SettingsService::new()?;
    service.save_window_size(width, height)
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
    use dioxus::prelude::WritableExt;

    /// 在 Dioxus 作用域内运行 Signal 相关测试
    /// Run Signal-related tests inside a Dioxus scope
    fn with_runtime<F: FnOnce()>(test: F) {
        use dioxus::prelude::*;

        /// 提供测试所需的空组件 / Provide an empty component for tests
        fn empty_component() -> Element {
            rsx! { div {} }
        }

        let vdom = VirtualDom::prebuilt(empty_component);
        vdom.in_scope(ScopeId::ROOT, test);
    }

    #[test]
    fn test_settings_default_values() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.language, "zh-CN");
        assert_eq!(settings.font_size, 16);
        assert!(!settings.auto_save_enabled);
        assert_eq!(settings.auto_save_interval, 30);
        assert!(settings.word_wrap);
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

    /// AppState 快照应包含工具栏即时设置且永不包含明文密钥
    /// AppState snapshots include immediate toolbar settings and never plaintext keys
    #[test]
    fn test_app_state_mapping_captures_immediate_settings_without_api_key() {
        with_runtime(|| {
            let mut state = AppState::new();
            *state.theme.write() = Theme::Light;
            *state.language.write() = Language::EnUS;
            *state.show_preview.write() = false;
            *state.sidebar_visible.write() = false;
            *state.auto_save_enabled.write() = true;
            state.ai_config.write().api_key = "secret".to_string();

            let settings = AppSettings::from_state(&state, 900.0, 700.0);
            assert_eq!(settings.theme, "light");
            assert_eq!(settings.language, "en-US");
            assert!(!settings.show_preview);
            assert!(!settings.sidebar_visible);
            assert!(settings.auto_save_enabled);
            assert!(settings.ai.api_key.is_none());
        });
    }

    /// 设置保存应原子替换并清理同目录临时文件
    /// Settings saves atomically replace the destination and clean same-directory temporary files
    #[test]
    fn test_settings_save_atomically_replaces_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("settings.json");
        let service = SettingsService::from_path(path.clone());
        service.save(&AppSettings::default()).unwrap();

        let mut updated = AppSettings {
            theme: "light".to_string(),
            ..AppSettings::default()
        };
        updated.ai.api_key = Some("never-on-disk".to_string());
        service.save(&updated).unwrap();

        let saved: AppSettings = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved.theme, "light");
        assert!(saved.ai.api_key.is_none());
        assert!(fs::read_dir(temp.path()).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
    }

    /// 迁移后应立即从已有 settings.json 去掉明文 API Key
    /// After migration, plaintext API keys must be stripped from an existing settings.json
    #[test]
    fn test_strip_plaintext_api_key_rewrites_existing_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(
            &path,
            r#"{
                "theme": "dark",
                "language": "zh-CN",
                "font_size": 16,
                "preview_font_size": 16,
                "word_wrap": false,
                "line_numbers": true,
                "sync_scroll": true,
                "sidebar_visible": true,
                "show_preview": true,
                "sidebar_width": 280,
                "ai": {
                    "enabled": true,
                    "provider": "openai",
                    "model": "gpt-4o-mini",
                    "api_key": "sk-legacy-plaintext",
                    "base_url": "https://api.openai.com/v1",
                    "system_prompt": "test",
                    "temperature": 0.7
                }
            }"#,
        )
        .unwrap();

        let service = SettingsService::from_path(path.clone());
        assert!(service.strip_plaintext_api_key_if_present().unwrap());
        let raw = fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("sk-legacy-plaintext"));
        assert!(!raw.contains("api_key"));
        let saved: AppSettings = serde_json::from_str(&raw).unwrap();
        assert!(saved.ai.api_key.is_none());
        assert!(saved.ai.enabled);
    }

    /// 并发全量保存与窗口保存应合并，不能互相覆盖
    /// Concurrent full and window saves merge without overwriting each other
    #[test]
    fn test_concurrent_full_and_window_saves_are_serialized_and_merged() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("settings.json");
        SettingsService::from_path(path.clone())
            .save(&AppSettings::default())
            .unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));

        let full_path = path.clone();
        let full_barrier = barrier.clone();
        let full = std::thread::spawn(move || {
            let service = SettingsService::from_path(full_path);
            let settings = AppSettings {
                theme: "light".to_string(),
                ..AppSettings::default()
            };
            full_barrier.wait();
            service.save(&settings).unwrap();
        });

        let window_path = path.clone();
        let window_barrier = barrier.clone();
        let window = std::thread::spawn(move || {
            let service = SettingsService::from_path(window_path);
            window_barrier.wait();
            service.save_window_size(1440.0, 960.0).unwrap();
        });

        barrier.wait();
        full.join().unwrap();
        window.join().unwrap();
        let saved = SettingsService::from_path(path).load().unwrap();
        assert_eq!(saved.theme, "light");
        assert_eq!(saved.window_width, 1440.0);
        assert_eq!(saved.window_height, 960.0);
    }

    #[test]
    fn test_ai_history_serde_roundtrip() {
        let turns = vec![ChatTurn::user("hello"), ChatTurn::assistant("world")];
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
