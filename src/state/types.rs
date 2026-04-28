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
#[derive(Clone, Debug, PartialEq)]
pub struct TabInfo {
    pub path: Option<PathBuf>, // 文件路径 / File Path
    pub title: String,         // 标签标题 / Tab Title
    pub content: String,       // 标签内容 / Tab Content
    pub modified: bool,        // 是否修改 / Is Modified
    pub history: History,      // 撤销/重做历史 / Undo/Redo History
}

impl TabInfo {
    /// 创建新标签 / Create New Tab
    pub fn new(title: &str) -> Self {
        Self {
            path: None,
            title: title.to_string(),
            content: String::new(),
            modified: false,
            history: History::default(),
        }
    }

    /// 从文件创建标签 / Create Tab from File
    pub fn from_file(path: PathBuf, content: String) -> Self {
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("未命名")
            .to_string();
        Self {
            path: Some(path),
            title,
            content,
            modified: false,
            history: History::default(),
        }
    }
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

        let max_size = if largest_snapshot_len > LARGE_FILE_HISTORY_THRESHOLD {
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
