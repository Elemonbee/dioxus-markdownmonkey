//! Markdown 渲染服务：pulldown-cmark + 自写 URL/原始 HTML 过滤
//! Markdown rendering: pulldown-cmark plus a small URL / raw-HTML filter

use pulldown_cmark::{
    html, CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::highlight;

/// 判断 URL 是否危险（XSS 载体）
/// Whether a URL is a dangerous XSS vector
/// 允许的光栅图 data URI MIME（不含 SVG，避免内嵌脚本）
/// Allowed raster data-URI MIME types (no SVG, which can embed scripts)
const SAFE_DATA_IMAGE_PREFIXES: &[&str] = &[
    "data:image/png",
    "data:image/jpeg",
    "data:image/jpg",
    "data:image/gif",
    "data:image/webp",
    "data:image/bmp",
    "data:image/x-icon",
    "data:image/vnd.microsoft.icon",
];

/// 判断 URL 是否危险（XSS 载体）
/// Whether a URL is a dangerous XSS vector
fn is_dangerous_url(value: &str) -> bool {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("vbscript:")
        || lower.starts_with("blob:")
    {
        return true;
    }
    if lower.starts_with("data:") {
        return !is_safe_data_image_url(&lower);
    }
    false
}

/// 仅放行常见光栅图 data URI，拦截 SVG 与其它 data: 载体
/// Allow common raster data URIs only; reject SVG and other data: payloads
fn is_safe_data_image_url(lower: &str) -> bool {
    SAFE_DATA_IMAGE_PREFIXES.iter().any(|prefix| {
        lower.starts_with(prefix)
            && lower.get(prefix.len()..).is_some_and(|rest| {
                rest.starts_with(';') || rest.starts_with(',') || rest.is_empty()
            })
    })
}

/// 清洗链接/图片目标，去掉危险 scheme
/// Sanitize link/image destinations by stripping dangerous schemes
fn sanitize_url(url: CowStr<'_>) -> CowStr<'_> {
    if is_dangerous_url(&url) {
        CowStr::Borrowed("")
    } else {
        url
    }
}

/// Markdown 服务 / Markdown Service
pub struct MarkdownService;

impl MarkdownService {
    /// 创建新实例 / Create a new instance
    pub fn new() -> Self {
        Self
    }

    /// 渲染 Markdown 为消毒后的 HTML
    /// Render Markdown to sanitized HTML
    pub fn render(&self, content: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_MATH);

        let sanitized = Parser::new_ext(content, options)
            .into_offset_iter()
            .filter_map(|(event, range)| sanitize_event(event).map(|event| (event, range)));
        let events = rewrite_code_blocks_and_source_lines(content, sanitized);
        let mut html_output = String::new();
        for event in events {
            match event {
                Event::Html(raw) => html_output.push_str(&raw),
                other => html::push_html(&mut html_output, std::iter::once(other)),
            }
        }
        html_output
    }
}

/// 字节偏移对应的 0 基行号 / 0-based line number for a byte offset
fn byte_to_line(content: &str, byte: usize) -> usize {
    let end = byte.min(content.len());
    content[..end].bytes().filter(|&b| b == b'\n').count()
}

/// 给首个开标签写入 data-source-line / Write data-source-line onto the first open tag
fn with_source_line(html: &str, line: usize) -> String {
    let Some(gt) = html.find('>') else {
        return html.to_string();
    };
    if html[..gt].contains("data-source-line") {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len() + 28);
    out.push_str(&html[..gt]);
    out.push_str(&format!(" data-source-line=\"{line}\""));
    out.push_str(&html[gt..]);
    out
}

/// 标题级别数字 / Numeric heading level
fn heading_level_num(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// 把代码块换成高亮 HTML，并为块级标签补上源行
/// Replace code fences with highlighted HTML and tag block elements with source lines
fn rewrite_code_blocks_and_source_lines<'a, I>(content: &str, events: I) -> Vec<Event<'a>>
where
    I: Iterator<Item = (Event<'a>, std::ops::Range<usize>)>,
{
    let mut out = Vec::new();
    let mut in_code = false;
    let mut language = String::new();
    let mut buffer = String::new();
    let mut code_line = 0usize;

    for (event, range) in events {
        if in_code {
            match event {
                Event::Text(text) | Event::Code(text) => buffer.push_str(&text),
                Event::SoftBreak | Event::HardBreak => buffer.push('\n'),
                Event::End(TagEnd::CodeBlock) => {
                    in_code = false;
                    let html = with_source_line(
                        &highlight::render_highlighted_block(&language, &buffer),
                        code_line,
                    );
                    out.push(Event::Html(html.into()));
                    language.clear();
                    buffer.clear();
                }
                _ => {}
            }
            continue;
        }

        let line = byte_to_line(content, range.start);
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                code_line = line;
                buffer.clear();
                language = match kind {
                    CodeBlockKind::Fenced(info) => {
                        info.split_whitespace().next().unwrap_or("").to_string()
                    }
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::Start(Tag::Heading { level, id, .. }) => {
                let n = heading_level_num(level);
                let mut open = format!("<h{n} data-source-line=\"{line}\"");
                if let Some(id) = id {
                    open.push_str(" id=\"");
                    open.push_str(&highlight::escape_attr(&id));
                    open.push('"');
                }
                open.push('>');
                out.push(Event::Html(open.into()));
            }
            Event::End(TagEnd::Heading(level)) => {
                out.push(Event::Html(
                    format!("</h{}>", heading_level_num(level)).into(),
                ));
            }
            Event::Start(Tag::Paragraph) => {
                out.push(Event::Html(
                    format!("<p data-source-line=\"{line}\">").into(),
                ));
            }
            Event::End(TagEnd::Paragraph) => {
                out.push(Event::Html("</p>".into()));
            }
            Event::Start(Tag::BlockQuote(_)) => {
                out.push(Event::Html(
                    format!("<blockquote data-source-line=\"{line}\">").into(),
                ));
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                out.push(Event::Html("</blockquote>".into()));
            }
            Event::Start(Tag::Item) => {
                out.push(Event::Html(
                    format!("<li data-source-line=\"{line}\">").into(),
                ));
            }
            Event::End(TagEnd::Item) => {
                out.push(Event::Html("</li>".into()));
            }
            Event::Start(Tag::List(start)) => {
                let open = match start {
                    None => format!("<ul data-source-line=\"{line}\">"),
                    Some(n) => format!("<ol start=\"{n}\" data-source-line=\"{line}\">"),
                };
                out.push(Event::Html(open.into()));
            }
            Event::End(TagEnd::List(ordered)) => {
                out.push(Event::Html(if ordered { "</ol>" } else { "</ul>" }.into()));
            }
            Event::Start(Tag::Table(_)) => {
                out.push(Event::Html(
                    format!("<table data-source-line=\"{line}\">").into(),
                ));
            }
            Event::End(TagEnd::Table) => {
                out.push(Event::Html("</table>".into()));
            }
            other => out.push(other),
        }
    }
    out
}

impl Default for MarkdownService {
    fn default() -> Self {
        Self::new()
    }
}

/// 过滤原始 HTML，并清洗链接/图片 URL
/// Drop raw HTML and sanitize link/image URLs
fn sanitize_event(event: Event<'_>) -> Option<Event<'_>> {
    match event {
        Event::Html(_) | Event::InlineHtml(_) => None,
        Event::Start(Tag::HtmlBlock) | Event::End(pulldown_cmark::TagEnd::HtmlBlock) => None,
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Some(Event::Start(Tag::Link {
            link_type,
            dest_url: sanitize_url(dest_url),
            title,
            id,
        })),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Some(Event::Start(Tag::Image {
            link_type,
            dest_url: sanitize_url(dest_url),
            title,
            id,
        })),
        other => Some(other),
    }
}

/// 便捷函数：渲染 Markdown / Convenience function: render Markdown
#[allow(dead_code)] // 测试入口 / Test entry
pub fn render_markdown(content: &str) -> String {
    MarkdownService::new().render(content)
}

/// 将预览里的本地图片改写成 file://，并限制在文档/工作区目录内
/// Rewrite local preview images to file:// URLs, constrained to the document/workspace
pub fn rewrite_local_preview_images(
    html: &str,
    source_dir: Option<&Path>,
    extra_roots: &[&Path],
) -> String {
    let Some(base) = source_dir.filter(|path| path.is_dir()) else {
        return html.to_string();
    };
    static IMG_SRC_RE: OnceLock<Regex> = OnceLock::new();
    let re = IMG_SRC_RE.get_or_init(|| {
        Regex::new(r#"(?i)(<img\b[^>]*?\bsrc\s*=\s*)(?:"([^"]+)"|'([^']+)')"#)
            .expect("preview img src regex")
    });

    let mut allowed: Vec<PathBuf> = Vec::new();
    if let Ok(canon) = base.canonicalize() {
        allowed.push(canon);
    }
    for root in extra_roots {
        if let Ok(canon) = root.canonicalize() {
            allowed.push(canon);
        }
    }
    if allowed.is_empty() {
        return html.to_string();
    }

    let mut out = String::with_capacity(html.len());
    let mut last = 0;
    for cap in re.captures_iter(html) {
        let full = cap.get(0).unwrap();
        let prefix = cap.get(1).unwrap().as_str();
        let (src, quote) = if let Some(double) = cap.get(2) {
            (double.as_str(), "\"")
        } else if let Some(single) = cap.get(3) {
            (single.as_str(), "'")
        } else {
            continue;
        };
        out.push_str(&html[last..full.start()]);
        last = full.end();
        if is_remote_preview_url(src) {
            out.push_str(full.as_str());
            continue;
        }
        match resolve_allowed_preview_image(base, src, &allowed) {
            Some(url) => {
                out.push_str(prefix);
                out.push_str(quote);
                out.push_str(&url);
                out.push_str(quote);
            }
            None => out.push_str(full.as_str()),
        }
    }
    out.push_str(&html[last..]);
    out
}

/// 是否为应保持原样的远程/内联地址 / Whether the URL should stay untouched
fn is_remote_preview_url(src: &str) -> bool {
    let lower = src.trim().to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("blob:")
        || lower.starts_with("file:")
        || lower.starts_with("//")
}

/// 解析并校验本地图片，返回 file:// URL
/// Resolve and validate a local image, returning a file:// URL
fn resolve_allowed_preview_image(base: &Path, src: &str, allowed: &[PathBuf]) -> Option<String> {
    let cleaned = src.trim().split(['?', '#']).next().unwrap_or("").trim();
    if cleaned.is_empty() {
        return None;
    }
    let candidate = {
        let path = PathBuf::from(cleaned);
        if path.is_absolute() {
            path
        } else {
            base.join(path)
        }
    };
    if !candidate.is_file() {
        return None;
    }
    let canon = candidate.canonicalize().ok()?;
    if !allowed.iter().any(|root| canon.starts_with(root)) {
        return None;
    }
    path_to_file_url(&canon)
}

/// 把本地路径编码为 file:// URL / Encode a local path as a file:// URL
fn path_to_file_url(path: &Path) -> Option<String> {
    let raw = path.to_string_lossy();
    let stripped = raw.trim_start_matches(r"\\?\").replace('\\', "/");
    let mut url = String::from("file://");
    if !stripped.starts_with('/') {
        url.push('/');
    }
    for (index, part) in stripped.split('/').enumerate() {
        if index > 0 {
            url.push('/');
        }
        if part.is_empty() {
            continue;
        }
        if index == 0 && part.len() == 2 && part.as_bytes()[1] == b':' {
            url.push_str(part);
        } else {
            url.push_str(&percent_encode_path_segment(part));
        }
    }
    Some(url)
}

/// 百分号编码路径段 / Percent-encode a path segment
fn percent_encode_path_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_javascript_urls() {
        let html = render_markdown("[click](javascript:alert(1))");
        assert!(!html.to_lowercase().contains("javascript:"));
    }

    #[test]
    fn strips_raw_script() {
        let html = render_markdown("ok\n\n<script>alert('xss')</script>");
        assert!(!html.contains("<script"));
        assert!(html.contains("ok"));
    }

    #[test]
    fn strips_svg_data_image_urls() {
        let html = render_markdown(
            "![x](data:image/svg+xml;base64,PHN2ZyBvbmxvYWQ9ImFsZXJ0KDEpIj48L3N2Zz4=)",
        );
        assert!(!html.to_lowercase().contains("data:image/svg"));
    }

    #[test]
    fn keeps_safe_png_data_image_urls() {
        let html = render_markdown("![x](data:image/png;base64,AAAA)");
        assert!(html.to_lowercase().contains("data:image/png;base64,aaaa"));
    }

    #[test]
    fn highlights_fenced_rust_and_escapes_html() {
        let html = render_markdown("```rust\nfn main() { let _ = \"<script>\"; }\n```");
        assert!(html.contains("code-block"));
        assert!(html.contains("data-lang=\"rust\""));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;") || html.contains("&lt;"));
    }

    #[test]
    fn renders_mermaid_fence_as_diagram_source() {
        let html = render_markdown("```mermaid\ngraph TD\nA-->B\n```");
        assert!(html.contains("class=\"mermaid\""));
        assert!(html.contains("graph TD"));
        assert!(html.contains("A--&gt;B") || html.contains("A-->B"));
    }

    #[test]
    fn block_elements_carry_source_lines() {
        let html = render_markdown("# Title\n\nHello\n\n- item\n");
        assert!(html.contains("<h1 data-source-line=\"0\""));
        assert!(html.contains("<p data-source-line=\"2\""));
        assert!(html.contains("<li data-source-line=\"4\""));
    }

    #[test]
    fn renders_inline_and_display_math() {
        let html = render_markdown("Euler: $e^{i\\pi}+1=0$\n\n$$\\int_0^1 x dx$$");
        assert!(html.contains("math-inline"));
        assert!(html.contains("math-display"));
        assert!(html.contains("e^{i\\pi}+1=0") || html.contains("e^{i"));
    }

    #[test]
    fn preview_images_use_file_url_inside_doc_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let img = dir.path().join("pic.png");
        std::fs::write(&img, [0x89, 0x50, 0x4E, 0x47]).unwrap();
        let html = r#"<p><img src="pic.png" alt="x"></p>"#;
        let rewritten = rewrite_local_preview_images(html, Some(dir.path()), &[]);
        assert!(rewritten.contains("file://"));
        assert!(rewritten.contains("pic.png"));
        assert!(!rewritten.contains("src=\"pic.png\""));
    }

    #[test]
    fn preview_images_skip_remote_and_escape() {
        let dir = tempfile::TempDir::new().unwrap();
        let html = r#"<img src="https://example.com/a.png"><img src="../secret.png">"#;
        let rewritten = rewrite_local_preview_images(html, Some(dir.path()), &[]);
        assert!(rewritten.contains("https://example.com/a.png"));
        assert!(rewritten.contains("../secret.png"));
        assert!(!rewritten.contains("file://"));
    }
}
