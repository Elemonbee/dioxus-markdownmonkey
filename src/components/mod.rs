//! 组件模块 / Components Module

pub mod ai_chat_modal; // AI 聊天弹窗 / AI Chat Modal
pub mod ai_result_modal; // AI 结果弹窗 / AI Result Modal
pub mod close_confirm_modal; // 关闭未保存确认弹窗 / Close Tab Confirm Modal
pub mod editor; // 编辑器 / Editor
pub mod file_modified_modal; // 文件修改提示弹窗 / File Modified Modal
pub mod file_tree; // 文件树 / File Tree
pub mod global_search_modal; // 全局搜索弹窗 / Global Search Modal
pub mod icons; // SVG 图标 / SVG Icons
pub mod large_file_warning_modal;
pub mod preview; // 预览 / Preview
pub mod search_modal; // 搜索替换弹窗 / Search & Replace Modal
pub mod settings_modal; // 设置弹窗 / Settings Modal
pub mod shortcuts_modal; // 快捷键弹窗 / Shortcuts Modal
pub mod sidebar; // 侧边栏 / Sidebar
pub mod statusbar; // 状态栏 / Status Bar
pub mod tabbar; // 标签栏 / Tab Bar
pub mod table_editor_modal; // 表格编辑器 / Table Editor
pub mod toolbar; // 工具栏 / Toolbar // 大文件警告弹窗 / Large File Warning Modal

// 重新导出 / Re-exports
pub use ai_chat_modal::AiChatModal;
pub use ai_result_modal::AiResultModal;
pub use close_confirm_modal::CloseConfirmModal;
pub use editor::Editor;
pub use file_modified_modal::FileModifiedModal;
pub use global_search_modal::GlobalSearchModal;
pub use large_file_warning_modal::LargeFileWarningModal;
pub use preview::Preview;
pub use search_modal::SearchModal;
pub use settings_modal::SettingsModal;
pub use shortcuts_modal::ShortcutsModal;
pub use sidebar::Sidebar;
pub use statusbar::StatusBar;
pub use tabbar::TabBar;
pub use table_editor_modal::TableEditorModal;
pub use toolbar::Toolbar;
// FileTree 在 sidebar.rs 内部使用，不需要重新导出 / FileTree is used internally in sidebar.rs, no re-export needed
