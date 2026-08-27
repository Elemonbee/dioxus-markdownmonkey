//! 文件操作 Actions / File Operation Actions
//!
//! 处理文件打开、保存、工作区等操作

use crate::config::LARGE_FILE_THRESHOLD_BYTES;
use crate::state::{AppState, CloseTabSnapshot, SaveStatus, TabId};
use crate::utils::file_encoding::{self, FileEncoding};
use crate::utils::file_utils;
use dioxus::prelude::{ReadableExt, WritableExt};
use rfd::AsyncFileDialog;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// 文件打开对话框是否正在显示 / Whether the open-file dialog is currently shown
static OPEN_DIALOG_ACTIVE: AtomicBool = AtomicBool::new(false);

/// 文件打开对话框防重入守卫 / Re-entrancy guard for the open-file dialog
pub(crate) struct OpenDialogGuard;

impl Drop for OpenDialogGuard {
    /// 释放打开对话框占用状态 / Release the open-dialog active state
    fn drop(&mut self) {
        OPEN_DIALOG_ACTIVE.store(false, Ordering::Release);
    }
}

/// 尝试获取文件打开对话框守卫 / Try to acquire the open-file dialog guard
pub(crate) fn try_begin_open_dialog() -> Option<OpenDialogGuard> {
    OPEN_DIALOG_ACTIVE
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .ok()
        .map(|_| OpenDialogGuard)
}

/// 文件 Actions 处理器 / File Actions Handler
pub struct FileActions;

impl FileActions {
    /// 显示共享异步打开对话框，并复用统一的 flush、编码、大文件和最近文件流程
    /// Show the shared async open dialog and reuse flush, encoding, large-file, and recent-file handling
    pub async fn open_file_dialog(state: &mut AppState) -> Result<(), String> {
        let Some(_guard) = try_begin_open_dialog() else {
            return Ok(());
        };
        let file = AsyncFileDialog::new()
            .add_filter("Markdown", &["md", "markdown", "txt"])
            .pick_file()
            .await;
        if let Some(file) = file {
            Self::open_file_and_track_recent_flushed(state, file.path().to_path_buf()).await?;
        }
        Ok(())
    }

    /// 打开文件 / Open File
    /// 大文件（>1MB）不会立即读入内存，而是等待用户确认后再加载
    /// Large files (>1MB) are not loaded into memory immediately; they wait for user confirmation
    pub fn open_file(state: &mut AppState, path: PathBuf) -> Result<(), String> {
        // 检查文件大小 / Check file size
        let metadata = fs::metadata(&path).map_err(|e| format!("无法读取文件信息: {}", e))?;
        let file_size = metadata.len() as usize;

        if file_size > LARGE_FILE_THRESHOLD_BYTES {
            // 大文件：暂存路径，显示警告，等待用户确认后再读取
            // Large file: store path, show warning, wait for user confirmation before reading
            tracing::info!(
                "[FileActions::open_file] Large file detected: {:?} ({} bytes), awaiting user confirmation",
                path, file_size
            );
            let mut doc = state.document();
            *doc.file_size_bytes.write() = file_size;
            *doc.pending_large_file.write() = Some(path);
            *doc.show_large_file_warning.write() = true;
            return Ok(());
        }

        // 使用编码检测读取文件 / Read file with encoding detection
        let (content, encoding) = Self::read_file_with_encoding(&path)?;

        // 记录打开文件前的状态 / Log state before opening file
        tracing::info!(
            "[FileActions::open_file] path={:?}, content_len={}, encoding={}",
            path,
            content.len(),
            encoding
        );

        state.open_file_in_tab_with_encoding(path, content, encoding);
        Ok(())
    }

    /// 打开文件并写入最近文件列表 / Open file and update recent-files list
    pub fn open_file_and_track_recent(state: &mut AppState, path: PathBuf) -> Result<(), String> {
        Self::open_file(state, path.clone())?;
        let mut recent = crate::services::recent_files::RecentFiles::load();
        recent.add(path);
        let _ = recent.save();
        Ok(())
    }

    /// 先 flush 非受控编辑器再打开文件 / Flush uncontrolled editor then open file
    pub async fn open_file_flushed(state: &mut AppState, path: PathBuf) -> Result<(), String> {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::open_file(state, path)
    }

    /// 先 flush 再打开并记录最近文件 / Flush then open and track recent
    pub async fn open_file_and_track_recent_flushed(
        state: &mut AppState,
        path: PathBuf,
    ) -> Result<(), String> {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::open_file_and_track_recent(state, path)
    }

    /// 处理编辑器拖放的 Markdown/文本路径（图片仍由 JS 处理）
    /// Handle editor drops of markdown/text paths (images remain JS-handled)
    pub fn handle_editor_file_drop(state: &mut AppState, paths: Vec<PathBuf>) {
        for path in paths {
            if path.as_os_str().is_empty() || !path.exists() {
                continue;
            }
            if file_utils::is_markdown_or_text_path(&path) {
                if let Err(e) = Self::open_file_and_track_recent(state, path) {
                    tracing::warn!("Drop open failed: {}", e);
                }
            }
        }
    }

    /// 先 flush 再处理拖放打开 / Flush then handle drop-open
    pub async fn handle_editor_file_drop_flushed(state: &mut AppState, paths: Vec<PathBuf>) {
        if paths.is_empty() {
            return;
        }
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::handle_editor_file_drop(state, paths);
    }

    /// 确认加载大文件 / Confirm loading large file
    /// 由 LargeFileWarningModal 的"继续编辑"按钮调用
    /// Called by the "Continue" button in LargeFileWarningModal
    pub fn confirm_load_large_file(state: &mut AppState) -> Result<(), String> {
        let mut doc = state.document();
        let pending_path = doc.pending_large_file.read().clone();

        match pending_path {
            Some(path) => {
                // 清除待加载状态 / Clear pending state
                *doc.pending_large_file.write() = None;
                *doc.show_large_file_warning.write() = false;

                // 读取文件内容 / Read file content
                let (content, encoding) = Self::read_file_with_encoding(&path)?;
                tracing::info!(
                    "[FileActions::confirm_load_large_file] Loading large file: {:?}, content_len={}, encoding={}",
                    path, content.len(), encoding
                );

                let _ = doc;
                state.open_file_in_tab_with_encoding(path, content, encoding);
                Ok(())
            }
            None => Err("没有待加载的大文件 / No pending large file".to_string()),
        }
    }

    /// 先 flush 再确认加载大文件 / Flush then confirm loading large file
    pub async fn confirm_load_large_file_flushed(state: &mut AppState) -> Result<(), String> {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::confirm_load_large_file(state)
    }

    /// 取消加载大文件 / Cancel loading large file
    /// 由 LargeFileWarningModal 的"取消"按钮调用
    /// Called by the "Cancel" button in LargeFileWarningModal
    pub fn cancel_load_large_file(state: &mut AppState) {
        let mut doc = state.document();
        *doc.pending_large_file.write() = None;
        *doc.show_large_file_warning.write() = false;
        *doc.file_size_bytes.write() = 0;
    }

    /// 带类型安全编码检测的文件读取 / Read a file with type-safe encoding detection
    pub(crate) fn read_file_with_encoding(path: &Path) -> Result<(String, FileEncoding), String> {
        file_encoding::read_file(path)
    }

    /// 保存当前文件 / Save Current File
    /// 如果文件没有路径（新建文件），触发另存为流程
    /// If file has no path (new file), triggers Save-As flow
    /// UI 路径请用 `save_current_file_async`；同步版供测试与非 UI 调用
    /// Prefer `save_current_file_async` in UI; sync version for tests / non-UI
    #[allow(dead_code)]
    pub fn save_current_file(state: &mut AppState) -> Result<(), String> {
        let mut doc = state.document();
        let current_file = doc.current_file.read().clone();
        let content = doc.content.read().clone();
        let encoding = *doc.file_encoding.read();
        let revision = *doc.content_revision.read();

        match current_file {
            Some(path) => {
                *doc.save_status.write() = SaveStatus::Saving;
                if let Err(error) = file_encoding::write_file(&path, &content, encoding) {
                    *doc.save_status.write() = SaveStatus::Unsaved;
                    return Err(error);
                }
                // 仅当内容未在写入期间变更时标记已保存 / Mark saved only if content did not change during write
                if *doc.content_revision.read() == revision {
                    state.mark_saved();
                } else {
                    *doc.modified.write() = true;
                    *doc.save_status.write() = SaveStatus::Unsaved;
                    tracing::warn!(
                        "保存完成但内容已变更，保持未保存状态 / Save finished but content changed; keep unsaved"
                    );
                }
                Ok(())
            }
            None => {
                *doc.trigger_save_as.write() = true;
                Err("没有文件路径，请选择保存位置".to_string())
            }
        }
    }

    /// 异步保存当前文件（阻塞 I/O 放到线程池，避免卡住 UI）
    /// Async save current file (blocking I/O on thread pool to avoid freezing UI)
    pub async fn save_current_file_async(state: &mut AppState) -> Result<(), String> {
        let mut doc = state.document();
        let current_file = doc.current_file.read().clone();
        let content = doc.content.read().clone();
        let encoding = *doc.file_encoding.read();
        let revision = *doc.content_revision.read();

        match current_file {
            Some(path) => {
                *doc.save_status.write() = SaveStatus::Saving;
                let write_path = path.clone();
                let write_content = content.clone();
                let write_result = tokio::task::spawn_blocking(move || {
                    file_encoding::write_file(&write_path, &write_content, encoding)
                })
                .await
                .map_err(|e| format!("保存任务失败 / Save task failed: {}", e))?;
                if let Err(error) = write_result {
                    *doc.save_status.write() = SaveStatus::Unsaved;
                    return Err(error);
                }

                if *doc.content_revision.read() == revision {
                    state.mark_saved();
                    Ok(())
                } else {
                    // 写入过期内容后立即用最新内容再写一次 / Rewrite with latest content after stale write
                    let latest = doc.content.read().clone();
                    let latest_rev = *doc.content_revision.read();
                    let rewrite_path = path;
                    let rewrite_result = tokio::task::spawn_blocking(move || {
                        file_encoding::write_file(&rewrite_path, &latest, encoding)
                    })
                    .await
                    .map_err(|e| format!("保存任务失败 / Save task failed: {}", e))?;
                    if let Err(error) = rewrite_result {
                        *doc.modified.write() = true;
                        *doc.save_status.write() = SaveStatus::Unsaved;
                        return Err(error);
                    }
                    if *doc.content_revision.read() == latest_rev {
                        state.mark_saved();
                    } else {
                        *doc.modified.write() = true;
                        *doc.save_status.write() = SaveStatus::Unsaved;
                    }
                    Ok(())
                }
            }
            None => {
                *doc.trigger_save_as.write() = true;
                Err("没有文件路径，请选择保存位置".to_string())
            }
        }
    }

    /// 从磁盘重新加载当前文件（外部修改后）
    /// Reload current file from disk after external modification
    pub fn reload_current_file(state: &mut AppState) -> Result<(), String> {
        let tab_id = {
            let doc = state.document();
            let index = *doc.current_tab_index.read();
            let result = doc
                .tabs
                .read()
                .get(index)
                .map(|tab| tab.id)
                .ok_or_else(|| "没有打开的标签 / No open tab".to_string())?;
            result
        };
        Self::reload_tab_from_disk(state, tab_id)
    }

    /// 按稳定标签标识从磁盘重载，并交由统一快照入口应用
    /// Reload from disk by stable tab identity and apply through the unified snapshot entry
    pub fn reload_tab_from_disk(state: &mut AppState, tab_id: TabId) -> Result<(), String> {
        let path = {
            let doc = state.document();
            let result = doc
                .tabs
                .read()
                .iter()
                .find(|tab| tab.id == tab_id)
                .and_then(|tab| tab.path.clone())
                .ok_or_else(|| "标签没有文件路径 / Tab has no file path".to_string())?;
            result
        };
        let (content, encoding) = Self::read_file_with_encoding(&path)?;
        state.apply_disk_snapshot(tab_id, path, content, encoding)
    }

    /// 先 flush，再从磁盘重载并推回 DOM
    /// Flush then reload from disk and push content into the DOM
    pub async fn reload_current_file_flushed(state: &mut AppState) -> Result<(), String> {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::reload_current_file(state)?;
        crate::actions::EditorActions::push_to_dom(&state.document().content.read());
        Ok(())
    }

    /// 读取目标标签的实时修改状态 / Read the target tab's live modified state
    pub(crate) fn target_tab_modified(state: &AppState, index: usize) -> bool {
        let doc = state.document();
        let current = *doc.current_tab_index.read();
        if index == current {
            *doc.modified.read()
        } else {
            doc.tabs
                .read()
                .get(index)
                .map(|tab| tab.modified)
                .unwrap_or(false)
        }
    }

    /// 按稳定标识查找标签当前索引 / Find a tab's current index by stable identity
    pub(crate) fn tab_index_by_id(state: &AppState, tab_id: TabId) -> Option<usize> {
        state
            .document()
            .tabs
            .read()
            .iter()
            .position(|tab| tab.id == tab_id)
    }

    /// 捕获目标标签正文、路径和编码快照 / Capture target tab body, path, and encoding
    pub(crate) fn close_tab_snapshot(state: &AppState, tab_id: TabId) -> Option<CloseTabSnapshot> {
        let doc = state.document();
        let index = Self::tab_index_by_id(state, tab_id)?;
        let current = *doc.current_tab_index.read();
        if index == current {
            Some(CloseTabSnapshot {
                tab_id,
                path: doc.current_file.read().clone(),
                content: doc.content.read().clone(),
                encoding: *doc.file_encoding.read(),
            })
        } else {
            let tabs = doc.tabs.read();
            let tab = tabs.get(index)?;
            Some(CloseTabSnapshot {
                tab_id,
                path: tab.path.clone(),
                content: tab.content.as_ref()?.to_string(),
                encoding: tab.encoding,
            })
        }
    }

    /// 清除关闭确认与异步另存为状态 / Clear close confirmation and async Save-As state
    pub fn cancel_close_request(state: &mut AppState) {
        let mut doc = state.document();
        *doc.show_close_confirm.write() = false;
        *doc.pending_close_tab_id.write() = None;
        *doc.pending_close_save_as.write() = None;
    }

    /// 按是否已修改决定确认关闭或直接关闭（同步，供测试与 flush 后调用）
    /// Confirm-close if modified, else close immediately (sync; for tests and post-flush)
    #[allow(dead_code)] // 同步测试入口 / Synchronous test entry point
    pub fn request_close_tab(state: &mut AppState, index: usize) {
        let tab_id = state.document().tabs.read().get(index).map(|tab| tab.id);
        if let Some(tab_id) = tab_id {
            Self::request_close_tab_by_id(state, tab_id);
        }
    }

    /// 按稳定标识请求关闭目标标签 / Request target-tab close by stable identity
    fn request_close_tab_by_id(state: &mut AppState, tab_id: TabId) {
        let Some(index) = Self::tab_index_by_id(state, tab_id) else {
            return;
        };
        state.save_current_tab_content();
        if Self::target_tab_modified(state, index) {
            let mut doc = state.document();
            *doc.pending_close_tab_id.write() = Some(tab_id);
            *doc.show_close_confirm.write() = true;
        } else {
            state.close_tab(index);
        }
    }

    /// 先 flush，再按是否已修改决定确认关闭或直接关闭
    /// Flush, then confirm-close if modified or close immediately
    pub async fn request_close_tab_flushed(state: &mut AppState, index: usize) {
        let tab_id = state.document().tabs.read().get(index).map(|tab| tab.id);
        crate::actions::EditorActions::flush_from_dom(state).await;
        if let Some(tab_id) = tab_id {
            Self::request_close_tab_by_id(state, tab_id);
        }
    }

    /// 丢弃目标标签修改并按稳定标识关闭 / Discard target edits and close by stable identity
    pub fn discard_and_close_tab_by_id(state: &mut AppState, tab_id: TabId) {
        let Some(index) = Self::tab_index_by_id(state, tab_id) else {
            Self::cancel_close_request(state);
            return;
        };
        let current = *state.document().current_tab_index.read();
        {
            let mut doc = state.document();
            if index == current {
                *doc.modified.write() = false;
            }
            let mut tabs = doc.tabs.write();
            if let Some(tab) = tabs.get_mut(index) {
                tab.modified = false;
            }
        }
        Self::cancel_close_request(state);
        state.close_tab(index);
    }

    /// 丢弃指定索引标签的修改并关闭 / Discard edits of an indexed tab and close it
    #[allow(dead_code)]
    pub fn discard_and_close_tab(state: &mut AppState, index: usize) {
        let tab_id = state.document().tabs.read().get(index).map(|tab| tab.id);
        if let Some(tab_id) = tab_id {
            Self::discard_and_close_tab_by_id(state, tab_id);
        } else {
            Self::cancel_close_request(state);
        }
    }

    /// 先 flush，再按稳定标识丢弃修改并关闭 / Flush, then discard and close by stable identity
    pub async fn discard_and_close_tab_flushed(state: &mut AppState, tab_id: TabId) {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::discard_and_close_tab_by_id(state, tab_id);
    }

    /// 判断目标标签是否仍与确认关闭时的快照一致
    /// Check whether the target tab still matches its close-confirmation snapshot
    fn close_snapshot_is_current(state: &AppState, snapshot: &CloseTabSnapshot) -> bool {
        let Some(index) = Self::tab_index_by_id(state, snapshot.tab_id) else {
            return false;
        };
        let doc = state.document();
        if index == *doc.current_tab_index.read() {
            *doc.current_file.read() == snapshot.path
                && *doc.file_encoding.read() == snapshot.encoding
                && *doc.content.read() == snapshot.content
        } else {
            doc.tabs.read().get(index).is_some_and(|tab| {
                tab.path == snapshot.path
                    && tab.encoding == snapshot.encoding
                    && tab.content_str() == snapshot.content
            })
        }
    }

    /// 在快照成功写盘后安全关闭对应标签 / Safely close the matching tab after snapshot write
    fn close_written_snapshot(state: &mut AppState, snapshot: &CloseTabSnapshot) -> bool {
        if !Self::close_snapshot_is_current(state, snapshot) {
            Self::cancel_close_request(state);
            return false;
        }
        let Some(index) = Self::tab_index_by_id(state, snapshot.tab_id) else {
            Self::cancel_close_request(state);
            return false;
        };
        let current = *state.document().current_tab_index.read();
        {
            let mut doc = state.document();
            if index == current {
                *doc.modified.write() = false;
                *doc.save_status.write() = SaveStatus::Saved;
            }
            let mut tabs = doc.tabs.write();
            if let Some(tab) = tabs.get_mut(index) {
                tab.modified = false;
            }
        }
        Self::cancel_close_request(state);
        state.close_tab(index);
        true
    }

    /// 保存有路径标签的确认快照并在成功后关闭 / Save a pathed tab snapshot and close on success
    pub(crate) async fn save_pathed_snapshot_and_close(
        state: &mut AppState,
        snapshot: CloseTabSnapshot,
        path: PathBuf,
    ) -> Result<(), String> {
        if Self::tab_index_by_id(state, snapshot.tab_id)
            == Some(*state.document().current_tab_index.read())
        {
            *state.document().save_status.write() = SaveStatus::Saving;
        }
        let content = snapshot.content.clone();
        let encoding = snapshot.encoding;
        let write_path = path;
        let write_result = tokio::task::spawn_blocking(move || {
            file_encoding::write_file(&write_path, &content, encoding)
        })
        .await
        .map_err(|e| format!("保存任务失败 / Save task failed: {}", e))?;
        if let Err(error) = write_result {
            if Self::tab_index_by_id(state, snapshot.tab_id)
                == Some(*state.document().current_tab_index.read())
            {
                *state.document().save_status.write() = SaveStatus::Unsaved;
            }
            return Err(error);
        }

        if Self::close_written_snapshot(state, &snapshot) {
            Ok(())
        } else {
            Err("标签在保存期间已变更，已保留 / Tab changed while saving and was kept".to_string())
        }
    }

    /// 将无路径关闭快照排入异步另存为流程 / Queue an untitled close snapshot for async Save As
    pub(crate) fn queue_close_save_as(state: &mut AppState, snapshot: CloseTabSnapshot) {
        let mut doc = state.document();
        *doc.show_close_confirm.write() = false;
        *doc.pending_close_save_as.write() = Some(snapshot);
        *doc.trigger_save_as.write() = true;
    }

    /// 先 flush，再按稳定标识保存并关闭；无路径时发出异步另存为意图
    /// Flush, save, and close by stable identity; untitled tabs emit async Save-As intent
    pub async fn save_and_close_tab_flushed(state: &mut AppState, tab_id: TabId) {
        crate::actions::EditorActions::flush_from_dom(state).await;
        state.save_current_tab_content();
        let Some(snapshot) = Self::close_tab_snapshot(state, tab_id) else {
            Self::cancel_close_request(state);
            return;
        };

        if let Some(path) = snapshot.path.clone() {
            if let Err(error) = Self::save_pathed_snapshot_and_close(state, snapshot, path).await {
                tracing::error!("Save and close failed: {}", error);
            }
        } else {
            Self::queue_close_save_as(state, snapshot);
        }
    }

    /// 用选定路径保存关闭快照，成功时仅关闭对应稳定标签
    /// Save a close snapshot to the selected path and close only that stable tab on success
    pub fn save_close_snapshot_as(
        state: &mut AppState,
        snapshot: CloseTabSnapshot,
        path: PathBuf,
    ) -> Result<(), String> {
        let path = if path.extension().is_none() {
            let mut path = path;
            path.set_extension("md");
            path
        } else {
            path
        };
        file_encoding::write_file(&path, &snapshot.content, snapshot.encoding)?;
        if !Self::close_snapshot_is_current(state, &snapshot) {
            Self::cancel_close_request(state);
            return Err(
                "标签在另存为期间已变更，已保留 / Tab changed during Save As and was kept"
                    .to_string(),
            );
        }
        Self::close_written_snapshot(state, &snapshot);
        Ok(())
    }

    /// 另存为 / Save As
    pub fn save_as(state: &mut AppState, path: PathBuf) -> Result<(), String> {
        let mut doc = state.document();
        let content = doc.content.read().clone();
        let encoding = *doc.file_encoding.read();
        let index = *doc.current_tab_index.read();
        let tab_id = doc
            .tabs
            .read()
            .get(index)
            .map(|tab| tab.id)
            .ok_or_else(|| "没有打开的标签 / No open tab".to_string())?;

        // 确保文件扩展名
        let path = if path.extension().is_none() {
            let mut p = path;
            p.set_extension("md");
            p
        } else {
            path
        };

        *doc.save_status.write() = SaveStatus::Saving;
        if let Err(error) = file_encoding::write_file(&path, &content, encoding) {
            *doc.save_status.write() = SaveStatus::Unsaved;
            return Err(error);
        }

        let old_key = state.current_ai_session_key();
        let new_key = crate::state::ai_session_key_for_path(&path);
        let _ = doc;
        state.apply_disk_snapshot(tab_id, path, content, encoding)?;
        {
            let mut doc = state.document();
            let mut tabs = doc.tabs.write();
            if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == tab_id) {
                tab.ai_session_key = new_key.clone();
            }
        }
        if old_key != new_key {
            state.persist_ai_history_key(&new_key);
        }
        Ok(())
    }

    /// 工作区字面全部替换（大小写不敏感），同步打开标签缓冲区并写回磁盘
    /// Literal case-insensitive replace-all in workspace; sync open tabs and write disk
    pub fn replace_in_workspace(
        state: &mut AppState,
        query: &str,
        replacement: &str,
    ) -> crate::utils::workspace_search::WorkspaceReplaceReport {
        use crate::utils::replace::{count_matches, replace_all_in_text};
        use crate::utils::workspace_search::{
            apply_workspace_replace, collect_open_buffer_overrides, collect_workspace_files,
            WorkspaceReplaceReport,
        };

        let mut report = WorkspaceReplaceReport::default();
        if query.is_empty() {
            return report;
        }

        let workspace = state.ui().workspace_root.read().clone();
        let Some(root) = workspace else {
            let content = state.document().content.read().clone();
            let n = count_matches(&content, query, true, false);
            if n == 0 {
                return report;
            }
            let next = replace_all_in_text(&content, query, replacement, true, false);
            state.update_content(next);
            crate::actions::EditorActions::push_to_dom(&state.document().content.read());
            report.files_touched = 1;
            report.matches_total = n;
            return report;
        };

        let overrides = collect_open_buffer_overrides(state);
        let files = collect_workspace_files(&root, &overrides);
        let (changed, apply_report) = apply_workspace_replace(&files, query, replacement);
        report = apply_report;

        for (path, new_content) in changed {
            let open_idx = {
                let doc = state.document();
                let tabs = doc.tabs.read();
                tabs.iter().position(|t| t.path.as_ref() == Some(&path))
            };

            if let Err(e) = fs::write(&path, &new_content) {
                report.errors.push(format!("{}: {}", path.display(), e));
                continue;
            }

            if let Some(idx) = open_idx {
                let current = *state.document().current_tab_index.read();
                if idx == current {
                    state.update_content(new_content.clone());
                    crate::actions::EditorActions::push_to_dom(&new_content);
                    state.mark_saved();
                } else {
                    let mut doc = state.document();
                    let mut tabs = doc.tabs.write();
                    if let Some(tab) = tabs.get_mut(idx) {
                        tab.set_content_arc(std::sync::Arc::<str>::from(new_content.as_str()));
                        tab.modified = false;
                        tab.history.reset_with_content(&new_content);
                    }
                }
            }
        }

        report
    }

    /// 设置工作区 / Set Workspace
    pub fn set_workspace(state: &mut AppState, path: PathBuf) {
        let mut ui = state.ui();
        *ui.workspace_root.write() = Some(path.clone());
        let files = file_utils::scan_markdown_files(&path);
        *ui.file_list.write() = files;
    }

    /// 创建新文件 / Create New File
    pub fn create_new_file(
        state: &mut AppState,
        dir: &Path,
        base_name: &str,
    ) -> Result<PathBuf, String> {
        let mut path = dir.join(base_name);
        let mut counter = 1;
        let base_without_ext = base_name.trim_end_matches(".md");

        while path.exists() {
            path = dir.join(format!("{}_{}.md", base_without_ext, counter));
            counter += 1;
        }

        fs::write(&path, "").map_err(|e| format!("无法创建文件: {}", e))?;

        // 刷新文件列表
        let mut ui = state.ui();
        let workspace = ui.workspace_root.read().clone();
        if let Some(workspace) = workspace {
            let files = file_utils::scan_markdown_files(&workspace);
            *ui.file_list.write() = files;
        }

        Ok(path)
    }

    /// 创建新文件夹 / Create New Folder
    pub fn create_new_folder(
        state: &mut AppState,
        dir: &Path,
        base_name: &str,
    ) -> Result<PathBuf, String> {
        let mut path = dir.join(base_name);
        let mut counter = 1;

        while path.exists() {
            path = dir.join(format!("{}_{}", base_name, counter));
            counter += 1;
        }

        fs::create_dir(&path).map_err(|e| format!("无法创建文件夹: {}", e))?;

        // 刷新文件列表 / Refresh file list
        let mut ui = state.ui();
        let workspace = ui.workspace_root.read().clone();
        if let Some(workspace) = workspace {
            let files = file_utils::scan_markdown_files(&workspace);
            *ui.file_list.write() = files;
        }

        Ok(path)
    }

    /// 刷新工作区 / Refresh Workspace
    pub fn refresh_workspace(state: &mut AppState) {
        let mut ui = state.ui();
        let workspace = ui.workspace_root.read().clone();
        if let Some(workspace) = workspace {
            let files = file_utils::scan_markdown_files(&workspace);
            *ui.file_list.write() = files;
        }
    }

    /// 判断路径是否等于目标或位于目标目录之下 / Check whether a path equals or descends from a target
    fn path_is_affected(path: &Path, target: &Path) -> bool {
        path == target || path.starts_with(target)
    }

    /// 将路径的旧前缀替换为新前缀 / Replace an old path prefix with a new prefix
    fn replace_path_prefix(path: &Path, old_prefix: &Path, new_prefix: &Path) -> PathBuf {
        path.strip_prefix(old_prefix)
            .map(|suffix| new_prefix.join(suffix))
            .unwrap_or_else(|_| path.to_path_buf())
    }

    /// 从文件路径生成标签标题 / Build a tab title from a file path
    fn tab_title_for_path(path: &Path) -> String {
        path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }

    /// 删除文件 / Delete File
    pub fn delete_file(state: &mut AppState, path: &Path) -> Result<(), String> {
        let workspace = state.ui().workspace_root.read().clone();
        if let Some(ref workspace) = workspace {
            file_utils::ensure_within_workspace(path, workspace)?;
        }

        state.save_current_tab_content();
        let evicted_tabs: Vec<(TabId, PathBuf)> = state
            .document()
            .tabs
            .read()
            .iter()
            .filter_map(|tab| {
                let tab_path = tab.path.as_ref()?;
                (Self::path_is_affected(tab_path, path) && tab.content.is_none())
                    .then(|| (tab.id, tab_path.clone()))
            })
            .collect();
        let mut hydrated = Vec::with_capacity(evicted_tabs.len());
        for (tab_id, tab_path) in evicted_tabs {
            let (content, encoding) = Self::read_file_with_encoding(&tab_path)?;
            hydrated.push((tab_id, content, encoding));
        }

        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| format!("无法删除文件夹: {}", e))?;
        } else {
            fs::remove_file(path).map_err(|e| format!("无法删除文件: {}", e))?;
        }

        let current_affected = state
            .document()
            .current_file
            .read()
            .as_ref()
            .is_some_and(|current| Self::path_is_affected(current, path));
        {
            let mut doc = state.document();
            let mut tabs = doc.tabs.write();
            for tab in tabs.iter_mut() {
                let affected = tab
                    .path
                    .as_ref()
                    .is_some_and(|tab_path| Self::path_is_affected(tab_path, path));
                if !affected {
                    continue;
                }
                if tab.content.is_none() {
                    if let Some((_, content, encoding)) =
                        hydrated.iter().find(|(tab_id, _, _)| *tab_id == tab.id)
                    {
                        tab.set_content_arc(std::sync::Arc::from(content.as_str()));
                        tab.encoding = *encoding;
                        tab.history.reset_with_content(content);
                    }
                }
                tab.path = None;
                tab.modified = true;
            }
        }

        if current_affected {
            let mut doc = state.document();
            *doc.current_file.write() = None;
            *doc.modified.write() = true;
            *doc.save_status.write() = SaveStatus::Unsaved;
            *doc.file_external_modified.write() = false;
            state.refresh_file_watch();
        }

        let workspace_deleted = workspace
            .as_ref()
            .is_some_and(|root| Self::path_is_affected(root, path));
        let mut ui = state.ui();
        if workspace_deleted {
            *ui.workspace_root.write() = None;
            *ui.file_list.write() = Vec::new();
        } else if let Some(workspace) = workspace {
            *ui.file_list.write() = file_utils::scan_markdown_files(&workspace);
        }

        Ok(())
    }

    /// 重命名文件/文件夹 / Rename File/Folder
    pub fn rename_file(
        state: &mut AppState,
        old_path: &Path,
        new_name: &str,
    ) -> Result<PathBuf, String> {
        file_utils::validate_rename_name(new_name)?;

        let new_path = old_path
            .parent()
            .map(|p| p.join(new_name))
            .ok_or_else(|| "无效路径".to_string())?;

        let workspace = state.ui().workspace_root.read().clone();
        if let Some(ref workspace) = workspace {
            file_utils::ensure_within_workspace(old_path, workspace)?;
            if old_path != workspace {
                file_utils::ensure_within_workspace(&new_path, workspace)?;
            }
        }

        if new_path.exists() {
            return Err("目标名称已存在".to_string());
        }

        state.save_current_tab_content();
        let old_current_key = state.current_ai_session_key();
        fs::rename(old_path, &new_path).map_err(|e| format!("无法重命名: {}", e))?;

        {
            let mut doc = state.document();
            let mut tabs = doc.tabs.write();
            for tab in tabs.iter_mut() {
                let Some(tab_path) = tab.path.clone() else {
                    continue;
                };
                if !Self::path_is_affected(&tab_path, old_path) {
                    continue;
                }
                let replaced = Self::replace_path_prefix(&tab_path, old_path, &new_path);
                tab.title = Self::tab_title_for_path(&replaced);
                tab.ai_session_key = crate::state::ai_session_key_for_path(&replaced);
                tab.path = Some(replaced);
            }
        }

        let current_replaced = state
            .document()
            .current_file
            .read()
            .clone()
            .filter(|current| Self::path_is_affected(current, old_path))
            .map(|current| Self::replace_path_prefix(&current, old_path, &new_path));
        if let Some(current) = current_replaced {
            *state.document().current_file.write() = Some(current);
            *state.document().file_external_modified.write() = false;
            state.refresh_file_watch();
            let new_current_key = state.current_ai_session_key();
            if old_current_key != new_current_key {
                state.persist_ai_history_key(&new_current_key);
            }
        }

        let updated_workspace = workspace.as_ref().map(|root| {
            if Self::path_is_affected(root, old_path) {
                Self::replace_path_prefix(root, old_path, &new_path)
            } else {
                root.clone()
            }
        });
        let mut ui = state.ui();
        *ui.workspace_root.write() = updated_workspace.clone();
        if let Some(root) = updated_workspace {
            *ui.file_list.write() = file_utils::scan_markdown_files(&root);
        } else {
            *ui.file_list.write() = Vec::new();
        }

        Ok(new_path)
    }

    /// 新建标签页 / New Tab
    pub fn new_tab(state: &mut AppState) {
        state.new_tab();
    }

    /// 先 flush 非受控编辑器再新建标签 / Flush uncontrolled editor then create tab
    pub async fn new_tab_flushed(state: &mut AppState) {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::new_tab(state);
    }

    /// 切换标签页 / Switch Tab
    pub fn switch_tab(state: &mut AppState, index: usize) {
        state.switch_to_tab(index);
    }

    /// 先 flush 非受控编辑器再切换标签 / Flush uncontrolled editor then switch tab
    pub async fn switch_tab_flushed(state: &mut AppState, index: usize) {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::switch_tab(state, index);
    }
}
