#![allow(dead_code)]
//! 导出服务 / Export Service
//! 支持 PDF 和 HTML 导出 / Support PDF and HTML Export
//!
//! PDF 导出使用系统字体支持中文 / PDF export uses system fonts for Chinese support
//! PDF export uses system fonts for Chinese support

use printpdf::*;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use thiserror::Error;

/// 系统中文字体路径 / System Chinese Font Paths
fn get_system_cjk_font_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        // Windows: 微软雅黑 / Microsoft YaHei
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
        paths.push(
            std::path::PathBuf::from(&windir)
                .join("Fonts")
                .join("msyh.ttc"),
        );
        paths.push(
            std::path::PathBuf::from(&windir)
                .join("Fonts")
                .join("msyh.ttf"),
        );
        paths.push(
            std::path::PathBuf::from(&windir)
                .join("Fonts")
                .join("simhei.ttf"),
        );
        paths.push(
            std::path::PathBuf::from(&windir)
                .join("Fonts")
                .join("simsun.ttc"),
        );
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 苹方 / PingFang, 黑体 / Heiti
        paths.push(std::path::PathBuf::from(
            "/System/Library/Fonts/PingFang.ttc",
        ));
        paths.push(std::path::PathBuf::from(
            "/System/Library/Fonts/STHeiti Light.ttc",
        ));
        paths.push(std::path::PathBuf::from(
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ));
        paths.push(std::path::PathBuf::from("/Library/Fonts/Arial Unicode.ttf"));
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: Noto CJK, WenQuanYi
        paths.push(std::path::PathBuf::from(
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        ));
        paths.push(std::path::PathBuf::from(
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ));
        paths.push(std::path::PathBuf::from(
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        ));
        paths.push(std::path::PathBuf::from(
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        ));
        paths.push(std::path::PathBuf::from(
            "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",
        ));
    }

    paths
}

/// 查找可用的系统字体 / Find available system font
fn find_available_system_font() -> Option<(std::path::PathBuf, Vec<u8>)> {
    let paths = get_system_cjk_font_paths();

    for path in paths {
        if path.exists() {
            if let Ok(mut file) = File::open(&path) {
                let mut buffer = Vec::new();
                if file.read_to_end(&mut buffer).is_ok() {
                    tracing::info!("使用系统字体/Using system font: {:?}", path);
                    return Some((path, buffer));
                }
            }
        }
    }

    tracing::warn!("未找到可用的系统中文字体/No available system Chinese font found");
    None
}

/// 导出错误类型 / Export Error Types
#[derive(Error, Debug)]
pub enum ExportError {
    #[error("PDF 导出错误/PDF Export Error: {0}")]
    Pdf(String),

    #[error("IO 错误/IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("渲染错误/Render Error: {0}")]
    Render(String),

    #[error("字体错误/Font Error: {0}")]
    Font(String),

    #[error("ZIP 错误/ZIP Error: {0}")]
    Zip(String),
}

impl From<zip::result::ZipError> for ExportError {
    fn from(e: zip::result::ZipError) -> Self {
        ExportError::Zip(e.to_string())
    }
}

impl From<printpdf::Error> for ExportError {
    fn from(e: printpdf::Error) -> Self {
        ExportError::Pdf(e.to_string())
    }
}

/// PDF 导出配置 / PDF Export Configuration
#[derive(Clone, Debug)]
pub struct PdfExportConfig {
    /// 页面宽度 (mm) / Page width (mm)
    pub page_width: f32,
    /// 页面高度 (mm) / Page height (mm)
    pub page_height: f32,
    /// 边距 (mm) / Margin (mm)
    pub margin: f32,
    /// 字体大小 / Font size
    pub font_size: f32,
    /// 行高倍数 / Line height multiplier
    pub line_height_multiplier: f32,
}

impl Default for PdfExportConfig {
    fn default() -> Self {
        Self {
            page_width: 210.0,  // A4 宽度
            page_height: 297.0, // A4 高度
            margin: 20.0,
            font_size: 11.0,
            line_height_multiplier: 1.4,
        }
    }
}

/// 导出服务 / Export Service
pub struct ExportService;

impl ExportService {
    /// 检查文本是否包含 CJK 字符 / Check if text contains CJK characters
    fn contains_cjk(text: &str) -> bool {
        text.chars().any(Self::is_cjk_char)
    }

    /// 导出为 PDF（改进版）/ Export to PDF (Improved)
    pub fn export_to_pdf(content: &str, output_path: &Path) -> Result<(), ExportError> {
        Self::export_to_pdf_with_config(content, output_path, PdfExportConfig::default())
    }

    /// 使用自定义配置导出为 PDF / Export to PDF with custom configuration
    pub fn export_to_pdf_with_config(
        content: &str,
        output_path: &Path,
        config: PdfExportConfig,
    ) -> Result<(), ExportError> {
        // 创建文档 / Create document
        let page_width = Mm(config.page_width);
        let page_height = Mm(config.page_height);

        let (doc, page1, layer1) =
            PdfDocument::new("Markdown Export", page_width, page_height, "Layer 1");

        // 检查内容是否包含中文，选择合适的字体
        // Check if content contains Chinese, select appropriate font
        let has_cjk = Self::contains_cjk(content);

        // 根据内容选择字体 / Select font based on content
        let font = if has_cjk {
            // 尝试加载系统中文字体 / Try to load system Chinese font
            if let Some((_path, font_data)) = find_available_system_font() {
                use std::io::Cursor;
                doc.add_external_font(Cursor::new(font_data)).map_err(|e| {
                    ExportError::Font(format!(
                        "加载系统字体失败/Failed to load system font: {}",
                        e
                    ))
                })?
            } else {
                tracing::warn!(
                    "未找到中文字体，中文可能无法正确显示 / \
                     No Chinese font found, Chinese may not display correctly"
                );
                doc.add_builtin_font(BuiltinFont::Helvetica)
                    .map_err(|e| ExportError::Font(e.to_string()))?
            }
        } else {
            doc.add_builtin_font(BuiltinFont::Helvetica)
                .map_err(|e| ExportError::Font(e.to_string()))?
        };

        let line_height = config.font_size * config.line_height_multiplier;
        // 使用 mm 单位计算内容宽度 / Calculate content width in mm units
        let content_width_mm = config.page_width - (config.margin * 2.0);

        let mut current_layer = doc.get_page(page1).get_layer(layer1);
        let mut y_pos = config.page_height - config.margin;
        let mut page_count = 1;
        let mut in_code_block = false;
        let mut code_block_lines: Vec<String> = Vec::new();

        for line in content.lines() {
            // 处理代码块开始/结束 / Handle code block start/end
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") {
                if in_code_block {
                    // 代码块结束，渲染代码 / Code block end, render code
                    in_code_block = false;
                    Self::render_code_block(
                        &code_block_lines.join("\n"),
                        &doc,
                        &mut current_layer,
                        &font,
                        &mut y_pos,
                        &mut page_count,
                        &config,
                        page_width,
                        page_height,
                    );
                    code_block_lines.clear();
                    y_pos -= line_height * 0.5; // 代码块后间距 / Space after code block
                } else {
                    // 代码块开始 / Code block start
                    in_code_block = true;
                    code_block_lines.clear();
                }
                continue;
            }

            // 在代码块内收集行 / Collect lines inside code block
            if in_code_block {
                code_block_lines.push(line.to_string());
                continue;
            }

            // 处理空行 / Handle empty lines
            if line.is_empty() {
                y_pos -= line_height;
                continue;
            }

            // 处理 Markdown 标题 / Handle Markdown headings
            // 注意：printpdf 不支持内联粗体，_is_bold 仅用于未来扩展
            // Note: printpdf doesn't support inline bold, _is_bold reserved for future use
            let (text, font_size_override, _is_bold) = Self::process_markdown_line_v2(line);
            let current_font_size = font_size_override.unwrap_or(config.font_size);

            // 基于实际宽度换行 / Wrap text based on actual measured width
            let wrapped_lines =
                Self::wrap_text_by_width(&text, content_width_mm, current_font_size);

            for wrapped_line in wrapped_lines {
                // 检查是否需要换页 / Check if page break is needed
                if y_pos < config.margin + current_font_size {
                    let (new_page, new_layer) = doc.add_page(page_width, page_height, "Layer 1");
                    current_layer = doc.get_page(new_page).get_layer(new_layer);
                    y_pos = config.page_height - config.margin;
                    page_count += 1;

                    // 添加页码 / Add page number
                    let page_text = format!("- {} -", page_count);
                    current_layer.use_text(
                        &page_text,
                        9.0,
                        Mm((config.page_width - 20.0) / 2.0),
                        Mm(config.margin / 2.0),
                        &font,
                    );
                }

                // 渲染文本 / Render text
                current_layer.use_text(
                    &wrapped_line,
                    current_font_size,
                    Mm(config.margin),
                    Mm(y_pos),
                    &font,
                );

                y_pos -= current_font_size * config.line_height_multiplier;
            }
        }

        // 处理未闭合的代码块 / Handle unclosed code block
        if in_code_block && !code_block_lines.is_empty() {
            Self::render_code_block(
                &code_block_lines.join("\n"),
                &doc,
                &mut current_layer,
                &font,
                &mut y_pos,
                &mut page_count,
                &config,
                page_width,
                page_height,
            );
        }

        // 保存文件 / Save file
        doc.save(&mut BufWriter::new(File::create(output_path)?))?;

        tracing::info!(
            "PDF 导出完成，共 {} 页 / PDF export completed, {} pages",
            page_count,
            page_count
        );
        Ok(())
    }

    /// 渲染代码块（自动换行而非截断）/ Render code block (auto-wrap instead of truncate)
    #[allow(clippy::too_many_arguments)]
    fn render_code_block(
        code: &str,
        doc: &PdfDocumentReference,
        current_layer: &mut PdfLayerReference,
        font: &IndirectFontRef,
        y_pos: &mut f32,
        page_count: &mut usize,
        config: &PdfExportConfig,
        page_width: Mm,
        page_height: Mm,
    ) {
        let code_font_size = config.font_size * 0.85;
        let line_height = code_font_size * config.line_height_multiplier;
        // 代码块内缩 3mm / Code block indent 3mm
        let code_width_mm = config.page_width - (config.margin * 2.0) - 6.0;

        for code_line in code.lines() {
            // 自动换行而非截断 / Auto-wrap instead of truncate
            let wrapped = if code_line.is_empty() {
                vec![" ".to_string()]
            } else {
                Self::wrap_text_by_width(code_line, code_width_mm, code_font_size)
            };

            for display_line in &wrapped {
                // 检查换页 / Check page break
                if *y_pos < config.margin + code_font_size {
                    let (new_page, new_layer) = doc.add_page(page_width, page_height, "Layer 1");
                    *current_layer = doc.get_page(new_page).get_layer(new_layer);
                    *y_pos = config.page_height - config.margin;
                    *page_count += 1;
                }

                current_layer.use_text(
                    display_line,
                    code_font_size,
                    Mm(config.margin + 3.0), // 左侧缩进 / Left indent
                    Mm(*y_pos),
                    font,
                );

                *y_pos -= line_height;
            }
        }
    }

    /// 基于实际宽度换行（带安全系数）/ Wrap text based on width (with safety margin)
    fn wrap_text_by_width(text: &str, max_width_mm: f32, font_size_pt: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0_f32;

        // 安全系数：预留 10% 宽度余量，防止估算偏差导致溢出
        // Safety margin: reserve 10% to prevent overflow from estimation errors
        let safe_max_width = max_width_mm * 0.9;

        for ch in text.chars() {
            let char_w = if Self::is_cjk_char(ch) {
                // CJK 字符通常占满一个 em 方块
                // CJK characters typically fill a full em square
                font_size_pt * 0.3528 * 1.0
            } else if ch.is_ascii() {
                let ratio = match ch {
                    'i' | 'l' | '!' | '.' | ',' | ':' | ';' | '\'' | '`' => 0.30,
                    'I' | 'f' | 'j' | '(' | ')' | '[' | ']' | '{' | '}' | '"' => 0.38,
                    't' | 'r' | '1' | '/' | '\\' => 0.40,
                    'W' | 'M' | 'm' | '@' | 'O' | 'Q' => 0.72,
                    'w' | 'G' | 'C' | 'D' | 'H' | 'N' | 'R' | 'S' | 'U' | 'V' | 'X' | 'Y' | 'Z' => {
                        0.60
                    }
                    ' ' => 0.25,
                    _ => 0.52,
                };
                font_size_pt * 0.3528 * ratio
            } else {
                // 非 ASCII 非 CJK（如西里尔字母、阿拉伯字母等）
                font_size_pt * 0.3528 * 0.80
            };

            if current_width + char_w > safe_max_width && !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
            }

            current_line.push(ch);
            current_width += char_w;
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// 检查是否为 CJK 字符 / Check if character is CJK
    fn is_cjk_char(ch: char) -> bool {
        matches!(ch,
            '\u{4E00}'..='\u{9FFF}' |      // CJK Unified Ideographs
            '\u{3400}'..='\u{4DBF}' |      // CJK Unified Ideographs Extension A
            '\u{20000}'..='\u{2A6DF}' |    // CJK Unified Ideographs Extension B
            '\u{F900}'..='\u{FAFF}' |      // CJK Compatibility Ideographs
            '\u{2F800}'..='\u{2FA1F}' |    // CJK Compatibility Ideographs Supplement
            '\u{3000}'..='\u{303F}' |      // CJK Symbols and Punctuation
            '\u{FF00}'..='\u{FFEF}'        // Halfwidth and Fullwidth Forms
        )
    }

    /// 处理 Markdown 行 / Process Markdown line
    fn process_markdown_line_v2(line: &str) -> (String, Option<f32>, bool) {
        let trimmed = line.trim_start();

        // 标题处理（加粗显示）/ Heading processing (bold display)
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return (stripped.to_string(), Some(20.0), true);
        } else if let Some(stripped) = trimmed.strip_prefix("## ") {
            return (stripped.to_string(), Some(16.0), true);
        } else if let Some(stripped) = trimmed.strip_prefix("### ") {
            return (stripped.to_string(), Some(14.0), true);
        } else if let Some(stripped) = trimmed.strip_prefix("#### ") {
            return (stripped.to_string(), Some(13.0), true);
        } else if let Some(stripped) = trimmed.strip_prefix("##### ") {
            return (stripped.to_string(), Some(12.0), true);
        } else if let Some(stripped) = trimmed.strip_prefix("###### ") {
            return (stripped.to_string(), Some(11.0), true);
        }

        // 去除内联格式 / Strip inline formatting
        let stripped_line = Self::strip_inline_markdown(line);
        let trimmed_stripped = stripped_line.trim_start();

        // 任务列表 / Task list (- [ ] or - [x] or - [X])
        if let Some(rest) = trimmed_stripped.strip_prefix("- [ ] ") {
            return (format!("☐ {}", rest), None, false);
        }
        if let Some(rest) = trimmed_stripped
            .strip_prefix("- [x] ")
            .or_else(|| trimmed_stripped.strip_prefix("- [X] "))
        {
            return (format!("☑ {}", rest), None, false);
        }

        // 列表处理 / List processing
        if let Some(stripped) = trimmed_stripped
            .strip_prefix("- ")
            .or_else(|| trimmed_stripped.strip_prefix("* "))
            .or_else(|| trimmed_stripped.strip_prefix("+ "))
        {
            return (format!("• {}", stripped), None, false);
        }

        // 有序列表 / Ordered list — 仅匹配行首的 "数字. "
        // Only match "digits. " at the start of the line to avoid false positives like "See fig. 1"
        let leading_digits: String = trimmed_stripped
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if leading_digits.len() > 0
            && trimmed_stripped.get(leading_digits.len()..leading_digits.len() + 2) == Some(". ")
        {
            return (trimmed_stripped.to_string(), None, false);
        }

        // 表格行处理 / Table row processing
        if trimmed_stripped.starts_with('|') && trimmed_stripped.ends_with('|') {
            // 去掉首尾 | 并拆分 / Strip outer | and split
            let inner = &trimmed_stripped[1..trimmed_stripped.len() - 1];
            let cells: Vec<&str> = inner.split('|').collect();
            // 检查是否为分隔行 (|---|---|) / Check if separator row
            if cells.iter().all(|c| {
                c.trim()
                    .chars()
                    .all(|ch| ch == '-' || ch == ':' || ch == ' ')
            }) {
                // 跳过表格分隔行 / Skip table separator row
                return (String::new(), None, false);
            }
            // 格式化表格行 / Format table row
            let formatted: String = cells
                .iter()
                .map(|c| c.trim())
                .collect::<Vec<&str>>()
                .join(" │ ");
            return (formatted, None, false);
        }

        // 引用 / Quote
        if let Some(stripped) = trimmed_stripped.strip_prefix("> ") {
            return (format!("> {}", stripped), None, false);
        }

        // 分隔线 / Horizontal rule
        if trimmed_stripped == "---" || trimmed_stripped == "***" || trimmed_stripped == "___" {
            return ("─".repeat(50), None, false);
        }

        (stripped_line, None, false)
    }

    /// 去除内联 Markdown 格式标记 / Strip inline Markdown formatting markers
    /// 去除 **bold**, *italic*, `code`, ~~strikethrough~~, [link](url) 的标记
    fn strip_inline_markdown(text: &str) -> String {
        let mut result = text.to_string();

        // 去除粗体 **text** 或 __text__ / Strip bold
        for _ in 0..10 {
            let prev = result.clone();
            result = Self::strip_pair(&result, "**", "**");
            result = Self::strip_pair(&result, "__", "__");
            if result == prev {
                break;
            }
        }

        // 去除斜体 *text* / Strip italic
        for _ in 0..10 {
            let prev = result.clone();
            result = Self::strip_pair(&result, "*", "*");
            if result == prev {
                break;
            }
        }

        // 去除行内代码 `code` / Strip inline code
        result = Self::strip_pair(&result, "`", "`");

        // 去除删除线 ~~text~~ / Strip strikethrough
        result = Self::strip_pair(&result, "~~", "~~");

        // 简化图片 ![alt](url) -> [alt] / Simplify images
        result = Self::strip_bracket_link(&result, true);

        // 简化链接 [text](url) -> text / Simplify links
        result = Self::strip_bracket_link(&result, false);

        result
    }

    /// 去除成对的标记 / Strip paired markers
    fn strip_pair(text: &str, open: &str, close: &str) -> String {
        if let Some(start) = text.find(open) {
            let after_open = &text[start + open.len()..];
            if let Some(end) = after_open.find(close) {
                let inner = &after_open[..end];
                if !inner.is_empty() {
                    return format!(
                        "{}{}{}",
                        &text[..start],
                        inner,
                        &after_open[end + close.len()..]
                    );
                }
            }
        }
        text.to_string()
    }

    /// 去除 [text](url) 或 ![alt](url) 格式 / Strip [text](url) or ![alt](url)
    fn strip_bracket_link(text: &str, is_image: bool) -> String {
        let prefix = if is_image { "![" } else { "[" };
        let mut result = String::new();
        let chars: Vec<char> = text.chars().collect();
        let char_len = chars.len();
        let mut i = 0;

        while i < char_len {
            // 检查是否匹配前缀 / Check if prefix matches
            let remaining: String = chars[i..].iter().collect();
            if remaining.starts_with(prefix) {
                // 找到 ] / Find ]
                let bracket_start = i + prefix.chars().count();
                if let Some(bracket_offset) = chars[bracket_start..].iter().position(|&c| c == ']')
                {
                    let bracket_end = bracket_start + bracket_offset;
                    let inner: String = chars[bracket_start..bracket_end].iter().collect();
                    let after_bracket = bracket_end + 1;
                    // 检查后面是否有 (url) / Check if (url) follows
                    if after_bracket < char_len && chars[after_bracket] == '(' {
                        let paren_content: String = chars[after_bracket..].iter().collect();
                        if let Some(paren_offset) = paren_content.find(')') {
                            // 跳过 ![alt](url) 或提取 [text] / Skip ![alt](url) or extract [text]
                            if is_image {
                                result.push('[');
                                result.push_str(&inner);
                                result.push(']');
                            } else {
                                result.push_str(&inner);
                            }
                            // 跳到 ) 后面 / Skip past )
                            i = after_bracket + paren_offset + 1;
                            continue;
                        }
                    }
                }
            }
            result.push(chars[i]);
            i += 1;
        }

        result
    }

    /// 导出为 HTML / Export to HTML
    pub fn export_to_html(markdown_content: &str, output_path: &Path) -> Result<(), ExportError> {
        use crate::services::markdown::render_markdown;

        let html_content = render_markdown(markdown_content);

        let full_html = format!(
            r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Markdown Export</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            line-height: 1.6;
            color: #333;
        }}
        pre {{
            background: #f4f4f4;
            padding: 16px;
            border-radius: 4px;
            overflow-x: auto;
        }}
        code {{
            background: #f4f4f4;
            padding: 2px 6px;
            border-radius: 3px;
        }}
        blockquote {{
            border-left: 4px solid #ddd;
            margin: 0;
            padding-left: 16px;
            color: #666;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }}
        th {{
            background: #f4f4f4;
        }}
        img {{
            max-width: 100%;
        }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            html_content
        );

        std::fs::write(output_path, full_html)?;

        Ok(())
    }

    /// 导出为纯文本 / Export to Plain Text
    pub fn export_to_text(content: &str, output_path: &Path) -> Result<(), ExportError> {
        std::fs::write(output_path, content)?;
        Ok(())
    }

    /// 导出为 Word/Docx 格式 / Export to Word/Docx format
    pub fn export_to_docx(content: &str, output_path: &Path) -> Result<(), ExportError> {
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let file = File::create(output_path)?;
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // 生成文档内容 XML / Generate document content XML
        let document_xml = Self::generate_document_xml(content);

        // [Content_Types].xml - 必需 / Required
        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(content_types.as_bytes())?;

        // _rels/.rels - 必需 / Required
        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(rels.as_bytes())?;

        // word/_rels/document.xml.rels
        let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;
        zip.start_file("word/_rels/document.xml.rels", options)?;
        zip.write_all(doc_rels.as_bytes())?;

        // word/styles.xml - 字体和样式定义 / Font and style definitions
        let styles_xml = Self::generate_styles_xml();
        zip.start_file("word/styles.xml", options)?;
        zip.write_all(styles_xml.as_bytes())?;

        // word/document.xml - 主文档 / Main document
        zip.start_file("word/document.xml", options)?;
        zip.write_all(document_xml.as_bytes())?;

        zip.finish()?;

        tracing::info!("DOCX 导出完成 / DOCX export completed: {:?}", output_path);
        Ok(())
    }

    /// 生成 Word 文档 XML（支持内联格式）/ Generate Word document XML (with inline formatting)
    fn generate_document_xml(content: &str) -> String {
        let mut paragraphs = String::new();
        let mut in_code_block = false;
        let mut code_lines: Vec<String> = Vec::new();
        // 连续表格行收集 / Collect consecutive table rows
        let mut table_rows: Vec<Vec<String>> = Vec::new();

        // 刷新表格缓冲 / Flush table buffer
        let flush_table = |table_rows: &mut Vec<Vec<String>>, paragraphs: &mut String| {
            if !table_rows.is_empty() {
                paragraphs.push_str(&Self::format_table_ooxml(table_rows));
                table_rows.clear();
            }
        };

        for line in content.lines() {
            let trimmed = line.trim_start();

            // 代码块处理 / Code block handling
            if trimmed.starts_with("```") {
                flush_table(&mut table_rows, &mut paragraphs);
                if in_code_block {
                    in_code_block = false;
                    paragraphs.push_str(&Self::format_code_block(&code_lines));
                    code_lines.clear();
                } else {
                    in_code_block = true;
                    code_lines.clear();
                }
                continue;
            }

            if in_code_block {
                code_lines.push(line.to_string());
                continue;
            }

            // 检测表格行 / Detect table row
            if trimmed.starts_with('|')
                && trimmed.ends_with('|')
                && trimmed.contains('|')
                && trimmed.len() >= 2
            {
                let inner = &trimmed[1..trimmed.len() - 1];
                let cells: Vec<&str> = inner.split('|').collect();
                // 检查是否为分隔行 / Check if separator row
                let is_separator = cells.iter().all(|c| {
                    c.trim()
                        .chars()
                        .all(|ch| ch == '-' || ch == ':' || ch == ' ')
                });
                if !is_separator {
                    // 收集数据行 / Collect data row
                    table_rows.push(cells.iter().map(|c| c.trim().to_string()).collect());
                }
                // 分隔行不结束表格（只跳过）/ Separator doesn't end table (just skip)
                continue;
            }

            // 非表格行：先刷新表格 / Non-table line: flush table first
            flush_table(&mut table_rows, &mut paragraphs);

            let para_xml = if let Some(stripped) = trimmed.strip_prefix("# ") {
                Self::format_heading(stripped, 1)
            } else if let Some(stripped) = trimmed.strip_prefix("## ") {
                Self::format_heading(stripped, 2)
            } else if let Some(stripped) = trimmed.strip_prefix("### ") {
                Self::format_heading(stripped, 3)
            } else if let Some(stripped) = trimmed.strip_prefix("#### ") {
                Self::format_heading(stripped, 4)
            } else if let Some(stripped) = trimmed.strip_prefix("##### ") {
                Self::format_heading(stripped, 5)
            } else if let Some(stripped) = trimmed.strip_prefix("###### ") {
                Self::format_heading(stripped, 6)
            } else if trimmed.starts_with("- [ ] ")
                || trimmed.starts_with("- [x] ")
                || trimmed.starts_with("- [X] ")
            {
                let checked = trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ");
                let text = &trimmed[6..];
                let marker = if checked { "☑ " } else { "☐ " };
                Self::format_rich_paragraph(&format!("{}{}", marker, text))
            } else if trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.starts_with("+ ")
            {
                let text = trimmed[2..].to_string();
                Self::format_rich_paragraph_with_indent(&format!("• {}", text), 360)
            } else if let Some(text) = trimmed.strip_prefix("> ") {
                Self::format_quote(text)
            } else if trimmed == "---" || trimmed == "***" || trimmed == "___" {
                r#"<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="6" w:space="1" w:color="auto"/></w:pBdr></w:pPr></w:p>"#.to_string()
            } else if !line.is_empty() {
                let processed = Self::process_footnote_references(line);
                Self::format_rich_paragraph(&processed)
            } else {
                "<w:p/>".to_string()
            };
            paragraphs.push_str(&para_xml);
        }

        // 刷新剩余表格 / Flush remaining table
        flush_table(&mut table_rows, &mut paragraphs);

        // 未闭合代码块 / Unclosed code block
        if in_code_block && !code_lines.is_empty() {
            paragraphs.push_str(&Self::format_code_block(&code_lines));
        }

        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>
{paragraphs}
</w:body>
</w:document>"#
        )
    }

    /// 生成 OOXML 表格（多行合并为一个 w:tbl）/ Generate OOXML table (multiple rows in one w:tbl)
    fn format_table_ooxml(rows: &[Vec<String>]) -> String {
        if rows.is_empty() {
            return String::new();
        }

        let mut xml = String::from("<w:tbl>\n");
        // 表格属性 / Table properties
        xml.push_str(
            "<w:tblPr><w:tblBorders>\
            <w:top w:val=\"single\" w:sz=\"4\" w:color=\"CCCCCC\"/>\
            <w:left w:val=\"single\" w:sz=\"4\" w:color=\"CCCCCC\"/>\
            <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"CCCCCC\"/>\
            <w:right w:val=\"single\" w:sz=\"4\" w:color=\"CCCCCC\"/>\
            <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"CCCCCC\"/>\
            <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"CCCCCC\"/>\
            </w:tblBorders><w:tblW w:w=\"5000\" w:type=\"pct\"/></w:tblPr>\n",
        );

        for (row_idx, row) in rows.iter().enumerate() {
            xml.push_str("<w:tr>\n");
            let is_header = row_idx == 0;
            for cell in row {
                // 表格单元格支持内联格式 / Table cells support inline formatting
                let runs = Self::parse_inline_runs(cell);
                if is_header {
                    xml.push_str(&format!(
                        "<w:tc><w:tcPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"E8E8E8\"/></w:tcPr>\
                        <w:p><w:pPr><w:jc w:val=\"left\"/></w:pPr>\
                        <w:r><w:rPr><w:b/><w:sz w:val=\"20\"/></w:rPr>{}</w:r></w:p></w:tc>\n",
                        runs
                    ));
                } else {
                    xml.push_str(&format!(
                        "<w:tc><w:p><w:pPr><w:jc w:val=\"left\"/></w:pPr>\
                        <w:r><w:rPr><w:sz w:val=\"20\"/></w:rPr>{}</w:r></w:p></w:tc>\n",
                        runs
                    ));
                }
            }
            xml.push_str("</w:tr>\n");
        }
        xml.push_str("</w:tbl>\n");
        xml
    }

    /// 解析内联 Markdown 并生成 OOXML runs / Parse inline Markdown to OOXML runs
    /// 支持 **bold**, *italic*, `code`, ~~strikethrough~~, [link](url)
    fn parse_inline_runs(text: &str) -> String {
        let mut runs = String::new();
        let mut remaining = text;

        while !remaining.is_empty() {
            // 尝试匹配粗体 **text** / Try matching bold
            if remaining.starts_with("**") && remaining.len() > 4 {
                if let Some(end) = Self::find_closing(&remaining[2..], "**") {
                    let inner = &remaining[2..2 + end];
                    runs.push_str(&Self::format_run(inner, true, false, false, false));
                    remaining = &remaining[2 + end + 2..];
                    continue;
                }
            }

            // 粗体 __text__ / Bold __text__
            if remaining.starts_with("__") && remaining.len() > 4 {
                if let Some(end) = Self::find_closing(&remaining[2..], "__") {
                    let inner = &remaining[2..2 + end];
                    runs.push_str(&Self::format_run(inner, true, false, false, false));
                    remaining = &remaining[2 + end + 2..];
                    continue;
                }
            }

            // 行内代码 `code` / Inline code
            if remaining.starts_with("`") {
                if let Some(end) = remaining[1..].find('`') {
                    let inner = &remaining[1..1 + end];
                    runs.push_str(&Self::format_run(inner, false, false, true, false));
                    remaining = &remaining[1 + end + 1..];
                    continue;
                }
            }

            // 删除线 ~~text~~ / Strikethrough
            if remaining.starts_with("~~") && remaining.len() > 4 {
                if let Some(end) = Self::find_closing(&remaining[2..], "~~") {
                    let inner = &remaining[2..2 + end];
                    runs.push_str(&Self::format_run(inner, false, false, false, true));
                    remaining = &remaining[2 + end + 2..];
                    continue;
                }
            }

            // 斜体 *text* / Italic *text*
            if remaining.starts_with("*") && !remaining.starts_with("**") {
                if let Some(end) = remaining[1..].find('*') {
                    let inner = &remaining[1..1 + end];
                    runs.push_str(&Self::format_run(inner, false, true, false, false));
                    remaining = &remaining[1 + end + 1..];
                    continue;
                }
            }

            // 链接 [text](url) / Link [text](url)
            if remaining.starts_with("[") {
                if let Some(bracket_end) = remaining[1..].find(']') {
                    let inner = &remaining[1..1 + bracket_end];
                    let after = &remaining[1 + bracket_end + 1..];
                    if let Some(after) = after.strip_prefix("(") {
                        if let Some(paren_end) = after.find(')') {
                            runs.push_str(&Self::format_run(inner, false, false, false, false));
                            runs.push_str(&Self::format_run(
                                &format!(" ({})", &after[..paren_end]),
                                false,
                                false,
                                false,
                                false,
                            ));
                            remaining = &after[paren_end + 1..];
                            continue;
                        }
                    }
                }
            }

            // 普通文本：收集直到下一个特殊字符 / Normal text: collect until next special char
            // 注意使用字符边界而非字节索引 / Note: use char boundary, not byte index
            let special = ['*', '`', '~', '[', '_'];
            let first_char_len = remaining.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            let end_pos = remaining[first_char_len..]
                .find(|c: char| special.contains(&c))
                .map(|p| first_char_len + p)
                .unwrap_or(remaining.len());
            let plain = &remaining[..end_pos];
            runs.push_str(&Self::format_run(plain, false, false, false, false));
            remaining = &remaining[end_pos..];
        }

        runs
    }

    /// 查找非嵌套的闭合标记 / Find non-nested closing marker
    fn find_closing(text: &str, marker: &str) -> Option<usize> {
        text.find(marker)
    }

    /// 生成 OOXML run / Generate OOXML run
    fn format_run(text: &str, bold: bool, italic: bool, code: bool, strike: bool) -> String {
        let mut rpr = String::new();
        if bold || italic || code || strike {
            rpr.push_str("<w:rPr>");
            if bold {
                rpr.push_str("<w:b/>");
            }
            if italic {
                rpr.push_str("<w:i/>");
            }
            if code {
                rpr.push_str(
                    "<w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\" w:cs=\"Consolas\"/>",
                );
                rpr.push_str("<w:sz w:val=\"20\"/>");
                rpr.push_str("<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F0F0F0\"/>");
            }
            if strike {
                rpr.push_str("<w:strike/>");
            }
            rpr.push_str("</w:rPr>");
        }
        format!(
            "<w:r>{}<w:t xml:space=\"preserve\">{}</w:t></w:r>",
            rpr,
            Self::escape_xml(text)
        )
    }

    /// 格式化带内联格式的段落 / Format paragraph with inline formatting
    fn format_rich_paragraph(text: &str) -> String {
        let runs = Self::parse_inline_runs(text);
        format!("<w:p>{}</w:p>", runs)
    }

    /// 格式化带缩进的内联段落 / Format paragraph with indent and inline formatting
    fn format_rich_paragraph_with_indent(text: &str, indent_twips: i32) -> String {
        let runs = Self::parse_inline_runs(text);
        format!(
            "<w:p><w:pPr><w:ind w:left=\"{}\"/></w:pPr>{}</w:p>",
            indent_twips, runs
        )
    }

    /// 格式化引用段落 / Format quote paragraph
    fn format_quote(text: &str) -> String {
        let runs = Self::parse_inline_runs(text);
        format!(
            "<w:p><w:pPr><w:pBdr><w:left w:val=\"single\" w:sz=\"12\" w:space=\"4\" w:color=\"999999\"/></w:pBdr><w:ind w:left=\"360\"/></w:pPr>{}</w:p>",
            runs
        )
    }

    /// 格式化代码块 / Format code block
    fn format_code_block(lines: &[String]) -> String {
        let mut result = String::new();
        for line in lines {
            let escaped = Self::escape_xml(line);
            result.push_str(&format!(
                "<w:p><w:pPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F5F5F5\"/><w:ind w:left=\"360\"/></w:pPr>\
                 <w:r><w:rPr><w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\" w:cs=\"Consolas\"/><w:sz w:val=\"20\"/></w:rPr>\
                 <w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
                escaped
            ));
        }
        result
    }

    /// 格式化标题（支持内联格式）/ Format heading (with inline formatting)
    fn format_heading(text: &str, level: u8) -> String {
        let size = match level {
            1 => "44",
            2 => "36",
            3 => "28",
            4 => "24",
            5 => "22",
            _ => "20",
        };
        let runs = Self::parse_inline_runs(text);
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading{}"/></w:pPr><w:r><w:rPr><w:sz w:val="{}"/><w:b/></w:rPr>{}</w:r></w:p>"#,
            level, size, runs
        )
    }

    /// 处理脚注引用 / Process footnote references
    ///
    /// Converts `[^1]` to superscript format and `[^1]: definition` to footnote paragraphs.
    fn process_footnote_references(line: &str) -> String {
        // 脚注定义行 [^1]: text → 脚注段落 / Footnote definition line
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('[') {
            if let Some(bracket_end) = rest.find("]:") {
                let label = &rest[..bracket_end];
                // 检查是否为脚注标签格式 / Check if footnote label format
                if label.starts_with('^') && label.len() > 1 {
                    let footnote_text = &rest[bracket_end + 2..].trim_start();
                    let num = &label[1..];
                    return format!("[{}] {}", num, footnote_text);
                }
            }
        }

        // 行内脚注引用 [^1] → 上标 / Inline footnote reference → superscript
        let mut result = line.to_string();
        static FOOTNOTE_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let re = FOOTNOTE_RE.get_or_init(|| {
            regex::Regex::new(r"\[\^(\d+)\]").expect("footnote regex should compile")
        });
        result = re.replace_all(&result, "|$1|").to_string();
        result
    }

    /// 生成 styles.xml（字体和默认样式定义）/ Generate styles.xml (font and default style definitions)
    fn generate_styles_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
          xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:docDefaults>
    <w:rPrDefault>
      <w:rPr>
        <w:rFonts w:ascii="Microsoft YaHei" w:eastAsia="Microsoft YaHei" w:hAnsi="Segoe UI" w:cs="Times New Roman"/>
        <w:sz w:val="22"/>
        <w:szCs w:val="22"/>
        <w:lang w:val="zh-CN" w:eastAsia="zh-CN" w:bidi="ar-SA"/>
      </w:rPr>
    </w:rPrDefault>
    <w:pPrDefault>
      <w:pPr>
        <w:spacing w:after="160" w:line="259" w:lineRule="auto"/>
      </w:pPr>
    </w:pPrDefault>
  </w:docDefaults>
  <w:style w:type="paragraph" w:styleId="Heading1">
    <w:name w:val="heading 1"/>
    <w:pPr><w:spacing w:before="480" w:after="120"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="44"/><w:szCs w:val="44"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading2">
    <w:name w:val="heading 2"/>
    <w:pPr><w:spacing w:before="360" w:after="80"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="36"/><w:szCs w:val="36"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading3">
    <w:name w:val="heading 3"/>
    <w:pPr><w:spacing w:before="240" w:after="60"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="28"/><w:szCs w:val="28"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading4">
    <w:name w:val="heading 4"/>
    <w:rPr><w:b/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading5">
    <w:name w:val="heading 5"/>
    <w:rPr><w:b/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading6">
    <w:name w:val="heading 6"/>
    <w:rPr><w:b/><w:sz w:val="20"/><w:szCs w:val="20"/></w:rPr>
  </w:style>
</w:styles>"#.to_string()
    }

    /// 转义 XML 特殊字符 / Escape XML special characters
    fn escape_xml(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}
