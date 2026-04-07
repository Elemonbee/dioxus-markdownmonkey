//! Runtime configuration constants shared across the app.

/// Large file warning threshold in bytes.
pub const LARGE_FILE_THRESHOLD_BYTES: usize = 1024 * 1024;

/// Preview render debounce delay in milliseconds.
pub const PREVIEW_DEBOUNCE_MS: u64 = 150;

/// Editor virtual scroll threshold in rendered lines.
pub const EDITOR_VIRTUAL_SCROLL_THRESHOLD_LINES: usize = 500;

/// Editor line height used by virtual scrolling calculations.
pub const EDITOR_LINE_HEIGHT_PX: f32 = 22.4;

/// Extra lines rendered above and below the visible editor viewport.
pub const EDITOR_VIRTUAL_SCROLL_BUFFER_LINES: usize = 10;

/// Sidebar width clamp range.
pub const SIDEBAR_MIN_WIDTH: u32 = 200;
pub const SIDEBAR_MAX_WIDTH: u32 = 400;

/// File watcher poll interval when a file is open.
pub const FILE_WATCH_ACTIVE_INTERVAL_MS: u64 = 500;

/// File watcher poll interval when no file is open.
pub const FILE_WATCH_IDLE_INTERVAL_SECS: u64 = 5;

/// Grace period for suppressing file watcher notifications after an internal write.
pub const FILE_WATCH_INTERNAL_WRITE_GRACE_MS: u64 = 1500;
