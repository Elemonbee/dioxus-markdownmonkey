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
            *state.file_size_bytes.write() = file_size;
            *state.pending_large_file.write() = Some(path);
            *state.show_large_file_warning.write() = true;
            return Ok(());
        }

        // 使用编码检测读取文件 / Read file with encoding detection
        let content = Self::read_file_with_encoding(&path)?;

        // 记录打开文件前的状态 / Log state before opening file
        tracing::info!(
            "[FileActions::open_file] path={:?}, content_len={}",
            path,
            content.len()
        );

        state.open_file_in_tab(path, content);
        Ok(())
    }

    /// 确认加载大文件 / Confirm loading large file
    /// 由 LargeFileWarningModal 的"继续编辑"按钮调用
    /// Called by the "Continue" button in LargeFileWarningModal
    pub fn confirm_load_large_file(state: &mut AppState) -> Result<(), String> {
        let pending_path = state.pending_large_file.read().clone();

        match pending_path {
            Some(path) => {
                // 清除待加载状态 / Clear pending state
                *state.pending_large_file.write() = None;
                *state.show_large_file_warning.write() = false;

                // 读取文件内容 / Read file content
                let content = Self::read_file_with_encoding(&path)?;
                tracing::info!(
                    "[FileActions::confirm_load_large_file] Loading large file: {:?}, content_len={}",
                    path, content.len()
                );

                state.open_file_in_tab(path, content);
                Ok(())
            }
            None => Err("没有待加载的大文件 / No pending large file".to_string()),
        }
    }

    /// 取消加载大文件 / Cancel loading large file
    /// 由 LargeFileWarningModal 的"取消"按钮调用
    /// Called by the "Cancel" button in LargeFileWarningModal
    pub fn cancel_load_large_file(state: &mut AppState) {
        *state.pending_large_file.write() = None;
        *state.show_large_file_warning.write() = false;
        *state.file_size_bytes.write() = 0;
    }

    /// 带编码检测的文件读取 / Read file with encoding detection
    /// 检测顺序：BOM → UTF-8（无 BOM）→ GBK/GB2312
    /// Detection order: BOM → UTF-8 (no BOM) → GBK/GB2312
    pub(crate) fn read_file_with_encoding(path: &Path) -> Result<String, String> {
        // 先读取原始字节 / First read raw bytes
        let bytes = fs::read(path).map_err(|e| format!("无法读取文件: {}", e))?;

        if bytes.is_empty() {
            return Ok(String::new());
        }

        // 1. 检查 BOM 标记（优先级最高）/ Check BOM markers (highest priority)

        // UTF-8 BOM: EF BB BF
        if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
            // UTF-8 with BOM：去掉 BOM 后返回
            return String::from_utf8(bytes[3..].to_vec())
                .map_err(|e| format!("UTF-8 BOM 解码失败: {}", e));
        }

        // UTF-16 LE BOM: FF FE
        if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
            let data = &bytes[2..];
            if data.len() % 2 != 0 {
                tracing::warn!("UTF-16 LE 文件被截断: {:?}", path);
                // 截断文件：丢弃最后一个不完整字节
                let safe_len = data.len() - (data.len() % 2);
                return Self::utf16_decode(&data[..safe_len], true);
            }
            return Self::utf16_decode(data, true);
        }

        // UTF-16 BE BOM: FE FF
        if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
            let data = &bytes[2..];
            if data.len() % 2 != 0 {
                tracing::warn!("UTF-16 BE 文件被截断: {:?}", path);
                let safe_len = data.len() - (data.len() % 2);
                return Self::utf16_decode(&data[..safe_len], false);
            }
            return Self::utf16_decode(data, false);
        }

        // 2. 尝试纯 UTF-8（无 BOM）/ Try pure UTF-8 (no BOM)
        match String::from_utf8(bytes.clone()) {
            Ok(content) => return Ok(content),
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
        Ok(cow.into_owned())
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
    pub fn save_current_file(state: &mut AppState) -> Result<(), String> {
        let current_file = state.current_file.read().clone();
        let content = state.content.read().clone();

        match current_file {
            Some(path) => {
                *state.save_status.write() = SaveStatus::Saving;
                fs::write(&path, &content).map_err(|e| format!("无法保存文件: {}", e))?;
                state.mark_saved();
                Ok(())
            }
            None => {
                *state.trigger_save_as.write() = true;
                Err("没有文件路径，请选择保存位置".to_string())
            }
        }
    }

    /// 另存为 / Save As
    pub fn save_as(state: &mut AppState, path: PathBuf) -> Result<(), String> {
        let content = state.content.read().clone();

        // 确保文件扩展名
        let path = if path.extension().is_none() {
            let mut p = path;
            p.set_extension("md");
            p
        } else {
            path
        };

        *state.save_status.write() = SaveStatus::Saving;
        fs::write(&path, &content).map_err(|e| format!("无法保存文件: {}", e))?;

        *state.current_file.write() = Some(path);
        state.mark_saved();
        Ok(())
    }

    /// 设置工作区 / Set Workspace
    pub fn set_workspace(state: &mut AppState, path: PathBuf) {
        *state.workspace_root.write() = Some(path.clone());
        let files = file_utils::scan_markdown_files(&path);
        *state.file_list.write() = files;
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
        if let Some(workspace) = state.workspace_root.read().clone() {
            let files = file_utils::scan_markdown_files(&workspace);
            *state.file_list.write() = files;
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
        if let Some(workspace) = state.workspace_root.read().clone() {
            let files = file_utils::scan_markdown_files(&workspace);
            *state.file_list.write() = files;
        }

        Ok(path)
    }

    /// 刷新工作区 / Refresh Workspace
    pub fn refresh_workspace(state: &mut AppState) {
        if let Some(workspace) = state.workspace_root.read().clone() {
            let files = file_utils::scan_markdown_files(&workspace);
            *state.file_list.write() = files;
        }
    }

    /// 删除文件 / Delete File
    pub fn delete_file(state: &mut AppState, path: &Path) -> Result<(), String> {
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| format!("无法删除文件夹: {}", e))?;
        } else {
            fs::remove_file(path).map_err(|e| format!("无法删除文件: {}", e))?;
        }

        if let Some(workspace) = state.workspace_root.read().clone() {
            let files = file_utils::scan_markdown_files(&workspace);
            *state.file_list.write() = files;
        }

        Ok(())
    }

    /// 重命名文件/文件夹 / Rename File/Folder
    pub fn rename_file(
        state: &mut AppState,
        old_path: &Path,
        new_name: &str,
    ) -> Result<PathBuf, String> {
        let new_path = old_path
            .parent()
            .map(|p| p.join(new_name))
            .ok_or_else(|| "无效路径".to_string())?;

        if new_path.exists() {
            return Err("目标名称已存在".to_string());
        }

        fs::rename(old_path, &new_path).map_err(|e| format!("无法重命名: {}", e))?;

        if let Some(workspace) = state.workspace_root.read().clone() {
            let files = file_utils::scan_markdown_files(&workspace);
            *state.file_list.write() = files;
        }

        Ok(new_path)
    }

    /// 新建标签页 / New Tab
    pub fn new_tab(state: &mut AppState) {
        state.new_tab();
    }

    /// 切换标签页 / Switch Tab
    pub fn switch_tab(state: &mut AppState, index: usize) {
        state.switch_to_tab(index);
    }
}
