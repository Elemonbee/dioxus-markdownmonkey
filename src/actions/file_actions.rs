//! 文件操作 Actions / File Operation Actions
//!
//! 处理文件打开、保存、工作区等操作

use crate::config::LARGE_FILE_THRESHOLD_BYTES;
use crate::state::AppState;
use crate::state::SaveStatus;
use crate::utils::file_utils;
use dioxus::prelude::{ReadableExt, WritableExt};
use std::fs;
use std::path::{Path, PathBuf};

/// 文件 Actions 处理器 / File Actions Handler
pub struct FileActions;

impl FileActions {
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

        *state.document().file_encoding.write() = encoding;
        state.open_file_in_tab(path, content);
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

                *doc.file_encoding.write() = encoding;
                state.open_file_in_tab(path, content);
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

    /// 带编码检测的文件读取 / Read file with encoding detection
    /// 检测顺序：BOM → UTF-8（无 BOM）→ GBK/GB2312
    /// Detection order: BOM → UTF-8 (no BOM) → GBK/GB2312
    /// 返回 (内容, 编码名称) / Returns (content, encoding name)
    pub(crate) fn read_file_with_encoding(path: &Path) -> Result<(String, String), String> {
        // 先读取原始字节 / First read raw bytes
        let bytes = fs::read(path).map_err(|e| format!("无法读取文件: {}", e))?;

        if bytes.is_empty() {
            return Ok((String::new(), "UTF-8".to_string()));
        }

        // 1. 检查 BOM 标记（优先级最高）/ Check BOM markers (highest priority)

        // UTF-8 BOM: EF BB BF
        if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
            // UTF-8 with BOM：去掉 BOM 后返回
            let content = String::from_utf8(bytes[3..].to_vec())
                .map_err(|e| format!("UTF-8 BOM 解码失败: {}", e))?;
            return Ok((content, "UTF-8 BOM".to_string()));
        }

        // UTF-16 LE BOM: FF FE
        if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
            let data = &bytes[2..];
            if data.len() % 2 != 0 {
                tracing::warn!("UTF-16 LE 文件被截断: {:?}", path);
                let safe_len = data.len() - (data.len() % 2);
                let content = Self::utf16_decode(&data[..safe_len], true)?;
                return Ok((content, "UTF-16 LE".to_string()));
            }
            let content = Self::utf16_decode(data, true)?;
            return Ok((content, "UTF-16 LE".to_string()));
        }

        // UTF-16 BE BOM: FE FF
        if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
            let data = &bytes[2..];
            if data.len() % 2 != 0 {
                tracing::warn!("UTF-16 BE 文件被截断: {:?}", path);
                let safe_len = data.len() - (data.len() % 2);
                let content = Self::utf16_decode(&data[..safe_len], false)?;
                return Ok((content, "UTF-16 BE".to_string()));
            }
            let content = Self::utf16_decode(data, false)?;
            return Ok((content, "UTF-16 BE".to_string()));
        }

        // 2. 尝试纯 UTF-8（无 BOM）/ Try pure UTF-8 (no BOM)
        match String::from_utf8(bytes.clone()) {
            Ok(content) => return Ok((content, "UTF-8".to_string())),
            Err(_) => {
                tracing::info!("文件不是有效 UTF-8，尝试 GBK/GB2312: {:?}", path);
            }
        }

        // 3. 尝试 GBK/GB2312 编码 / Try GBK/GB2312 encoding
        let (cow, encoding_used, _had_errors) = encoding_rs::GBK.decode(&bytes);
        tracing::info!(
            "使用 {} 编码读取文件: {:?} / File read with {} encoding: {:?}",
            encoding_used.name(),
            path,
            encoding_used.name(),
            path
        );
        Ok((cow.into_owned(), encoding_used.name().to_string()))
    }

    /// UTF-16 字节流转为 String / Convert UTF-16 byte stream to String
    fn utf16_decode(data: &[u8], little_endian: bool) -> Result<String, String> {
        let u16_vec: Vec<u16> = data
            .chunks_exact(2)
            .map(|c| {
                if little_endian {
                    u16::from_le_bytes([c[0], c[1]])
                } else {
                    u16::from_be_bytes([c[0], c[1]])
                }
            })
            .collect();

        String::from_utf16(&u16_vec).map_err(|e| format!("UTF-16 解码失败: {}", e))
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
        let revision = *doc.content_revision.read();

        match current_file {
            Some(path) => {
                *doc.save_status.write() = SaveStatus::Saving;
                fs::write(&path, &content).map_err(|e| format!("无法保存文件: {}", e))?;
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
        let revision = *doc.content_revision.read();

        match current_file {
            Some(path) => {
                *doc.save_status.write() = SaveStatus::Saving;
                let write_path = path.clone();
                let write_content = content.clone();
                tokio::task::spawn_blocking(move || fs::write(&write_path, &write_content))
                    .await
                    .map_err(|e| format!("保存任务失败 / Save task failed: {}", e))?
                    .map_err(|e| format!("无法保存文件: {}", e))?;

                if *doc.content_revision.read() == revision {
                    state.mark_saved();
                    Ok(())
                } else {
                    // 写入过期内容后立即用最新内容再写一次 / Rewrite with latest content after stale write
                    let latest = doc.content.read().clone();
                    let latest_rev = *doc.content_revision.read();
                    let rewrite_path = path;
                    tokio::task::spawn_blocking(move || fs::write(&rewrite_path, &latest))
                        .await
                        .map_err(|e| format!("保存任务失败 / Save task failed: {}", e))?
                        .map_err(|e| format!("无法保存文件: {}", e))?;
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
        let mut doc = state.document();
        let path = doc
            .current_file
            .read()
            .clone()
            .ok_or_else(|| "没有打开的文件 / No open file".to_string())?;

        let (content, encoding) = Self::read_file_with_encoding(&path)?;
        *doc.content.write() = content;
        *doc.file_encoding.write() = encoding;
        *doc.modified.write() = false;
        {
            let body = doc.content.read().clone();
            doc.history.write().reset_with_content(&body);
        }
        state.bump_content_revision();
        state.update_outline();
        state.run_spell_check();
        state.refresh_file_watch();
        *doc.file_external_modified.write() = false;
        Ok(())
    }

    /// 先 flush，再从磁盘重载并推回 DOM
    /// Flush then reload from disk and push content into the DOM
    pub async fn reload_current_file_flushed(state: &mut AppState) -> Result<(), String> {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::reload_current_file(state)?;
        crate::actions::EditorActions::push_to_dom(&state.document().content.read());
        Ok(())
    }

    /// 按是否已修改决定确认关闭或直接关闭（同步，供测试与 flush 后调用）
    /// Confirm-close if modified, else close immediately (sync; for tests and post-flush)
    pub fn request_close_tab(state: &mut AppState, index: usize) {
        state.save_current_tab_content();
        let mut doc = state.document();
        let should_confirm = {
            let tabs = doc.tabs.read();
            tabs.get(index).map(|tab| tab.modified).unwrap_or(false)
        } || *doc.modified.read();
        if should_confirm {
            *doc.pending_close_tab_index.write() = Some(index);
            *doc.show_close_confirm.write() = true;
        } else {
            state.close_tab(index);
        }
    }

    /// 先 flush，再按是否已修改决定确认关闭或直接关闭
    /// Flush, then confirm-close if modified or close immediately
    pub async fn request_close_tab_flushed(state: &mut AppState, index: usize) {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::request_close_tab(state, index);
    }

    /// 丢弃修改并关闭标签（强制未修改后关闭）
    /// Discard changes and close tab (force unmodified, then close)
    pub fn discard_and_close_tab(state: &mut AppState, index: usize) {
        let mut doc = state.document();
        *doc.modified.write() = false;
        {
            let mut tabs = doc.tabs.write();
            if let Some(tab) = tabs.get_mut(index) {
                tab.modified = false;
            }
        }
        *doc.show_close_confirm.write() = false;
        *doc.pending_close_tab_index.write() = None;
        state.close_tab(index);
    }

    /// 丢弃修改并关闭标签（先 flush 再强制未修改后关闭）
    /// Discard changes and close tab (flush, force unmodified, then close)
    pub async fn discard_and_close_tab_flushed(state: &mut AppState, index: usize) {
        crate::actions::EditorActions::flush_from_dom(state).await;
        Self::discard_and_close_tab(state, index);
    }

    /// 先 flush，再保存并关闭标签；无路径时触发另存为，成功后再关
    /// Flush, save, then close; untitled files trigger Save-As and close after success
    pub async fn save_and_close_tab_flushed(state: &mut AppState, index: usize) {
        crate::actions::EditorActions::flush_from_dom(state).await;

        // 确保待关闭标签为当前标签，保存才写对内容
        // Ensure the tab being closed is current so save writes the right content
        let doc = state.document();
        if *doc.current_tab_index.read() != index {
            state.save_current_tab_content();
            state.switch_to_tab(index);
        } else {
            state.save_current_tab_content();
        }

        match Self::save_current_file_async(state).await {
            Ok(()) => {
                let mut doc = state.document();
                *doc.show_close_confirm.write() = false;
                *doc.pending_close_tab_index.write() = None;
                let close_idx = *doc.current_tab_index.read();
                state.close_tab(close_idx);
            }
            Err(_) if *state.document().trigger_save_as.read() => {
                // 另存为由工具栏接管；保留 pending 以便保存成功后关闭
                // Toolbar owns Save-As; keep pending index to close after success
                let mut doc = state.document();
                *doc.show_close_confirm.write() = false;
                *doc.pending_close_tab_index.write() = Some(*doc.current_tab_index.read());
            }
            Err(e) => {
                tracing::error!("Save and close failed: {}", e);
            }
        }
    }

    /// 另存为 / Save As
    pub fn save_as(state: &mut AppState, path: PathBuf) -> Result<(), String> {
        let mut doc = state.document();
        let content = doc.content.read().clone();

        // 确保文件扩展名
        let path = if path.extension().is_none() {
            let mut p = path;
            p.set_extension("md");
            p
        } else {
            path
        };

        *doc.save_status.write() = SaveStatus::Saving;
        fs::write(&path, &content).map_err(|e| format!("无法保存文件: {}", e))?;

        let idx = *doc.current_tab_index.read();
        let old_key = state.current_ai_session_key();
        let new_key = crate::state::ai_session_key_for_path(&path);
        *doc.current_file.write() = Some(path.clone());
        let _ = doc;

        {
            let mut doc = state.document();
            let mut tabs = doc.tabs.write();
            if let Some(tab) = tabs.get_mut(idx) {
                tab.ai_session_key = new_key.clone();
                tab.path = Some(path);
                tab.title = tab
                    .path
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string();
            }
        }
        if old_key != new_key {
            state.persist_ai_history_key(&new_key);
        }
        state.mark_saved();
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

    /// 删除文件 / Delete File
    pub fn delete_file(state: &mut AppState, path: &Path) -> Result<(), String> {
        let mut ui = state.ui();
        let workspace = ui.workspace_root.read().clone();
        if let Some(ref workspace) = workspace {
            file_utils::ensure_within_workspace(path, workspace)?;
        }

        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| format!("无法删除文件夹: {}", e))?;
        } else {
            fs::remove_file(path).map_err(|e| format!("无法删除文件: {}", e))?;
        }

        if let Some(workspace) = workspace {
            let files = file_utils::scan_markdown_files(&workspace);
            *ui.file_list.write() = files;
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

        let mut ui = state.ui();
        let workspace = ui.workspace_root.read().clone();
        if let Some(ref workspace) = workspace {
            file_utils::ensure_within_workspace(old_path, workspace)?;
            file_utils::ensure_within_workspace(&new_path, workspace)?;
        }

        if new_path.exists() {
            return Err("目标名称已存在".to_string());
        }

        fs::rename(old_path, &new_path).map_err(|e| format!("无法重命名: {}", e))?;

        if let Some(workspace) = workspace {
            let files = file_utils::scan_markdown_files(&workspace);
            *ui.file_list.write() = files;
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
