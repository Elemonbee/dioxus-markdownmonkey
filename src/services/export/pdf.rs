//! PDF 导出 / PDF Export
//!
//! PDF 导出使用系统字体支持中文 / PDF export uses system fonts for Chinese support

use super::shared::*;
use printpdf::{
    BuiltinFont, FontId, Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions,
    PdfWarnMsg, Point, Pt, RawImage, TextItem, XObjectTransform,
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
fn get_system_cjk_font_paths(preferred: Option<&str>) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    // 设置优先，其次环境变量 / Settings first, then env override
    if let Some(custom) = preferred {
        let p = std::path::PathBuf::from(custom.trim());
        if !p.as_os_str().is_empty() {
            paths.push(p);
        }
    }
    if let Ok(custom) = std::env::var("MARKDOWNMONKEY_PDF_FONT") {
        let p = std::path::PathBuf::from(custom.trim());
        if !p.as_os_str().is_empty() {
            paths.push(p);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
        let fonts = std::path::PathBuf::from(&windir).join("Fonts");
        for name in [
            "msyh.ttc",
            "msyh.ttf",
            "msyhbd.ttc",
            "simhei.ttf",
            "simsun.ttc",
            "simsun.ttf",
            "msjh.ttc",
            "seguiemj.ttf",
        ] {
            paths.push(fonts.join(name));
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let user_fonts = std::path::PathBuf::from(local).join("Microsoft\\Windows\\Fonts");
            for name in ["msyh.ttc", "msyh.ttf", "NotoSansCJKsc-Regular.otf"] {
                paths.push(user_fonts.join(name));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        for p in [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/Supplemental/Songti.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        ] {
            paths.push(std::path::PathBuf::from(p));
        }
    }

    #[cfg(target_os = "linux")]
    {
        for p in [
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/arphic/uming.ttc",
        ] {
            paths.push(std::path::PathBuf::from(p));
        }
        if let Ok(home) = std::env::var("HOME") {
            let local = std::path::PathBuf::from(home).join(".local/share/fonts");
            for name in ["NotoSansCJK-Regular.ttc", "NotoSansCJKsc-Regular.otf"] {
                paths.push(local.join(name));
            }
        }
    }

    paths
}

/// 尝试解析字体字节（支持 TTC 多索引）
/// Try parsing font bytes (supports multiple TTC face indices)
fn try_parse_font_bytes(font_data: &[u8]) -> Option<ParsedFont> {
    for index in [0usize, 1, 2] {
        let mut warnings = Vec::new();
        if let Some(parsed) = ParsedFont::from_bytes(font_data, index, &mut warnings) {
            for warning in warnings {
                tracing::warn!("PDF font parse warning (index {}): {:?}", index, warning);
            }
            return Some(parsed);
        }
    }
    None
}

/// 查找并可成功解析的系统 CJK 字体 / Find a parseable system CJK font
fn find_parseable_cjk_font(preferred: Option<&str>) -> Option<(std::path::PathBuf, ParsedFont)> {
    let paths = get_system_cjk_font_paths(preferred);

    for path in paths {
        if !path.exists() {
            continue;
        }
        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                tracing::debug!("无法打开字体 {:?}: {}", path, e);
                continue;
            }
        };
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_err() {
            continue;
        }
        if let Some(parsed) = try_parse_font_bytes(&buffer) {
            tracing::info!("使用系统字体/Using system font: {:?}", path);
            return Some((path, parsed));
        }
        tracing::debug!("字体存在但无法解析，尝试下一个: {:?}", path);
    }

    tracing::warn!("未找到可解析的系统中文字体/No parseable system Chinese font found");
    None
}

/// 导出为 PDF / Export to PDF
#[allow(dead_code)] // 经 ExportService 供测试调用 / Reached via ExportService in tests
pub fn export_to_pdf(content: &str, output_path: &Path) -> Result<(), ExportError> {
    export_to_pdf_with_assets(content, output_path, PdfExportConfig::default(), None)
}

/// 使用自定义配置导出为 PDF / Export to PDF with custom configuration
#[allow(dead_code)] // 经 ExportService / with_assets(None) 调用 / Via ExportService / with_assets(None)
pub fn export_to_pdf_with_config(
    content: &str,
    output_path: &Path,
    config: PdfExportConfig,
) -> Result<(), ExportError> {
    export_to_pdf_with_assets(content, output_path, config, None)
}

/// 导出 PDF 并嵌入本地 PNG/JPEG 图片
/// Export PDF and embed local PNG/JPEG images
pub fn export_to_pdf_with_assets(
    content: &str,
    output_path: &Path,
    config: PdfExportConfig,
    source_dir: Option<&Path>,
) -> Result<(), ExportError> {
    let page_width = Mm(config.page_width);
    let page_height = Mm(config.page_height);
    let mut doc = PdfDocument::new("Markdown Export");
    let font = resolve_font(&mut doc, content, config.cjk_font_path.as_deref())?;

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

        // 整行本地图片：尝试嵌入，失败则回退为文字占位
        // Standalone local image: try embed, else fall back to text placeholder
        if let Some((alt, img_path)) = parse_standalone_image(trimmed) {
            if try_place_image(
                &mut doc,
                source_dir,
                &img_path,
                &mut pages,
                &mut ops,
                &mut y_pos,
                &mut page_count,
                &config,
                page_width,
                page_height,
            ) {
                continue;
            }
            let fallback = format!("Image: {} ({})", alt, img_path);
            write_wrapped_text(
                &fallback,
                config.margin,
                config.font_size,
                &mut pages,
                &mut ops,
                &font,
                &mut y_pos,
                &mut page_count,
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

/// 尝试将本地 PNG/JPEG 嵌入当前 PDF 页面
/// Try embedding a local PNG/JPEG onto the current PDF page
#[allow(clippy::too_many_arguments)]
fn try_place_image(
    doc: &mut PdfDocument,
    source_dir: Option<&Path>,
    img_path: &str,
    pages: &mut Vec<PdfPage>,
    ops: &mut Vec<Op>,
    y_pos: &mut f32,
    page_count: &mut usize,
    config: &PdfExportConfig,
    page_width: Mm,
    page_height: Mm,
) -> bool {
    let base = match source_dir {
        Some(b) => b,
        None => return false,
    };
    let abs = match resolve_image_path(base, img_path) {
        Some(p) if p.is_file() => p,
        _ => return false,
    };
    match image_content_type(&abs) {
        Some("image/png") | Some("image/jpeg") => {}
        _ => return false,
    }
    let bytes = match std::fs::read(&abs) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let mut decode_warnings = Vec::new();
    let image = match RawImage::decode_from_bytes(&bytes, &mut decode_warnings) {
        Ok(img) => img,
        Err(e) => {
            tracing::warn!("PDF 图片解码失败 / PDF image decode failed: {}", e);
            return false;
        }
    };
    for w in decode_warnings {
        tracing::debug!("PDF image decode warning: {:?}", w);
    }

    // 96 DPI 下 1px ≈ 0.2646mm；必要时等比缩小以适应内容区
    // At 96 DPI, 1px ≈ 0.2646mm; scale down if needed to fit content area
    const DPI: f32 = 96.0;
    let px_to_mm = 25.4 / DPI;
    let natural_w = (image.width as f32) * px_to_mm;
    let natural_h = (image.height as f32) * px_to_mm;
    let max_w = config.page_width - 2.0 * config.margin;
    let max_h = (config.page_height - 2.0 * config.margin).max(10.0);
    let scale = (max_w / natural_w.max(0.01))
        .min(max_h / natural_h.max(0.01))
        .min(1.0);
    let draw_h = natural_h * scale;
    let gap = config.font_size * 0.5;

    ensure_page_space(
        pages,
        ops,
        y_pos,
        page_count,
        draw_h + gap,
        config,
        page_width,
        page_height,
    );

    let image_id = doc.add_image(&image);
    // 图片必须在文本段之外绘制 / Images must be drawn outside text sections
    ops.push(Op::EndTextSection);
    let translate_y = *y_pos - draw_h;
    ops.push(Op::UseXobject {
        id: image_id,
        transform: XObjectTransform {
            translate_x: Some(Mm(config.margin).into()),
            translate_y: Some(Mm(translate_y).into()),
            rotate: None,
            scale_x: Some(scale),
            scale_y: Some(scale),
            dpi: Some(DPI),
        },
    });
    ops.push(Op::StartTextSection);

    *y_pos -= draw_h + gap;
    true
}

fn resolve_font(
    doc: &mut PdfDocument,
    content: &str,
    preferred_font: Option<&str>,
) -> Result<ActiveFont, ExportError> {
    if contains_cjk(content) {
        if let Some((path, parsed)) = find_parseable_cjk_font(preferred_font) {
            let id = doc.add_font(&parsed);
            tracing::info!("PDF CJK font embedded from {:?}", path);
            return Ok(ActiveFont::External(id));
        }
        return Err(ExportError::Font(
            "未找到可用的中文字体。请在设置中指定字体路径，安装微软雅黑/Noto CJK，或设置环境变量 MARKDOWNMONKEY_PDF_FONT。 / \
             No usable CJK font found. Set a font path in Settings, install Microsoft YaHei / Noto CJK, or set MARKDOWNMONKEY_PDF_FONT."
                .to_string(),
        ));
    }

    Ok(ActiveFont::Builtin(BuiltinFont::Helvetica))
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
                        let target: String = chars[after_bracket + 1..after_bracket + paren_offset]
                            .iter()
                            .collect();
                        if is_image {
                            result.push_str(&format!("Image: {} ({})", inner, target));
                        } else {
                            result.push_str(&inner);
                            result.push_str(&format!(" ({})", target));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// 最小合法 1×1 PNG / Minimal valid 1×1 PNG
    fn tiny_png() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }

    #[test]
    fn strip_inline_markdown_preserves_link_target() {
        assert_eq!(
            strip_inline_markdown("Read [Docs](https://example.com?a=1&b=2)"),
            "Read Docs (https://example.com?a=1&b=2)"
        );
    }

    #[test]
    fn strip_inline_markdown_preserves_image_reference() {
        assert_eq!(
            strip_inline_markdown("Diagram: ![Flow](images/flow.png)"),
            "Diagram: Image: Flow (images/flow.png)"
        );
    }

    /// 嵌入本地 PNG 后 PDF 应大于纯文本导出 / Embedded PNG should make PDF larger than text-only
    #[test]
    fn pdf_embeds_local_png_when_source_dir_set() {
        let dir = TempDir::new().unwrap();
        let img = dir.path().join("pic.png");
        fs::write(&img, tiny_png()).unwrap();

        let md = "# Title\n\n![Pic](pic.png)\n\nHello\n";
        let with_img = dir.path().join("with_img.pdf");
        let text_only = dir.path().join("text_only.pdf");

        export_to_pdf_with_assets(md, &with_img, PdfExportConfig::default(), Some(dir.path()))
            .expect("pdf with image");
        export_to_pdf_with_assets(
            "# Title\n\nImage: Pic (pic.png)\n\nHello\n",
            &text_only,
            PdfExportConfig::default(),
            None,
        )
        .expect("pdf text only");

        let a = fs::metadata(&with_img).unwrap().len();
        let b = fs::metadata(&text_only).unwrap().len();
        assert!(
            a > b,
            "embedded image PDF ({a}) should be larger than text-only ({b})"
        );
    }

    /// 缺少图片文件时回退为文字占位且不报错 / Missing image falls back to text without error
    #[test]
    fn pdf_missing_image_falls_back_to_text() {
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("out.pdf");
        let md = "![Missing](nope.png)\n";
        export_to_pdf_with_assets(md, &out, PdfExportConfig::default(), Some(dir.path()))
            .expect("should succeed with fallback");
        assert!(out.is_file());
        assert!(fs::metadata(&out).unwrap().len() > 100);
    }
}
