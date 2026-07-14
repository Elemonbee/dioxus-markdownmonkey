//! 应用状态相关数据结构 / Data structures for application state
//!
//! 将类型从 `AppState` 中拆分出来，便于维护与单测
//! Split types out from `AppState` for maintainability and focused tests

use std::path::PathBuf;

/// 主题枚举 / Theme Enum
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
    #[default]
    Dark, // 深色主题 / Dark Theme
    Light,  // 浅色主题 / Light Theme
    System, // 系统主题 / System Theme
}

/// 语言枚举 / Language Enum
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Language {
    #[default]
    ZhCN, // 简体中文 / Simplified Chinese
    EnUS, // 美式英语 / American English
}

/// 保存状态 / Save Status
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SaveStatus {
    #[default]
    Saved, // 已保存 / Saved
    Saving,  // 保存中 / Saving
    Unsaved, // 未保存 / Unsaved
}

/// 标签信息 / Tab Information
///
/// `content` 使用 `Arc<str>` 共享只读正文；`None` 表示已驱逐（未修改且已有路径时可从磁盘重载）
/// `content` is an `Arc<str>` shared body; `None` means evicted (reload from disk when unmodified)
#[derive(Clone, Debug, PartialEq)]
pub struct TabInfo {
    pub path: Option<PathBuf>, // 文件路径 / File Path
    pub title: String,         // 标签标题 / Tab Title
    pub modified: bool,        // 是否修改 / Is Modified
    /// 驻留正文；驱逐后为 None / Resident body; None after eviction
    pub content: Option<std::sync::Arc<str>>,
    pub history: History, // 撤销/重做历史 / Undo/Redo History
    /// LRU 访问时钟（越大越新）/ LRU access clock (higher = more recent)
    pub last_accessed: u64,
    /// 按文档隔离的 AI 会话键 / Per-document AI session key
    pub ai_session_key: String,
}

impl TabInfo {
    /// 创建新标签 / Create New Tab
    pub fn new(title: &str) -> Self {
        let mut history = History::default();
        history.reset_with_content("");
        Self {
            path: None,
            title: title.to_string(),
            content: Some(std::sync::Arc::from("")),
            modified: false,
            history,
            last_accessed: 0,
            ai_session_key: new_untitled_ai_session_key(),
        }
    }

    /// 从文件创建标签（内容以 Arc 共享，避免额外 String 拷贝）
    /// Create tab from file (share body via Arc to avoid an extra String copy)
    pub fn from_file(path: PathBuf, content: &str) -> Self {
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();
        let ai_session_key = ai_session_key_for_path(&path);
        let mut history = History::default();
        history.reset_with_content(content);
        Self {
            path: Some(path),
            title,
            content: Some(std::sync::Arc::from(content)),
            modified: false,
            history,
            last_accessed: 0,
            ai_session_key,
        }
    }

    /// 是否已被驱逐出内存 / Whether the tab body was evicted from memory
    pub fn is_evicted(&self) -> bool {
        self.content.is_none()
    }

    /// 读取正文（驱逐后返回空串）/ Read body text (empty string if evicted)
    #[allow(dead_code)] // 测试与调试辅助 / Helper for tests and debugging
    pub fn content_str(&self) -> &str {
        self.content.as_deref().unwrap_or("")
    }

    /// 用 Arc 设置驻留正文 / Set resident body from an Arc
    pub fn set_content_arc(&mut self, content: std::sync::Arc<str>) {
        self.content = Some(content);
    }

    /// 在可安全重载时驱逐正文与历史 / Evict body and history when safe to reload
    pub fn try_evict(&mut self) -> bool {
        if self.modified || self.path.is_none() || self.content.is_none() {
            return false;
        }
        self.content = None;
        self.history = History::default();
        true
    }
}

/// 为未命名标签生成稳定 AI 会话键 / Generate a stable AI session key for untitled tabs
pub fn new_untitled_ai_session_key() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("untitled-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// 由文件路径派生 AI 会话键 / Derive AI session key from a file path
pub fn ai_session_key_for_path(path: &std::path::Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let normalized = path.to_string_lossy().to_lowercase().replace('\\', "/");
    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    format!("path-{:016x}", hasher.finish())
}

/// 大纲项 / Outline Item
#[derive(Clone, Debug)]
pub struct OutlineItem {
    pub level: u8,    // 标题级别 / Heading Level (1-6)
    pub text: String, // 标题文本 / Heading Text
    pub line: usize,  // 行号 / Line Number
}

/// AI 提供商 / AI Provider
#[derive(Clone, Debug, Default, PartialEq)]
pub enum AIProvider {
    #[default]
    OpenAI, // OpenAI API
    Claude,     // Anthropic Claude
    Ollama,     // Ollama Local
    DeepSeek,   // DeepSeek API
    Kimi,       // Moonshot Kimi
    OpenRouter, // OpenRouter API
}

impl AIProvider {
    /// 用于持久化与密钥环查找的稳定提供商 ID / Stable ID for persistence and keyring lookups
    pub fn as_str(&self) -> &'static str {
        match self {
            AIProvider::OpenAI => "openai",
            AIProvider::Claude => "claude",
            AIProvider::Ollama => "ollama",
            AIProvider::DeepSeek => "deepseek",
            AIProvider::Kimi => "kimi",
            AIProvider::OpenRouter => "openrouter",
        }
    }
}

/// AI 配置 / AI Configuration
#[derive(Clone, Debug, PartialEq)]
pub struct AIConfig {
    pub enabled: bool,         // 是否启用 / Is Enabled
    pub provider: AIProvider,  // 提供商 / Provider
    pub model: String,         // 模型名称 / Model Name
    pub api_key: String,       // API 密钥 / API Key
    pub base_url: String,      // 基础 URL / Base URL
    pub system_prompt: String, // 系统提示词 / System Prompt
    pub temperature: f32,      // 温度参数 / Temperature
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AIProvider::OpenAI,
            model: "gpt-4o-mini".to_string(),
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            system_prompt: "You are a helpful assistant for markdown writing.".to_string(),
            temperature: 0.7,
        }
    }
}

/// AI 会话轮次（不含 system）/ AI conversation turn (excluding system)
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatTurn {
    /// 角色：user 或 assistant / Role: user or assistant
    pub role: String,
    /// 文本内容 / Text content
    pub content: String,
}

impl ChatTurn {
    /// 创建用户轮次 / Create a user turn
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// 创建助手轮次 / Create an assistant turn
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// 侧边栏标签 / Sidebar Tab
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SidebarTab {
    #[default]
    Outline, // 大纲 / Outline
    Files, // 文件 / Files
}

/// 历史记录最大容量 / History Maximum Capacity
pub(crate) const MAX_HISTORY_SIZE: usize = 50;
/// 触发内存优化阈值 (字节) / Memory optimization threshold (bytes)
pub(crate) const LARGE_FILE_HISTORY_THRESHOLD: usize = 100 * 1024; // 100KB
/// 大文件时减少历史容量 / Reduced history capacity for large files
pub(crate) const LARGE_FILE_MAX_HISTORY: usize = 10;
/// 超大文件阈值 (字节) / Huge-file threshold (bytes)
pub(crate) const HUGE_FILE_HISTORY_THRESHOLD: usize = 1024 * 1024; // 1MB
/// 超大文件历史容量 / History capacity for huge files
pub(crate) const HUGE_FILE_MAX_HISTORY: usize = 5;

/// 历史记录（用于撤销/重做）/ History (for Undo/Redo)
/// 使用 Arc<str> 共享不可变字符串，避免多标签切换时重复克隆内容
/// Uses Arc<str> to share immutable strings, avoiding repeated content cloning during tab switches
/// 使用 VecDeque 替代 Vec，使 pop_front/push_back 均为 O(1)
/// Uses VecDeque instead of Vec so pop_front/push_back are both O(1)
#[derive(Clone, Debug)]
pub struct History {
    pub past: std::collections::VecDeque<std::sync::Arc<str>>, // 过去状态（共享引用）/ Past States (shared references)
    pub future: std::collections::VecDeque<std::sync::Arc<str>>, // 未来状态（共享引用）/ Future States (shared references)
    /// 当前内容的哈希，用于检测实际变化 / Hash of current content for change detection
    last_hash: u64,
}

impl PartialEq for History {
    fn eq(&self, other: &Self) -> bool {
        // 比较 Arc 内容而非指针 / Compare Arc contents, not pointers
        self.past.len() == other.past.len()
            && self.future.len() == other.future.len()
            && self
                .past
                .iter()
                .zip(other.past.iter())
                .all(|(a, b)| a.as_ref() == b.as_ref())
            && self
                .future
                .iter()
                .zip(other.future.iter())
                .all(|(a, b)| a.as_ref() == b.as_ref())
    }
}

impl Default for History {
    fn default() -> Self {
        Self {
            past: std::collections::VecDeque::with_capacity(MAX_HISTORY_SIZE),
            future: std::collections::VecDeque::with_capacity(MAX_HISTORY_SIZE),
            last_hash: 0,
        }
    }
}

impl History {
    /// 清空历史并用当前正文种子化哈希（打开/切换文件后避免误标未保存）
    /// Clear history and seed hash from current body (avoids false unsaved after open/switch)
    pub fn reset_with_content(&mut self, content: &str) {
        self.past.clear();
        self.future.clear();
        self.last_hash = Self::hash(content);
    }

    /// 计算字符串的简单哈希 / Calculate simple hash of string
    /// 结合长度和哈希值降低碰撞风险 / Combine length and hash to reduce collision risk
    fn hash(s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // 先哈希长度，再哈希内容 / Hash length first, then content
        (s.len() as u64).hash(&mut hasher);
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// 检查内容是否真的改变了 / Check if content actually changed
    pub fn is_different(&mut self, content: &str) -> bool {
        let new_hash = Self::hash(content);
        if new_hash == self.last_hash {
            return false;
        }
        self.last_hash = new_hash;
        true
    }

    /// 添加到历史，使用 Arc 共享字符串内存
    /// Add to history, using Arc to share string memory
    pub fn push(&mut self, content: String) {
        // 根据已有历史中最大内容大小动态调整历史容量，而不是仅依赖本次传入内容
        // Dynamically adjust history capacity based on retained snapshot sizes, not only the new content
        let largest_snapshot_len = self
            .past
            .iter()
            .map(|item| item.len())
            .chain(std::iter::once(content.len()))
            .max()
            .unwrap_or(0);

        let max_size = if largest_snapshot_len > HUGE_FILE_HISTORY_THRESHOLD {
            HUGE_FILE_MAX_HISTORY
        } else if largest_snapshot_len > LARGE_FILE_HISTORY_THRESHOLD {
            LARGE_FILE_MAX_HISTORY
        } else {
            MAX_HISTORY_SIZE
        };

        // 限制历史记录大小（使用 pop_front 避免 O(n) 的 Vec::remove(0)）
        // Limit history size (use pop_front to avoid O(n) Vec::remove(0))
        if self.past.len() >= max_size {
            self.past.pop_front();
        }
        // 使用 Arc<str> 而非 String，避免后续切换标签时重复克隆
        self.past.push_back(std::sync::Arc::from(content.as_str()));
    }
}
