//! PDF 导出 / PDF Export
//!
//! PDF 导出使用系统字体支持中文 / PDF export uses system fonts for Chinese support

use super::shared::*;
use printpdf::*;
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::Path;

/// 系统中文字体路径 / System Chinese Font Paths
fn get_system_cjk_font_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
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

/// 导出为 PDF（改进版）/ Export to PDF (Improved)
pub fn export_to_pdf(content: &str, output_path: &Path) -> Result<(), ExportError> {
    export_to_pdf_with_config(content, output_path, PdfExportConfig::default())
}

/// 使用自定义配置导出为 PDF / Export to PDF with custom configuration
pub fn export_to_pdf_with_config(
    content: &str,
    output_path: &Path,
    config: PdfExportConfig,
) -> Result<(), ExportError> {
    let page_width = Mm(config.page_width);
    let page_height = Mm(config.page_height);

    let (doc, page1, layer1) =
        PdfDocument::new("Markdown Export", page_width, page_height, "Layer 1");

    let has_cjk = contains_cjk(content);

    let font = if has_cjk {
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
    let content_width_mm = config.page_width - (config.margin * 2.0);

    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y_pos = config.page_height - config.margin;
    let mut page_count = 1;
    let mut in_code_block = false;
    let mut code_block_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if in_code_block {
                in_code_block = false;
                render_code_block(
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
                y_pos -= line_height * 0.5;
            } else {
                in_code_block = true;
                code_block_lines.clear();
            }
            continue;
        }

        if in_code_block {
            code_block_lines.push(line.to_string());
            continue;
        }

        if line.is_empty() {
            y_pos -= line_height;
            continue;
        }

        let (text, font_size_override, _is_bold) = process_markdown_line_v2(line);
        let current_font_size = font_size_override.unwrap_or(config.font_size);

        let wrapped_lines = wrap_text_by_width(&text, content_width_mm, current_font_size);

        for wrapped_line in wrapped_lines {
            if y_pos < config.margin + current_font_size {
                let (new_page, new_layer) = doc.add_page(page_width, page_height, "Layer 1");
                current_layer = doc.get_page(new_page).get_layer(new_layer);
                y_pos = config.page_height - config.margin;
                page_count += 1;

                let page_text = format!("- {} -", page_count);
                current_layer.use_text(
                    &page_text,
                    9.0,
                    Mm((config.page_width - 20.0) / 2.0),
                    Mm(config.margin / 2.0),
                    &font,
                );
            }

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

    if in_code_block && !code_block_lines.is_empty() {
        render_code_block(
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
    let code_width_mm = config.page_width - (config.margin * 2.0) - 6.0;

    for code_line in code.lines() {
        let wrapped = if code_line.is_empty() {
            vec![" ".to_string()]
        } else {
            wrap_text_by_width(code_line, code_width_mm, code_font_size)
        };

        for display_line in &wrapped {
            if *y_pos < config.margin + code_font_size {
                let (new_page, new_layer) = doc.add_page(page_width, page_height, "Layer 1");
                *current_layer = doc.get_page(new_page).get_layer(new_layer);
                *y_pos = config.page_height - config.margin;
                *page_count += 1;
            }

            current_layer.use_text(
                display_line,
                code_font_size,
                Mm(config.margin + 3.0),
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

    let safe_max_width = max_width_mm * 0.9;

    for ch in text.chars() {
        let char_w = if is_cjk_char(ch) {
            font_size_pt * 0.3528 * 1.0
        } else if ch.is_ascii() {
            let ratio = match ch {
                'i' | 'l' | '!' | '.' | ',' | ':' | ';' | '\'' | '`' => 0.30,
                'I' | 'f' | 'j' | '(' | ')' | '[' | ']' | '{' | '}' | '"' => 0.38,
                't' | 'r' | '1' | '/' | '\\' => 0.40,
                'W' | 'M' | 'm' | '@' | 'O' | 'Q' => 0.72,
                'w' | 'G' | 'C' | 'D' | 'H' | 'N' | 'R' | 'S' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 0.60,
                ' ' => 0.25,
                _ => 0.52,
            };
            font_size_pt * 0.3528 * ratio
        } else {
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

/// 处理 Markdown 行 / Process Markdown line
fn process_markdown_line_v2(line: &str) -> (String, Option<f32>, bool) {
    let trimmed = line.trim_start();

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

    let stripped_line = strip_inline_markdown(line);
    let trimmed_stripped = stripped_line.trim_start();

    if let Some(rest) = trimmed_stripped.strip_prefix("- [ ] ") {
        return (format!("\u{2610} {}", rest), None, false);
    }
    if let Some(rest) = trimmed_stripped
        .strip_prefix("- [x] ")
        .or_else(|| trimmed_stripped.strip_prefix("- [X] "))
    {
        return (format!("\u{2611} {}", rest), None, false);
    }

    if let Some(stripped) = trimmed_stripped
        .strip_prefix("- ")
        .or_else(|| trimmed_stripped.strip_prefix("* "))
        .or_else(|| trimmed_stripped.strip_prefix("+ "))
    {
        return (format!("\u{2022} {}", stripped), None, false);
    }

    let leading_digits: String = trimmed_stripped
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if !leading_digits.is_empty()
        && trimmed_stripped.get(leading_digits.len()..leading_digits.len() + 2) == Some(". ")
    {
        return (trimmed_stripped.to_string(), None, false);
    }

    if trimmed_stripped.starts_with('|') && trimmed_stripped.ends_with('|') {
        let inner = &trimmed_stripped[1..trimmed_stripped.len() - 1];
        let cells: Vec<&str> = inner.split('|').collect();
        if cells.iter().all(|c| {
            c.trim()
                .chars()
                .all(|ch| ch == '-' || ch == ':' || ch == ' ')
        }) {
            return (String::new(), None, false);
        }
        let formatted: String = cells
            .iter()
            .map(|c| c.trim())
            .collect::<Vec<&str>>()
            .join(" \u{2502} ");
        return (formatted, None, false);
    }

    if let Some(stripped) = trimmed_stripped.strip_prefix("> ") {
        return (format!("> {}", stripped), None, false);
    }

    if trimmed_stripped == "---" || trimmed_stripped == "***" || trimmed_stripped == "___" {
        return ("\u{2500}".repeat(50), None, false);
    }

    (stripped_line, None, false)
}

/// 去除内联 Markdown 格式标记 / Strip inline Markdown formatting markers
fn strip_inline_markdown(text: &str) -> String {
    let mut result = text.to_string();

    for _ in 0..10 {
        let prev = result.clone();
        result = strip_pair(&result, "**", "**");
        result = strip_pair(&result, "__", "__");
        if result == prev {
            break;
        }
    }

    for _ in 0..10 {
        let prev = result.clone();
        result = strip_pair(&result, "*", "*");
        if result == prev {
            break;
        }
    }

    result = strip_pair(&result, "`", "`");
    result = strip_pair(&result, "~~", "~~");
    result = strip_bracket_link(&result, true);
    result = strip_bracket_link(&result, false);

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
        let remaining: String = chars[i..].iter().collect();
        if remaining.starts_with(prefix) {
            let bracket_start = i + prefix.chars().count();
            if let Some(bracket_offset) = chars[bracket_start..].iter().position(|&c| c == ']') {
                let bracket_end = bracket_start + bracket_offset;
                let inner: String = chars[bracket_start..bracket_end].iter().collect();
                let after_bracket = bracket_end + 1;
                if after_bracket < char_len && chars[after_bracket] == '(' {
                    let paren_content: String = chars[after_bracket..].iter().collect();
                    if let Some(paren_offset) = paren_content.find(')') {
                        if is_image {
                            result.push('[');
                            result.push_str(&inner);
                            result.push(']');
                        } else {
                            result.push_str(&inner);
                        }
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
