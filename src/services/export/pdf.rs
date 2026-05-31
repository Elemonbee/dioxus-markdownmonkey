//! PDF 导出 / PDF Export
//!
//! PDF 导出使用系统字体支持中文 / PDF export uses system fonts for Chinese support

use super::shared::*;
use printpdf::{
    BuiltinFont, FontId, Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions,
    PdfWarnMsg, Point, Pt, TextItem,
};
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::Path;

#[derive(Clone)]
enum ActiveFont {
    Builtin(BuiltinFont),
    External(FontId),
}

impl ActiveFont {
    fn handle(&self) -> PdfFontHandle {
        match self {
            Self::Builtin(font) => PdfFontHandle::Builtin(*font),
            Self::External(id) => PdfFontHandle::External(id.clone()),
        }
    }
}

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

/// 导出为 PDF / Export to PDF
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
    let mut doc = PdfDocument::new("Markdown Export");
    let font = resolve_font(&mut doc, content);

    let mut pages = Vec::new();
    let mut ops = start_page_ops();
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
                    &mut pages,
                    &mut ops,
                    &font,
                    &mut y_pos,
                    &mut page_count,
                    &config,
                    page_width,
                    page_height,
                );
                code_block_lines.clear();
                y_pos -= config.font_size * config.line_height_multiplier * 0.5;
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

        let line_height = config.font_size * config.line_height_multiplier;
        if line.is_empty() {
            y_pos -= line_height;
            ensure_page_space(
                &mut pages,
                &mut ops,
                &mut y_pos,
                &mut page_count,
                line_height,
                &config,
                page_width,
                page_height,
            );
            continue;
        }

        let (text, font_size_override, _is_bold) = process_markdown_line_v2(line);
        let current_font_size = font_size_override.unwrap_or(config.font_size);
        write_wrapped_text(
            &text,
            config.margin,
            current_font_size,
            &mut pages,
            &mut ops,
            &font,
            &mut y_pos,
            &mut page_count,
            &config,
            page_width,
            page_height,
        );
    }

    if in_code_block && !code_block_lines.is_empty() {
        render_code_block(
            &code_block_lines.join("\n"),
            &mut pages,
            &mut ops,
            &font,
            &mut y_pos,
            &mut page_count,
            &config,
            page_width,
            page_height,
        );
    }

    finish_page(&mut pages, &mut ops, page_width, page_height);
    doc.with_pages(pages);

    let mut warnings: Vec<PdfWarnMsg> = Vec::new();
    let mut writer = BufWriter::new(File::create(output_path)?);
    doc.save_writer(&mut writer, &PdfSaveOptions::default(), &mut warnings);
    for warning in warnings {
        tracing::warn!("PDF export warning: {:?}", warning);
    }

    tracing::info!(
        "PDF 导出完成，共 {} 页 / PDF export completed, {} pages",
        page_count,
        page_count
    );
    Ok(())
}

fn resolve_font(doc: &mut PdfDocument, content: &str) -> ActiveFont {
    if contains_cjk(content) {
        if let Some((path, font_data)) = find_available_system_font() {
            let mut warnings = Vec::new();
            if let Some(parsed) = ParsedFont::from_bytes(&font_data, 0, &mut warnings) {
                for warning in warnings {
                    tracing::warn!("PDF font parse warning: {:?}", warning);
                }
                let id = doc.add_font(&parsed);
                return ActiveFont::External(id);
            }

            tracing::warn!(
                "系统字体无法被 printpdf 解析，回退到 Helvetica: {:?} / \
                 System font could not be parsed by printpdf, falling back to Helvetica: {:?}",
                path,
                path
            );
        }
    }

    ActiveFont::Builtin(BuiltinFont::Helvetica)
}

fn start_page_ops() -> Vec<Op> {
    vec![Op::StartTextSection]
}

fn finish_page(pages: &mut Vec<PdfPage>, ops: &mut Vec<Op>, page_width: Mm, page_height: Mm) {
    ops.push(Op::EndTextSection);
    let page_ops = std::mem::replace(ops, start_page_ops());
    pages.push(PdfPage::new(page_width, page_height, page_ops));
}

#[allow(clippy::too_many_arguments)]
fn ensure_page_space(
    pages: &mut Vec<PdfPage>,
    ops: &mut Vec<Op>,
    y_pos: &mut f32,
    page_count: &mut usize,
    required_height: f32,
    config: &PdfExportConfig,
    page_width: Mm,
    page_height: Mm,
) {
    if *y_pos >= config.margin + required_height {
        return;
    }

    finish_page(pages, ops, page_width, page_height);
    *y_pos = config.page_height - config.margin;
    *page_count += 1;
}

#[allow(clippy::too_many_arguments)]
fn write_wrapped_text(
    text: &str,
    x_mm: f32,
    font_size: f32,
    pages: &mut Vec<PdfPage>,
    ops: &mut Vec<Op>,
    font: &ActiveFont,
    y_pos: &mut f32,
    page_count: &mut usize,
    config: &PdfExportConfig,
    page_width: Mm,
    page_height: Mm,
) {
    let content_width_mm = config.page_width - x_mm - config.margin;
    let wrapped_lines = wrap_text_by_width(text, content_width_mm, font_size);
    let line_height = font_size * config.line_height_multiplier;

    for wrapped_line in wrapped_lines {
        ensure_page_space(
            pages,
            ops,
            y_pos,
            page_count,
            line_height,
            config,
            page_width,
            page_height,
        );

        ops.push(Op::SetFont {
            font: font.handle(),
            size: Pt(font_size),
        });
        ops.push(Op::SetLineHeight {
            lh: Pt(line_height),
        });
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(x_mm), Mm(*y_pos)),
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(wrapped_line)],
        });

        *y_pos -= line_height;
    }
}

/// 渲染代码块（自动换行而非截断）/ Render code block (auto-wrap instead of truncate)
#[allow(clippy::too_many_arguments)]
fn render_code_block(
    code: &str,
    pages: &mut Vec<PdfPage>,
    ops: &mut Vec<Op>,
    font: &ActiveFont,
    y_pos: &mut f32,
    page_count: &mut usize,
    config: &PdfExportConfig,
    page_width: Mm,
    page_height: Mm,
) {
    let code_font_size = config.font_size * 0.85;

    for code_line in code.lines() {
        let display = if code_line.is_empty() { " " } else { code_line };
        write_wrapped_text(
            display,
            config.margin + 3.0,
            code_font_size,
            pages,
            ops,
            font,
            y_pos,
            page_count,
            config,
            page_width,
            page_height,
        );
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
