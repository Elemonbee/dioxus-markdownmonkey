//! 应用全局状态 / Application Global State
//!
//! 使用 Dioxus Signal 实现响应式状态管理 / Using Dioxus Signal for reactive state management

use dioxus::prelude::*;
use std::path::{Path, PathBuf};

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
    /// Stable provider identifier used in persisted settings and keyring lookups.
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

/// 历史记录最大容量 / History Maximum Capacity
const MAX_HISTORY_SIZE: usize = 50;
/// 触发内存优化阈值 (字节) / Memory optimization threshold (bytes)
const LARGE_FILE_HISTORY_THRESHOLD: usize = 100 * 1024; // 100KB
/// 大文件时减少历史容量 / Reduced history capacity for large files
const LARGE_FILE_MAX_HISTORY: usize = 10;

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

/// 应用全局状态 / Application Global State
///
/// # 架构说明 / Architecture Notes
///
/// 当前所有状态集中在一个 struct 中。当项目规模增长后，建议拆分为子模块：
/// - `DocumentState` — 文档、历史记录、保存状态
/// - `EditorState` — 光标、字体、行号等编辑器配置
/// - `UIState` — 主题、语言、弹窗、侧边栏
/// - `SearchState` — 搜索/替换相关
/// - `AIState` — AI 配置与状态
/// - `FileTreeState` — 工作区、文件列表
///
/// Currently all state is centralized. As the project grows, consider splitting into:
/// - `DocumentState` — document, history, save status
/// - `EditorState` — cursor, font, line numbers, etc.
/// - `UIState` — theme, language, modals, sidebar
/// - `SearchState` — search/replace related
/// - `AIState` — AI config and state
/// - `FileTreeState` — workspace, file list
#[derive(Clone, Copy)]
pub struct AppState {
    // ========== 文档状态 / Document State ==========
    /// 当前文件路径 / Current File Path
    pub current_file: Signal<Option<PathBuf>>,
    /// 文档内容 / Document Content
    pub content: Signal<String>,
    /// 是否已修改 / Is Modified
    pub modified: Signal<bool>,
    /// 历史记录 / History
    pub history: Signal<History>,
    /// 保存状态 / Save Status
    pub save_status: Signal<SaveStatus>,
    /// 上次保存时间 / Last Saved Time
    pub last_saved: Signal<Option<std::time::Instant>>,

    // ========== 标签页状态 / Tab State ==========
    /// 打开的标签页 / Open Tabs
    pub tabs: Signal<Vec<TabInfo>>,
    /// 当前标签索引 / Current Tab Index
    pub current_tab_index: Signal<usize>,

    // ========== 编辑器状态 / Editor State ==========
    /// 光标起始位置 / Cursor Start Position
    pub cursor_start: Signal<usize>,
    /// 光标结束位置 / Cursor End Position
    pub cursor_end: Signal<usize>,
    /// 字体大小 / Font Size
    pub font_size: Signal<u32>,
    /// 预览字体大小 / Preview Font Size
    pub preview_font_size: Signal<u32>,
    /// 自动换行 / Word Wrap
    pub word_wrap: Signal<bool>,
    /// 显示行号 / Show Line Numbers
    pub line_numbers: Signal<bool>,
    /// 同步滚动 / Sync Scroll
    pub sync_scroll: Signal<bool>,

    // ========== UI 状态 / UI State ==========
    /// 主题 / Theme
    pub theme: Signal<Theme>,
    /// 语言 / Language
    pub language: Signal<Language>,
    /// 侧边栏可见 / Sidebar Visible
    pub sidebar_visible: Signal<bool>,
    /// 预览可见 / Preview Visible
    pub show_preview: Signal<bool>,
    /// 侧边栏宽度 / Sidebar Width
    pub sidebar_width: Signal<u32>,
    /// 侧边栏标签 (大纲/文件) / Sidebar Tab (Outline/Files)
    pub sidebar_tab: Signal<SidebarTab>,
    /// 编辑器/预览分隔比例 / Editor/Preview Split Ratio
    // ========== 弹窗状态 / Modal State ==========
    pub show_settings: Signal<bool>, // 设置弹窗 / Settings Modal
    pub show_shortcuts: Signal<bool>, // 快捷键弹窗 / Shortcuts Modal
    pub show_ai_chat: Signal<bool>,   // AI 聊天弹窗 / AI Chat Modal
    pub show_ai_result: Signal<bool>, // AI 结果弹窗 / AI Result Modal
    pub show_search: Signal<bool>,    // 搜索弹窗 / Search Modal
    pub show_global_search: Signal<bool>, // 全局搜索弹窗 / Global Search Modal
    pub show_table_editor: Signal<bool>, // 表格编辑器弹窗 / Table Editor Modal

    // ========== 搜索状态 / Search State ==========
    pub search_query: Signal<String>,  // 搜索词 / Search Query
    pub replace_query: Signal<String>, // 替换词 / Replace Query
    pub search_regex: Signal<bool>,    // 正则搜索 / Regex Search
    pub search_case_insensitive: Signal<bool>, // 忽略大小写 / Case Insensitive
    pub search_index: Signal<usize>,   // 当前索引 / Current Index
    pub search_total: Signal<usize>,   // 总结果数 / Total Results

    // ========== 大纲状态 / Outline State ==========
    pub outline_items: Signal<Vec<OutlineItem>>,

    // ========== 文件树状态 / File Tree State ==========
    pub workspace_root: Signal<Option<PathBuf>>, // 工作区根目录 / Workspace Root
    pub file_list: Signal<Vec<PathBuf>>,         // 文件列表 / File List

    // ========== 自动保存状态 / Auto Save State ==========
    /// 自动保存是否启用 / Is Auto Save Enabled
    pub auto_save_enabled: Signal<bool>,
    /// 自动保存间隔（秒）/ Auto Save Interval (seconds)
    pub auto_save_interval: Signal<u32>,

    // ========== 文件监控状态 / File Watch State ==========
    /// 文件是否被外部修改 / Is File Externally Modified
    pub file_external_modified: Signal<bool>,
    /// 文件监控刷新序列号（内部保存/确认后递增）/ File watch refresh sequence
    pub file_watch_refresh_seq: Signal<u64>,

    // ========== 关闭标签确认状态 / Close Tab Confirmation State ==========
    /// 是否显示关闭未保存标签确认弹窗 / Show close unsaved tab confirmation modal
    pub show_close_confirm: Signal<bool>,
    /// 待关闭的标签索引（用户确认后执行）/ Pending close tab index (executed after user confirms)
    pub pending_close_tab_index: Signal<Option<usize>>,

    /// 是否触发另存为对话框（新文件首次保存时）/ Trigger Save-As dialog (first save of new file)
    pub trigger_save_as: Signal<bool>,

    // ========== 性能警告状态 / Performance Warning State ==========
    /// 是否显示大文件警告 / Show Large File Warning
    pub show_large_file_warning: Signal<bool>,
    /// 文件大小 (字节) / File Size (bytes)
    pub file_size_bytes: Signal<usize>,
    /// 待加载的大文件路径（等待用户确认后加载）/ Pending large file path (awaiting user confirmation)
    pub pending_large_file: Signal<Option<PathBuf>>,

    /// 上次大纲更新时间（用于防抖）/ Last outline update time (for debounce)
    last_outline_update: Signal<Option<std::time::Instant>>,

    // ========== AI 状态 / AI State ==========
    pub ai_config: Signal<AIConfig>, // AI 配置 / AI Config
    pub ai_loading: Signal<bool>,    // AI 加载中 / AI Loading
    pub ai_result: Signal<String>,   // AI 结果 / AI Result
    pub ai_title: Signal<String>,    // AI 标题 / AI Title
    pub ai_input: Signal<String>,    // AI 输入 / AI Input
}

/// 侧边栏标签 / Sidebar Tab
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SidebarTab {
    #[default]
    Outline, // 大纲 / Outline
    Files, // 文件 / Files
}

impl AppState {
    /// 创建新的应用状态 / Create New Application State
    pub fn new() -> Self {
        let mut state = Self {
            // 文档状态 / Document State
            current_file: Signal::new(None),
            content: Signal::new(String::new()),
            modified: Signal::new(false),
            history: Signal::new(History::default()),
            save_status: Signal::new(SaveStatus::Saved),
            last_saved: Signal::new(None),

            // 标签页状态 / Tab State
            tabs: Signal::new(Vec::new()),
            current_tab_index: Signal::new(0),

            // 编辑器状态 / Editor State
            cursor_start: Signal::new(0),
            cursor_end: Signal::new(0),
            font_size: Signal::new(16),
            preview_font_size: Signal::new(16),
            word_wrap: Signal::new(false),
            line_numbers: Signal::new(true),
            sync_scroll: Signal::new(true),

            // UI 状态 / UI State
            theme: Signal::new(Theme::Dark),
            language: Signal::new(Language::ZhCN),
            sidebar_visible: Signal::new(true),
            show_preview: Signal::new(true),
            sidebar_width: Signal::new(280),
            sidebar_tab: Signal::new(SidebarTab::Outline),

            // 弹窗状态 / Modal State
            show_settings: Signal::new(false),
            show_shortcuts: Signal::new(false),
            show_ai_chat: Signal::new(false),
            show_ai_result: Signal::new(false),
            show_search: Signal::new(false),
            show_global_search: Signal::new(false),
            show_table_editor: Signal::new(false),

            // 搜索状态 / Search State
            search_query: Signal::new(String::new()),
            replace_query: Signal::new(String::new()),
            search_regex: Signal::new(false),
            search_case_insensitive: Signal::new(true),
            search_index: Signal::new(0),
            search_total: Signal::new(0),

            // 大纲状态 / Outline State
            outline_items: Signal::new(Vec::new()),

            // 文件树状态 / File Tree State
            workspace_root: Signal::new(None),
            file_list: Signal::new(Vec::new()),

            // 自动保存状态 / Auto Save State
            auto_save_enabled: Signal::new(false),
            auto_save_interval: Signal::new(30),

            // 文件监控状态 / File Watch State
            file_external_modified: Signal::new(false),
            file_watch_refresh_seq: Signal::new(0),

            // 关闭标签确认状态 / Close Tab Confirmation State
            show_close_confirm: Signal::new(false),
            pending_close_tab_index: Signal::new(None),

            // 另存为触发 / Save-As Trigger
            trigger_save_as: Signal::new(false),

            // 性能警告状态 / Performance Warning State
            show_large_file_warning: Signal::new(false),
            file_size_bytes: Signal::new(0),
            pending_large_file: Signal::new(None),

            // 防抖状态 / Debounce state
            last_outline_update: Signal::new(None),

            // AI 状态 / AI State
            ai_config: Signal::new(AIConfig::default()),
            ai_loading: Signal::new(false),
            ai_result: Signal::new(String::new()),
            ai_title: Signal::new(String::new()),
            ai_input: Signal::new(String::new()),
        };

        // 确保至少有一个初始标签页（否则 TabBar 调用 init_first_tab 时才能显示）
        // Ensure at least one initial tab exists for proper tab/outline functionality
        state.tabs.write().push(TabInfo::new("未命名"));

        state
    }

    /// 更新内容 / Update Content
    pub fn update_content(&mut self, new_content: String) {
        // 使用哈希检测实际变化，避免重复记录 / Use hash to detect actual changes
        {
            let mut history = self.history.write();
            if !history.is_different(&new_content) {
                return; // 内容没变，不记录 / Content unchanged, skip
            }
        }

        // 保存当前内容到历史 / Save current content to history
        let current = self.content.read().clone();
        {
            let mut history = self.history.write();
            history.push(current);
            history.future.clear();
        }

        *self.content.write() = new_content;
        *self.modified.write() = true;
        *self.save_status.write() = SaveStatus::Unsaved;

        // 更新大纲（带防抖：中等以上文件 500ms 内不重复更新）
        // Update outline (with debounce: skip if <500ms since last update for medium+ files)
        let content_len = self.content.read().len();
        if content_len > 50 * 1024 {
            let now = std::time::Instant::now();
            let should_skip = self.last_outline_update.read().is_some_and(|last| {
                now.duration_since(last) < std::time::Duration::from_millis(500)
            });
            if should_skip {
                return;
            }
        }
        self.update_outline();
    }

    /// 撤销 / Undo
    pub fn undo(&mut self) -> bool {
        // 先获取当前内容，避免嵌套借用 / Get current content first to avoid nested borrowing
        let current = self.content.read().clone();

        let past_content = {
            let mut history = self.history.write();
            history.past.pop_back()
        };

        if let Some(past_content) = past_content {
            // 将当前内容转为 Arc<str> 存入 future
            // Convert current content to Arc<str> and store in future
            self.history
                .write()
                .future
                .push_back(std::sync::Arc::from(current.as_str()));

            // 从 Arc<str> 转回 String 恢复内容 / Convert Arc<str> back to String to restore content
            *self.content.write() = past_content.to_string();
            *self.modified.write() = true;
            // 同步哈希，确保后续 is_different 判断正确
            // Sync hash so subsequent is_different checks work correctly
            self.history.write().is_different(past_content.as_ref());
            self.update_outline();
            return true;
        }
        false
    }

    /// 重做 / Redo
    pub fn redo(&mut self) -> bool {
        // 先获取当前内容，避免嵌套借用 / Get current content first to avoid nested borrowing
        let current = self.content.read().clone();

        let future_content = {
            let mut history = self.history.write();
            history.future.pop_back()
        };

        if let Some(future_content) = future_content {
            // 将当前内容转为 Arc<str> 存入 past
            // Convert current content to Arc<str> and store in past
            self.history
                .write()
                .past
                .push_back(std::sync::Arc::from(current.as_str()));

            // 从 Arc<str> 转回 String 恢复内容 / Convert Arc<str> back to String to restore content
            *self.content.write() = future_content.to_string();
            *self.modified.write() = true;
            // 同步哈希，确保后续 is_different 判断正确
            // Sync hash so subsequent is_different checks work correctly
            self.history.write().is_different(future_content.as_ref());
            self.update_outline();
            return true;
        }
        false
    }

    /// 更新大纲 / Update Outline
    /// 大文件时跳过大纲更新以提升性能 / Skip outline update for large files to improve performance
    pub fn update_outline(&mut self) {
        *self.last_outline_update.write() = Some(std::time::Instant::now());
        let content = self.content.read();
        let content_len = content.len();

        if content_len < 50 * 1024 {
            tracing::debug!(
                "[update_outline] content_len={}, lines={}",
                content_len,
                content.lines().count()
            );
        }

        // 超大文件时限制大纲更新（超过 500KB 只提取前 100 个标题，超过 1MB 跳过）
        // For very large files (>500KB), limit outline; >1MB skip entirely for performance
        let max_items = if content_len > 1024 * 1024 {
            None // 超过 1MB 跳过大纲更新 / Skip outline for >1MB files
        } else if content_len > 500 * 1024 {
            Some(100)
        } else {
            None
        };

        let items: Vec<OutlineItem> = content
            .lines()
            .enumerate()
            .filter_map(|(line_idx, line)| {
                // 匹配 Markdown 标题: # 到 ###### / Match Markdown headings: # to ######
                let trimmed = line.trim_start();
                if trimmed.starts_with('#') {
                    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                    if hash_count <= 6 && hash_count > 0 {
                        let text = trimmed[hash_count..].trim().to_string();
                        if !text.is_empty() {
                            tracing::debug!(
                                "[update_outline] Found heading: level={}, text='{}', line={}",
                                hash_count,
                                text,
                                line_idx
                            );
                            return Some(OutlineItem {
                                level: hash_count as u8,
                                text,
                                line: line_idx,
                            });
                        }
                    }
                }
                None
            })
            .take(max_items.unwrap_or(usize::MAX))
            .collect();

        tracing::debug!("[update_outline] Total headings: {}", items.len());
        *self.outline_items.write() = items;
    }

    /// 标记已保存 / Mark as Saved
    pub fn mark_saved(&mut self) {
        *self.modified.write() = false;
        *self.save_status.write() = SaveStatus::Saved;
        *self.last_saved.write() = Some(std::time::Instant::now());
        self.refresh_file_watch();
    }

    /// 刷新文件监控基线 / Refresh file watch baseline
    pub fn refresh_file_watch(&mut self) {
        let next = *self.file_watch_refresh_seq.read() + 1;
        *self.file_watch_refresh_seq.write() = next;
    }

    /// 获取字符统计 / Get Character Count
    pub fn char_count(&self) -> usize {
        self.content
            .read()
            .chars()
            .filter(|c| !c.is_whitespace())
            .count()
    }

    /// 获取词数统计 (支持中英文混排) / Get Word Count (supports Chinese/English mixed)
    pub fn word_count(&self) -> usize {
        let content = self.content.read();
        // 统计 CJK 字符作为独立"词"/ Count CJK characters as individual "words"
        let cjk_count = content
            .chars()
            .filter(|c| ('\u{4E00}'..='\u{9FFF}').contains(c))
            .count();
        // 移除 CJK 字符后统计英文单词 / Remove CJK chars then count English words
        let non_cjk: String = content
            .chars()
            .filter(|c| !('\u{4E00}'..='\u{9FFF}').contains(c))
            .collect();
        let english_words = non_cjk
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .count();
        cjk_count + english_words
    }

    /// 获取预计阅读时间 (分钟) / Get Estimated Reading Time (minutes)
    pub fn read_time(&self) -> usize {
        let words = self.word_count();
        if words == 0 {
            0
        } else {
            (words / 200).max(1)
        }
    }

    // ========== 多标签页管理 / Multi-Tab Management ==========

    /// 新建标签页 / Create New Tab
    pub fn new_tab(&mut self) {
        // 保存当前标签内容 / Save current tab content
        self.save_current_tab_content();

        // 创建新标签 / Create new tab
        let tab = TabInfo::new(&format!("未命名 {}", self.tabs.read().len() + 1));
        self.tabs.write().push(tab);
        *self.current_tab_index.write() = self.tabs.read().len() - 1;

        // 重置编辑器状态 / Reset editor state
        *self.content.write() = String::new();
        *self.current_file.write() = None;
        *self.modified.write() = false;
        *self.history.write() = History::default();
        self.update_outline();
    }

    /// 打开文件到新标签页 / Open File in New Tab
    pub fn open_file_in_tab(&mut self, path: PathBuf, content: String) {
        tracing::info!("[open_file_in_tab] Attempting to open: {:?}", path);

        // 检查文件是否已打开 / Check if file is already open
        let existing_index = {
            let tabs = self.tabs.read();
            tabs.iter().position(|tab| tab.path.as_ref() == Some(&path))
        };

        if let Some(i) = existing_index {
            tracing::info!("[open_file_in_tab] File already open at tab index {}", i);
            self.switch_to_tab(i);
            return;
        }

        // 保存当前标签内容 / Save current tab content
        self.save_current_tab_content();

        // 创建新标签 / Create new tab
        let tab = TabInfo::from_file(path.clone(), content.clone());
        self.tabs.write().push(tab);
        *self.current_tab_index.write() = self.tabs.read().len() - 1;

        // 更新编辑器状态 / Update editor state
        *self.content.write() = content;
        *self.current_file.write() = Some(path.clone());
        *self.modified.write() = false;
        *self.history.write() = History::default();

        // 自动设置工作区为文件所在目录 / Auto-set workspace to file's parent directory
        if let Some(parent) = path.parent() {
            let should_update = {
                let current_root = self.workspace_root.read();
                current_root.is_none()
                    || current_root
                        .as_ref()
                        .is_none_or(|root| !path.starts_with(root))
            };
            if should_update {
                *self.workspace_root.write() = Some(parent.to_path_buf());
                let files = self.scan_directory(parent);
                tracing::info!("工作区已更新: {:?}，扫描到 {} 个文件 / Workspace updated: {:?}, found {} files", parent, files.len(), parent, files.len());
                *self.file_list.write() = files;
            }
        }

        // 更新大纲 / Update outline
        tracing::info!("[open_file_in_tab] About to call update_outline()");
        self.update_outline();
        tracing::info!(
            "[open_file_in_tab] After update_outline(), outline items: {}",
            self.outline_items.read().len()
        );
    }

    /// 扫描目录中的 Markdown 文件 / Scan Markdown files in directory
    pub(crate) fn scan_directory(&self, dir: &Path) -> Vec<PathBuf> {
        crate::utils::file_utils::scan_markdown_files(dir)
    }

    /// 切换到指定标签页 / Switch to Specified Tab
    pub fn switch_to_tab(&mut self, index: usize) {
        if index >= self.tabs.read().len() {
            return;
        }

        // 保存当前标签内容 / Save current tab content
        self.save_current_tab_content();

        // 切换到新标签 / Switch to new tab
        *self.current_tab_index.write() = index;

        // 获取标签数据（包含历史记录）/ Get tab data (including history)
        let (content, path, modified, history) = {
            let tabs = self.tabs.read();
            let tab = &tabs[index];
            (
                tab.content.clone(),
                tab.path.clone(),
                tab.modified,
                tab.history.clone(),
            )
        };

        // 恢复标签状态（含历史记录）/ Restore tab state (including history)
        *self.content.write() = content;
        *self.current_file.write() = path;
        *self.modified.write() = modified;
        *self.history.write() = history;
        self.update_outline();
    }

    /// 关闭当前标签页 / Close Current Tab
    pub fn close_current_tab(&mut self) -> bool {
        let tabs_len = self.tabs.read().len();

        if tabs_len <= 1 {
            // 只有一个标签时，清空内容但不关闭 / When only one tab, clear content but don't close
            *self.content.write() = String::new();
            *self.current_file.write() = None;
            *self.modified.write() = false;
            *self.history.write() = History::default();

            let mut tabs = self.tabs.write();
            tabs[0].content = String::new();
            tabs[0].path = None;
            tabs[0].modified = false;
            tabs[0].title = "未命名".to_string();
            tabs[0].history = History::default();
            drop(tabs);

            self.update_outline();
            return false;
        }

        // 检查是否已修改 / Check if modified
        if *self.modified.read() {
            *self.pending_close_tab_index.write() = Some(*self.current_tab_index.read());
            *self.show_close_confirm.write() = true;
            return false;
        }

        let current_index = *self.current_tab_index.read();

        // 移除标签 / Remove tab
        self.tabs.write().remove(current_index);

        // 调整当前索引 / Adjust current index
        let new_index = if current_index > 0 {
            current_index - 1
        } else {
            0
        };
        *self.current_tab_index.write() = new_index;

        // 恢复到新当前标签（含历史记录）/ Restore to new current tab (including history)
        let (content, path, modified, history) = {
            let tabs = self.tabs.read();
            let tab = &tabs[new_index];
            (
                tab.content.clone(),
                tab.path.clone(),
                tab.modified,
                tab.history.clone(),
            )
        };

        *self.content.write() = content;
        *self.current_file.write() = path;
        *self.modified.write() = modified;
        *self.history.write() = history;
        self.update_outline();

        true
    }

    /// 关闭指定标签页 / Close Specified Tab
    pub fn close_tab(&mut self, index: usize) -> bool {
        let tabs_len = self.tabs.read().len();
        if index >= tabs_len {
            return false;
        }

        if tabs_len == 1 {
            return self.close_current_tab();
        }

        let current_index = *self.current_tab_index.read();

        let is_current = index == current_index;
        if is_current {
            return self.close_current_tab();
        }

        // 检查目标标签是否已修改 / Check if target tab is modified
        let target_modified = {
            let tabs = self.tabs.read();
            tabs.get(index).map(|tab| tab.modified).unwrap_or(false)
        };

        if target_modified {
            *self.pending_close_tab_index.write() = Some(index);
            *self.show_close_confirm.write() = true;
            return false;
        }

        // 关闭非当前标签 / Close non-current tab
        // 移除标签 / Remove tab
        self.tabs.write().remove(index);

        // 调整当前索引 / Adjust current index
        if index < current_index {
            *self.current_tab_index.write() = current_index - 1;
        }
        true
    }

    /// 保存当前标签内容到 tabs（含历史记录）/ Save Current Tab Content to Tabs (including history)
    fn save_current_tab_content(&mut self) {
        let current_index = *self.current_tab_index.read();
        let tabs_len = self.tabs.read().len();

        if current_index < tabs_len {
            // 先获取所有需要的数据 / First get all required data
            let content = self.content.read().clone();
            let modified = *self.modified.read();
            let path = self.current_file.read().clone();
            let history = self.history.read().clone();

            // 然后写入 / Then write
            let mut tabs = self.tabs.write();
            tabs[current_index].content = content;
            tabs[current_index].modified = modified;
            tabs[current_index].path = path;
            tabs[current_index].history = history;
        }
    }

    /// 初始化第一个标签页 / Initialize First Tab
    pub fn init_first_tab(&mut self) {
        if self.tabs.read().is_empty() {
            let tab = TabInfo::new("未命名");
            self.tabs.write().push(tab);
        }
    }

    // ========== 文本编辑操作 / Text Editing Operations ==========

    /// 在选中文本前后插入格式 / Insert format around selected text
    pub fn insert_format_around_selection(&mut self, prefix: &str, suffix: &str) {
        let content = self.content.read().clone();
        let start = *self.cursor_start.read();
        let end = *self.cursor_end.read();

        // 确保 start <= end / Ensure start <= end
        let (real_start, real_end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        // 获取选中的文本 / Get selected text
        let selected_text = if real_start < content.len() {
            if real_end <= content.len() {
                &content[real_start..real_end]
            } else {
                &content[real_start..]
            }
        } else {
            ""
        };

        // 构建新内容 / Build new content
        let lang = *self.language.read();
        let placeholder = crate::utils::i18n::t("placeholder_text", lang);
        let new_content = format!(
            "{}{}{}{}{}",
            &content[..real_start.min(content.len())],
            prefix,
            if selected_text.is_empty() {
                &placeholder
            } else {
                selected_text
            },
            suffix,
            if real_end < content.len() {
                &content[real_end..]
            } else {
                ""
            }
        );

        // 计算新的光标位置 / Calculate new cursor position
        let placeholder_len = placeholder.len();
        let new_cursor_pos = real_start
            + prefix.len()
            + if selected_text.is_empty() {
                placeholder_len
            } else {
                selected_text.len()
            };

        // 先保存旧内容到历史（在修改 content 之前）/ Save old content to history (before modifying content)
        self.history.write().push(content.clone());
        self.history.write().future.clear();
        // 同步更新哈希，确保后续 update_content 的哈希比对正确
        // Sync hash so subsequent update_content change detection works correctly
        {
            let mut history = self.history.write();
            history.is_different(&new_content);
        }

        *self.content.write() = new_content;
        *self.cursor_start.write() = new_cursor_pos;
        *self.cursor_end.write() = new_cursor_pos;
        *self.modified.write() = true;
        *self.save_status.write() = SaveStatus::Unsaved;

        self.update_outline();
    }

    /// 在行首插入前缀 / Insert prefix at line start
    pub fn insert_line_prefix(&mut self, line_prefix: &str) {
        let content = self.content.read().clone();
        let cursor_pos = *self.cursor_end.read();

        // 找到当前行的开始位置 / Find current line start
        let line_start = content[..cursor_pos.min(content.len())]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        // 在行首插入前缀 / Insert prefix at line start
        let new_content = format!(
            "{}{}{}",
            &content[..line_start],
            line_prefix,
            &content[line_start..]
        );

        let new_cursor_pos = cursor_pos + line_prefix.len();

        // 保存到历史并同步哈希 / Save to history and sync hash
        self.history.write().push(content.clone());
        self.history.write().future.clear();
        {
            let mut history = self.history.write();
            history.is_different(&new_content);
        }

        *self.content.write() = new_content;
        *self.cursor_start.write() = new_cursor_pos;
        *self.cursor_end.write() = new_cursor_pos;
        *self.modified.write() = true;
        *self.save_status.write() = SaveStatus::Unsaved;

        self.update_outline();
    }

    /// 在光标位置插入文本 / Insert text at cursor position
    pub fn insert_at_cursor(&mut self, text: &str) {
        let content = self.content.read().clone();
        let cursor_pos = *self.cursor_end.read();

        let new_content = format!(
            "{}{}{}",
            &content[..cursor_pos.min(content.len())],
            text,
            if cursor_pos < content.len() {
                &content[cursor_pos..]
            } else {
                ""
            }
        );

        let new_cursor_pos = cursor_pos + text.len();

        // 保存到历史并同步哈希 / Save to history and sync hash
        self.history.write().push(content.clone());
        self.history.write().future.clear();
        {
            let mut history = self.history.write();
            history.is_different(&new_content);
        }

        *self.content.write() = new_content;
        *self.cursor_start.write() = new_cursor_pos;
        *self.cursor_end.write() = new_cursor_pos;
        *self.modified.write() = true;
        *self.save_status.write() = SaveStatus::Unsaved;

        self.update_outline();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== History 测试 / History Tests ==========

    #[test]
    fn test_history_default() {
        let history = History::default();
        assert!(history.past.is_empty());
        assert!(history.future.is_empty());
    }

    #[test]
    fn test_history_push() {
        let mut history = History::default();
        history.push("state1".to_string());
        history.push("state2".to_string());
        assert_eq!(history.past.len(), 2);
        assert_eq!(history.past[0].as_ref(), "state1");
        assert_eq!(history.past[1].as_ref(), "state2");
    }

    #[test]
    fn test_history_is_different() {
        let mut history = History::default();
        assert!(history.is_different("hello"));
        assert!(!history.is_different("hello")); // 相同内容 / Same content
        assert!(history.is_different("world"));
        assert!(!history.is_different("world")); // 相同内容 / Same content
    }

    #[test]
    fn test_history_max_size() {
        let mut history = History::default();
        // 插入超过最大容量的记录 / Insert more than max capacity
        for i in 0..(MAX_HISTORY_SIZE + 10) {
            history.push(format!("state_{}", i));
        }
        assert_eq!(history.past.len(), MAX_HISTORY_SIZE);
        // 最早的记录被移除 / Oldest records removed
        assert_eq!(history.past[0].as_ref(), format!("state_{}", 10));
    }

    #[test]
    fn test_history_large_file_memory_optimization() {
        // 测试大文件时历史容量自动减少 / Test that history capacity is reduced for large files
        let mut history = History::default();

        // 创建超过阈值的内容 (100KB) / Create content exceeding threshold
        let large_content: String = "x".repeat(LARGE_FILE_HISTORY_THRESHOLD + 1);

        // 插入大文件历史记录 / Insert large file history records
        for _i in 0..(LARGE_FILE_MAX_HISTORY + 5) {
            history.push(large_content.clone());
        }

        // 大文件时历史容量应该限制为 LARGE_FILE_MAX_HISTORY / History should be limited to LARGE_FILE_MAX_HISTORY
        assert_eq!(history.past.len(), LARGE_FILE_MAX_HISTORY);
        assert!(history.past.len() < MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_history_small_file_full_capacity() {
        // 测试小文件时历史容量保持最大 / Test that small files keep full history capacity
        let mut history = History::default();

        // 创建小于阈值的内容 / Create content below threshold
        let small_content = "small content".to_string();
        assert!(small_content.len() < LARGE_FILE_HISTORY_THRESHOLD);

        // 插入小文件历史记录 / Insert small file history records
        for i in 0..MAX_HISTORY_SIZE {
            history.push(format!("{}_{}", small_content, i));
        }

        // 小文件时历史容量应该保持最大 / History should be at full capacity for small files
        assert_eq!(history.past.len(), MAX_HISTORY_SIZE);
    }

    // ========== TabInfo 测试 / TabInfo Tests ==========

    #[test]
    fn test_tab_info_new() {
        let tab = TabInfo::new("Test Tab");
        assert_eq!(tab.title, "Test Tab");
        assert!(tab.path.is_none());
        assert!(tab.content.is_empty());
        assert!(!tab.modified);
    }

    #[test]
    fn test_tab_info_from_file() {
        let path = PathBuf::from("/docs/test.md");
        let tab = TabInfo::from_file(path.clone(), "# Hello".to_string());
        assert_eq!(tab.title, "test");
        assert_eq!(tab.path, Some(path));
        assert_eq!(tab.content, "# Hello");
        assert!(!tab.modified);
    }

    #[test]
    fn test_tab_info_from_file_no_extension() {
        let path = PathBuf::from("/docs/README");
        let tab = TabInfo::from_file(path, "content".to_string());
        assert_eq!(tab.title, "README");
    }

    // ========== OutlineItem 测试 / OutlineItem Tests ==========

    #[test]
    fn test_outline_parsing() {
        let content = "# Title\n\n## Section 1\n\n### Subsection\n\n## Section 2";
        let items: Vec<OutlineItem> = content
            .lines()
            .enumerate()
            .filter_map(|(line_idx, line)| {
                let trimmed = line.trim_start();
                if trimmed.starts_with('#') {
                    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                    if hash_count <= 6 && hash_count > 0 {
                        let text = trimmed[hash_count..].trim().to_string();
                        if !text.is_empty() {
                            return Some(OutlineItem {
                                level: hash_count as u8,
                                text,
                                line: line_idx,
                            });
                        }
                    }
                }
                None
            })
            .collect();

        assert_eq!(items.len(), 4);
        assert_eq!(items[0].level, 1);
        assert_eq!(items[0].text, "Title");
        assert_eq!(items[1].level, 2);
        assert_eq!(items[1].text, "Section 1");
        assert_eq!(items[2].level, 3);
        assert_eq!(items[2].text, "Subsection");
        assert_eq!(items[3].level, 2);
        assert_eq!(items[3].text, "Section 2");
    }

    // ========== Theme 测试 / Theme Tests ==========

    #[test]
    fn test_theme_default() {
        assert_eq!(Theme::default(), Theme::Dark);
    }

    #[test]
    fn test_language_default() {
        assert_eq!(Language::default(), Language::ZhCN);
    }

    #[test]
    fn test_save_status_default() {
        assert_eq!(SaveStatus::default(), SaveStatus::Saved);
    }

    // ========== AIConfig 测试 / AIConfig Tests ==========

    #[test]
    fn test_ai_config_default() {
        let config = AIConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.provider, AIProvider::OpenAI);
        assert_eq!(config.model, "gpt-4o-mini");
        assert!(config.api_key.is_empty());
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_ai_provider_default() {
        assert_eq!(AIProvider::default(), AIProvider::OpenAI);
    }

    // ========== SidebarTab 测试 / SidebarTab Tests ==========

    #[test]
    fn test_sidebar_tab_default() {
        assert_eq!(SidebarTab::default(), SidebarTab::Outline);
    }

    // ========== TableData 测试 / TableData Tests ==========
    // (TableData 在 TableEditorModal.rs 中定义)

    // ========== update_content → update_outline 集成测试 ==========

    fn with_runtime<F: FnOnce()>(f: F) {
        let vdom = dioxus::prelude::VirtualDom::prebuilt(|| {
            rsx! { div {} }
        });
        let scope_id = dioxus::prelude::ScopeId::ROOT;
        vdom.in_scope(scope_id, f);
    }

    #[test]
    fn test_update_content_updates_outline() {
        with_runtime(|| {
            let mut state = AppState::new();
            assert!(state.outline_items.read().is_empty());

            state
                .update_content("# Title\n\n## Section 1\n\nSome text\n\n## Section 2".to_string());

            let items = state.outline_items.read();
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].level, 1);
            assert_eq!(items[0].text, "Title");
            assert_eq!(items[0].line, 0);
            assert_eq!(items[1].level, 2);
            assert_eq!(items[1].text, "Section 1");
            assert_eq!(items[1].line, 2);
            assert_eq!(items[2].level, 2);
            assert_eq!(items[2].text, "Section 2");
            assert_eq!(items[2].line, 6);
        });
    }

    #[test]
    fn test_update_content_empty_clears_outline() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("# Title\n## Sub".to_string());
            assert_eq!(state.outline_items.read().len(), 2);

            state.update_content(String::new());
            assert!(state.outline_items.read().is_empty());
        });
    }

    #[test]
    fn test_update_content_no_headings() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("Just some text\nwith no headings".to_string());
            assert!(state.outline_items.read().is_empty());
        });
    }

    #[test]
    fn test_update_content_nested_headings() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6".to_string());

            let items = state.outline_items.read();
            assert_eq!(items.len(), 6);
            for i in 0..6 {
                assert_eq!(items[i].level, (i + 1) as u8);
            }
        });
    }

    #[test]
    fn test_update_content_skips_empty_heading() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("# \n## Valid\n###".to_string());

            let items = state.outline_items.read();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].text, "Valid");
        });
    }

    #[test]
    fn test_update_content_dedup_same_content() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("# Title".to_string());
            assert_eq!(state.history.read().past.len(), 1);

            state.update_content("# Title".to_string());
            assert_eq!(state.history.read().past.len(), 1);
        });
    }

    #[test]
    fn test_update_content_tracks_modified() {
        with_runtime(|| {
            let mut state = AppState::new();
            assert!(!*state.modified.read());

            state.update_content("# Changed".to_string());
            assert!(*state.modified.read());
            assert_eq!(*state.save_status.read(), SaveStatus::Unsaved);
        });
    }

    #[test]
    fn test_update_content_undo_redo_outline() {
        with_runtime(|| {
            let mut state = AppState::new();

            state.update_content("# V1".to_string());
            assert_eq!(state.outline_items.read().len(), 1);

            state.update_content("# V2\n## Sub".to_string());
            assert_eq!(state.outline_items.read().len(), 2);

            let undone = state.undo();
            assert!(undone);
            {
                let items = state.outline_items.read();
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].text, "V1");
            }

            let redone = state.redo();
            assert!(redone);
            {
                let items = state.outline_items.read();
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].text, "V2");
                assert_eq!(items[1].text, "Sub");
            }
        });
    }

    #[test]
    fn test_update_content_heading_line_numbers() {
        with_runtime(|| {
            let mut state = AppState::new();
            state.update_content("line0\nline1\n# Title\n\n## Sub\n".to_string());

            let items = state.outline_items.read();
            assert_eq!(items[0].line, 2);
            assert_eq!(items[1].line, 4);
        });
    }
}
