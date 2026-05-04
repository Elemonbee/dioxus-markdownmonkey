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

/// File watcher poll interval when a file is open.
pub const FILE_WATCH_ACTIVE_INTERVAL_MS: u64 = 500;

/// File watcher poll interval when no file is open.
pub const FILE_WATCH_IDLE_INTERVAL_SECS: u64 = 5;

/// Grace period for suppressing file watcher notifications after an internal write.
pub const FILE_WATCH_INTERNAL_WRITE_GRACE_MS: u64 = 1500;
