//! AppState 业务方法 / Application state — business logic
//!
//! 与 UI 解耦的操作集中于此，避免单文件过大
//! Keeps behavior out of the struct definition file

use dioxus::prelude::*;
use std::path::{Path, PathBuf};

use super::app_state::AppState;
use super::types::History as DocumentHistory;
use super::types::{OutlineItem, SaveStatus, TabInfo};

impl AppState {
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
        *self.history.write() = DocumentHistory::default();
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
        *self.history.write() = DocumentHistory::default();

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
            *self.history.write() = DocumentHistory::default();

            let mut tabs = self.tabs.write();
            tabs[0].content = String::new();
            tabs[0].path = None;
            tabs[0].modified = false;
            tabs[0].title = "未命名".to_string();
            tabs[0].history = DocumentHistory::default();
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
