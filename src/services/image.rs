#![allow(dead_code)]
//! 图片服务 / Image Service
//! 处理图片粘贴和保存 / Handle image paste and save
//!
//! 注意：此模块为预留功能，暂未集成到 UI
//! Note: This module is reserved for future use, not yet integrated into UI

use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::path::{Path, PathBuf};

/// 图片错误类型 / Image Error Types
#[derive(Debug)]
pub enum ImageError {
    ClipboardError(String),
    WriteError(String),
    InvalidFormat(String),
}

/// 支持的图片格式 / Supported Image Formats
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
}

impl ImageFormat {
    /// 从 MIME 类型获取格式 / Get format from MIME type
    pub fn from_mime(mime: &str) -> Option<Self> {
        match mime {
            "image/png" => Some(Self::Png),
            "image/jpeg" | "image/jpg" => Some(Self::Jpeg),
            "image/gif" => Some(Self::Gif),
            "image/webp" => Some(Self::WebP),
            _ => None,
        }
    }

    /// 获取文件扩展名 / Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::WebP => "webp",
        }
    }
}

/// 图片服务 / Image Service
pub struct ImageService;

impl ImageService {
    /// 从 base64 数据保存图片 / Save image from base64 data
    pub fn save_image_from_base64(
        base64_data: &str,
        format: ImageFormat,
        output_dir: &Path,
        filename: Option<&str>,
    ) -> Result<PathBuf, ImageError> {
        // 创建输出目录 / Create output directory
        if !output_dir.exists() {
            fs::create_dir_all(output_dir).map_err(|e| ImageError::WriteError(e.to_string()))?;
        }

        // 解码 base64 / Decode base64
        let data = if base64_data.contains(',') {
            base64_data.split(',').nth(1).unwrap_or(base64_data)
        } else {
            base64_data
        };

        let bytes = general_purpose::STANDARD
            .decode(data.trim())
            .map_err(|e| ImageError::InvalidFormat(e.to_string()))?;

        // 生成文件名 / Generate filename
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let name = filename
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("image_{}", timestamp));

        let filename = format!("{}.{}", name, format.extension());
        let path = output_dir.join(&filename);

        // 保存文件 / Save file
        fs::write(&path, bytes).map_err(|e| ImageError::WriteError(e.to_string()))?;

        Ok(path)
    }

    /// 生成 Markdown 图片语法 / Generate Markdown image syntax
    pub fn generate_markdown(path: &Path, alt_text: Option<&str>) -> String {
        let alt = alt_text.unwrap_or("图片/Image");
        format!("![{}]({})", alt, path.display())
    }

    /// 处理粘贴的图片数据 / Handle pasted image data
    pub fn handle_pasted_image(
        image_data: &str,
        mime_type: &str,
        workspace: Option<&Path>,
    ) -> Option<String> {
        let format = ImageFormat::from_mime(mime_type)?;

        // 确定输出目录 / Determine output directory
        let output_dir = workspace.map(|p| p.join("images")).unwrap_or_else(|| {
            dirs::picture_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("markdownmonkey_images")
        });

        // 保存图片 / Save image
        match Self::save_image_from_base64(image_data, format, &output_dir, None) {
            Ok(path) => Some(Self::generate_markdown(&path, None)),
            Err(e) => {
                tracing::error!("保存图片失败/Failed to save image: {:?}", e);
                None
            }
        }
    }
}
