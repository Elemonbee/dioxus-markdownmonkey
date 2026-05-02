//! DOCX (Word) 导出 / DOCX (Word) Export

use super::shared::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// 导出为 Word/Docx 格式 / Export to Word/Docx format
pub fn export_to_docx(content: &str, output_path: &Path) -> Result<(), ExportError> {
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let document_xml = generate_document_xml(content);

    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types.as_bytes())?;

    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels.as_bytes())?;

    let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;
    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(doc_rels.as_bytes())?;

    let styles_xml = generate_styles_xml();
    zip.start_file("word/styles.xml", options)?;
    zip.write_all(styles_xml.as_bytes())?;

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
    let mut table_rows: Vec<Vec<String>> = Vec::new();

    let flush_table = |table_rows: &mut Vec<Vec<String>>, paragraphs: &mut String| {
        if !table_rows.is_empty() {
            paragraphs.push_str(&format_table_ooxml(table_rows));
            table_rows.clear();
        }
    };

    for line in content.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") {
            flush_table(&mut table_rows, &mut paragraphs);
            if in_code_block {
                in_code_block = false;
                paragraphs.push_str(&format_code_block(&code_lines));
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

        if trimmed.starts_with('|')
            && trimmed.ends_with('|')
            && trimmed.contains('|')
            && trimmed.len() >= 2
        {
            let inner = &trimmed[1..trimmed.len() - 1];
            let cells: Vec<&str> = inner.split('|').collect();
            let is_separator = cells.iter().all(|c| {
                c.trim()
                    .chars()
                    .all(|ch| ch == '-' || ch == ':' || ch == ' ')
            });
            if !is_separator {
                table_rows.push(cells.iter().map(|c| c.trim().to_string()).collect());
            }
            continue;
        }

        flush_table(&mut table_rows, &mut paragraphs);

        let para_xml = if let Some(stripped) = trimmed.strip_prefix("# ") {
            format_heading(stripped, 1)
        } else if let Some(stripped) = trimmed.strip_prefix("## ") {
            format_heading(stripped, 2)
        } else if let Some(stripped) = trimmed.strip_prefix("### ") {
            format_heading(stripped, 3)
        } else if let Some(stripped) = trimmed.strip_prefix("#### ") {
            format_heading(stripped, 4)
        } else if let Some(stripped) = trimmed.strip_prefix("##### ") {
            format_heading(stripped, 5)
        } else if let Some(stripped) = trimmed.strip_prefix("###### ") {
            format_heading(stripped, 6)
        } else if trimmed.starts_with("- [ ] ")
            || trimmed.starts_with("- [x] ")
            || trimmed.starts_with("- [X] ")
        {
            let checked = trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ");
            let text = &trimmed[6..];
            let marker = if checked { "\u{2611} " } else { "\u{2610} " };
            format_rich_paragraph(&format!("{}{}", marker, text))
        } else if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
        {
            let text = trimmed[2..].to_string();
            format_rich_paragraph_with_indent(&format!("\u{2022} {}", text), 360)
        } else if let Some(text) = trimmed.strip_prefix("> ") {
            format_quote(text)
        } else if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            r#"<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="6" w:space="1" w:color="auto"/></w:pBdr></w:pPr></w:p>"#.to_string()
        } else if !line.is_empty() {
            let processed = process_footnote_references(line);
            format_rich_paragraph(&processed)
        } else {
            "<w:p/>".to_string()
        };
        paragraphs.push_str(&para_xml);
    }

    flush_table(&mut table_rows, &mut paragraphs);

    if in_code_block && !code_lines.is_empty() {
        paragraphs.push_str(&format_code_block(&code_lines));
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

/// 生成 OOXML 表格 / Generate OOXML table
fn format_table_ooxml(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let mut xml = String::from("<w:tbl>\n");
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
            let runs = parse_inline_runs(cell);
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
fn parse_inline_runs(text: &str) -> String {
    let mut runs = String::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.starts_with("**") && remaining.len() > 4 {
            if let Some(end) = find_closing(&remaining[2..], "**") {
                let inner = &remaining[2..2 + end];
                runs.push_str(&format_run(inner, true, false, false, false));
                remaining = &remaining[2 + end + 2..];
                continue;
            }
        }

        if remaining.starts_with("__") && remaining.len() > 4 {
            if let Some(end) = find_closing(&remaining[2..], "__") {
                let inner = &remaining[2..2 + end];
                runs.push_str(&format_run(inner, true, false, false, false));
                remaining = &remaining[2 + end + 2..];
                continue;
            }
        }

        if remaining.starts_with("`") {
            if let Some(end) = remaining[1..].find('`') {
                let inner = &remaining[1..1 + end];
                runs.push_str(&format_run(inner, false, false, true, false));
                remaining = &remaining[1 + end + 1..];
                continue;
            }
        }

        if remaining.starts_with("~~") && remaining.len() > 4 {
            if let Some(end) = find_closing(&remaining[2..], "~~") {
                let inner = &remaining[2..2 + end];
                runs.push_str(&format_run(inner, false, false, false, true));
                remaining = &remaining[2 + end + 2..];
                continue;
            }
        }

        if remaining.starts_with("*") && !remaining.starts_with("**") {
            if let Some(end) = remaining[1..].find('*') {
                let inner = &remaining[1..1 + end];
                runs.push_str(&format_run(inner, false, true, false, false));
                remaining = &remaining[1 + end + 1..];
                continue;
            }
        }

        if remaining.starts_with("[") {
            if let Some(bracket_end) = remaining[1..].find(']') {
                let inner = &remaining[1..1 + bracket_end];
                let after = &remaining[1 + bracket_end + 1..];
                if let Some(after) = after.strip_prefix("(") {
                    if let Some(paren_end) = after.find(')') {
                        runs.push_str(&format_run(inner, false, false, false, false));
                        runs.push_str(&format_run(
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

        let special = ['*', '`', '~', '[', '_'];
        let first_char_len = remaining.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        let end_pos = remaining[first_char_len..]
            .find(|c: char| special.contains(&c))
            .map(|p| first_char_len + p)
            .unwrap_or(remaining.len());
        let plain = &remaining[..end_pos];
        runs.push_str(&format_run(plain, false, false, false, false));
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
            rpr.push_str("<w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\" w:cs=\"Consolas\"/>");
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
        escape_xml(text)
    )
}

/// 格式化带内联格式的段落 / Format paragraph with inline formatting
fn format_rich_paragraph(text: &str) -> String {
    let runs = parse_inline_runs(text);
    format!("<w:p>{}</w:p>", runs)
}

/// 格式化带缩进的内联段落 / Format paragraph with indent and inline formatting
fn format_rich_paragraph_with_indent(text: &str, indent_twips: i32) -> String {
    let runs = parse_inline_runs(text);
    format!(
        "<w:p><w:pPr><w:ind w:left=\"{}\"/></w:pPr>{}</w:p>",
        indent_twips, runs
    )
}

/// 格式化引用段落 / Format quote paragraph
fn format_quote(text: &str) -> String {
    let runs = parse_inline_runs(text);
    format!(
        "<w:p><w:pPr><w:pBdr><w:left w:val=\"single\" w:sz=\"12\" w:space=\"4\" w:color=\"999999\"/></w:pBdr><w:ind w:left=\"360\"/></w:pPr>{}</w:p>",
        runs
    )
}

/// 格式化代码块 / Format code block
fn format_code_block(lines: &[String]) -> String {
    let mut result = String::new();
    for line in lines {
        let escaped = escape_xml(line);
        result.push_str(&format!(
            "<w:p><w:pPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F5F5F5\"/><w:ind w:left=\"360\"/></w:pPr>\
             <w:r><w:rPr><w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\" w:cs=\"Consolas\"/><w:sz w:val=\"20\"/></w:rPr>\
             <w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
            escaped
        ));
    }
    result
}

/// 格式化标题 / Format heading
fn format_heading(text: &str, level: u8) -> String {
    let size = match level {
        1 => "44",
        2 => "36",
        3 => "28",
        4 => "24",
        5 => "22",
        _ => "20",
    };
    let runs = parse_inline_runs(text);
    format!(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading{}"/></w:pPr><w:r><w:rPr><w:sz w:val="{}"/><w:b/></w:rPr>{}</w:r></w:p>"#,
        level, size, runs
    )
}

/// 处理脚注引用 / Process footnote references
fn process_footnote_references(line: &str) -> String {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix('[') {
        if let Some(bracket_end) = rest.find("]:") {
            let label = &rest[..bracket_end];
            if label.starts_with('^') && label.len() > 1 {
                let footnote_text = &rest[bracket_end + 2..].trim_start();
                let num = &label[1..];
                return format!("[{}] {}", num, footnote_text);
            }
        }
    }

    let mut result = line.to_string();
    static FOOTNOTE_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = FOOTNOTE_RE.get_or_init(|| {
        regex::Regex::new(r"\[\^(\d+)\]").unwrap_or_else(|e| {
            tracing::error!(
                "脚注正则编译失败，使用空匹配回退 / Footnote regex compile failed, using empty fallback: {}",
                e
            );
            regex::Regex::new("$^").unwrap_or_else(|_| regex::Regex::new("a^").unwrap())
        })
    });
    result = re.replace_all(&result, "|$1|").to_string();
    result
}

/// 生成 styles.xml / Generate styles.xml
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
