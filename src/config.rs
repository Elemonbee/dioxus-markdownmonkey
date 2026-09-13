//! Runtime configuration constants shared across the app.

/// Large file warning threshold in bytes.
pub const LARGE_FILE_THRESHOLD_BYTES: usize = 1024 * 1024;

/// Preview render debounce delay in milliseconds (normal files).
pub const PREVIEW_DEBOUNCE_MS: u64 = 150;
/// Preview render debounce delay for large files (milliseconds).
pub const PREVIEW_LARGE_FILE_DEBOUNCE_MS: u64 = 500;
/// Content size threshold to switch to the longer preview debounce (bytes).
pub const PREVIEW_LARGE_FILE_THRESHOLD_BYTES: usize = 100 * 1024;

/// Editor virtual scroll threshold in rendered lines.
pub const EDITOR_VIRTUAL_SCROLL_THRESHOLD_LINES: usize = 500;

/// Editor line height used by virtual scrolling calculations.
pub const EDITOR_LINE_HEIGHT_PX: f32 = 22.4;

/// Extra lines rendered above and below the visible editor viewport.
pub const EDITOR_VIRTUAL_SCROLL_BUFFER_LINES: usize = 10;

/// Sidebar width clamp range.
pub const SIDEBAR_MIN_WIDTH: u32 = 200;
pub const SIDEBAR_MAX_WIDTH: u32 = 400;
/// Default sidebar width on first launch.
pub const DEFAULT_SIDEBAR_WIDTH: u32 = 280;

/// Font size clamp range.
pub const FONT_SIZE_MIN: u32 = 10;
pub const FONT_SIZE_MAX: u32 = 32;
/// Default editor font size.
pub const DEFAULT_FONT_SIZE: u32 = 16;
/// Default preview pane font size.
pub const DEFAULT_PREVIEW_FONT_SIZE: u32 = 16;

/// Auto-save interval clamp range (seconds).
pub const AUTO_SAVE_INTERVAL_MIN_SECS: u32 = 10;
pub const AUTO_SAVE_INTERVAL_MAX_SECS: u32 = 300;
/// Default auto-save interval (seconds).
pub const DEFAULT_AUTO_SAVE_INTERVAL_SECS: u32 = 30;

/// Auto-save poll interval when active (seconds).
pub const AUTO_SAVE_ACTIVE_POLL_SECS: u64 = 5;
/// Auto-save poll interval when idle (seconds).
pub const AUTO_SAVE_IDLE_POLL_SECS: u64 = 60;

/// Outline update debounce threshold (bytes) — files larger than this get debounced.
pub const OUTLINE_DEBOUNCE_THRESHOLD_BYTES: usize = 50 * 1024;
/// Outline update debounce delay (milliseconds).
pub const OUTLINE_DEBOUNCE_MS: u64 = 500;
/// File size threshold to limit outline to a maximum number of headings (bytes).
pub const OUTLINE_LIMIT_THRESHOLD_BYTES: usize = 500 * 1024;
/// Maximum headings extracted for large files.
pub const OUTLINE_LARGE_FILE_MAX_HEADINGS: usize = 100;

/// File watcher poll interval when a file is open and events are unavailable.
pub const FILE_WATCH_ACTIVE_INTERVAL_MS: u64 = 500;

/// 有事件监视时的 mtime 兜底间隔（秒）
/// mtime fallback interval when filesystem events are available (seconds)
pub const FILE_WATCH_FALLBACK_INTERVAL_SECS: u64 = 5;

/// File watcher poll interval when no file is open.
pub const FILE_WATCH_IDLE_INTERVAL_SECS: u64 = 5;

/// Grace period for suppressing file watcher notifications after an internal write.
pub const FILE_WATCH_INTERNAL_WRITE_GRACE_MS: u64 = 1500;

/// 大文件编辑器改为非受控模式的阈值（字节）
/// Threshold to switch the editor to uncontrolled mode (bytes).
pub const UNCONTROLLED_EDITOR_THRESHOLD_BYTES: usize = 200 * 1024;

/// 非受控编辑器内容同步防抖（毫秒）
/// Debounce for syncing uncontrolled editor content to Rust (ms).
pub const UNCONTROLLED_EDITOR_SYNC_DEBOUNCE_MS: u64 = 350;

/// 非活动标签在内存中保留的最大数量（其余未修改已保存标签可驱逐）
/// Max resident inactive tabs kept in memory (others may be evicted if saved).
pub const MAX_RESIDENT_INACTIVE_TABS: usize = 3;

/// 工作区搜索最大目录深度 / Max workspace-search directory depth
pub const WORKSPACE_SEARCH_MAX_DEPTH: usize = 10;
/// 工作区搜索最多扫描的文件数 / Max files scanned during workspace search
pub const WORKSPACE_SEARCH_MAX_FILES: usize = 1000;
/// 工作区搜索最多返回的命中数 / Max workspace-search hits returned
pub const WORKSPACE_SEARCH_MAX_RESULTS: usize = 200;
/// 工作区搜索跳过大于此尺寸的文件（字节）/ Skip files larger than this during workspace search
pub const WORKSPACE_SEARCH_MAX_FILE_BYTES: u64 = 1024 * 1024;

/// 跟随系统主题时的轮询间隔（秒）/ Poll interval when following the OS theme
pub const SYSTEM_THEME_POLL_SECS: u64 = 2;

/// 预览代码块超过此尺寸则跳过 syntect（字节）
/// Skip syntect highlighting for preview code blocks larger than this (bytes)
pub const CODE_HIGHLIGHT_MAX_BYTES: usize = 64 * 1024;

/// Mermaid 源码超过此尺寸则只转义、不交给图表引擎（字节）
/// Skip Mermaid rendering for diagrams larger than this (bytes)
pub const MERMAID_MAX_BYTES: usize = 64 * 1024;
