//! 应用全局状态容器 / Application global state container
//!
//! 类型定义见 `types.rs`；业务方法见 `app_state_ops.rs`
//! See `types.rs` for data shapes; see `app_state_ops.rs` for behavior

use super::types::{
    AIConfig, Language, OutlineItem, SaveStatus, SidebarTab, TabInfo, Theme,
};
use super::types::History as DocumentHistory;
use dioxus::prelude::*;
use std::path::PathBuf;

/// 应用全局状态 / Application Global State
///
/// 按领域分组字段，便于导航；逻辑实现集中在 `app_state_ops.rs`
/// Fields are grouped by domain for navigation; logic lives in `app_state_ops.rs`
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
    pub history: Signal<DocumentHistory>,
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
    pub(crate) last_outline_update: Signal<Option<std::time::Instant>>,

    // ========== AI 状态 / AI State ==========
    pub ai_config: Signal<AIConfig>, // AI 配置 / AI Config
    pub ai_loading: Signal<bool>,    // AI 加载中 / AI Loading
    pub ai_result: Signal<String>,   // AI 结果 / AI Result
    pub ai_title: Signal<String>,    // AI 标题 / AI Title
    pub ai_input: Signal<String>,    // AI 输入 / AI Input
}

impl AppState {
    /// 创建新的应用状态 / Create New Application State
    pub fn new() -> Self {
        let mut state = Self {
            // 文档状态 / Document State
            current_file: Signal::new(None),
            content: Signal::new(String::new()),
            modified: Signal::new(false),
            history: Signal::new(DocumentHistory::default()),
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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
