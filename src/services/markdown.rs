//! Markdown 渲染服务：pulldown-cmark + 自写 URL/原始 HTML 过滤
//! Markdown rendering: pulldown-cmark plus a small URL / raw-HTML filter

use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};

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

        let parser = Parser::new_ext(content, options);
        let sanitized = parser.filter_map(sanitize_event);
        let events = rewrite_code_blocks(sanitized);
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

/// 把围栏/缩进代码块换成已高亮且消毒的 HTML
/// Replace fenced/indented code blocks with highlighted, sanitized HTML
fn rewrite_code_blocks<'a, I>(events: I) -> Vec<Event<'a>>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut out = Vec::new();
    let mut in_code = false;
    let mut language = String::new();
    let mut buffer = String::new();

    for event in events {
        if in_code {
            match event {
                Event::Text(text) | Event::Code(text) => buffer.push_str(&text),
                Event::SoftBreak | Event::HardBreak => buffer.push('\n'),
                Event::End(TagEnd::CodeBlock) => {
                    in_code = false;
                    let html = highlight::render_highlighted_block(&language, &buffer);
                    out.push(Event::Html(html.into()));
                    language.clear();
                    buffer.clear();
                }
                _ => {}
            }
            continue;
        }

        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                buffer.clear();
                language = match kind {
                    CodeBlockKind::Fenced(info) => info
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
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
}
