//! Markdown 渲染服务：pulldown-cmark + 自写 URL/原始 HTML 过滤
//! Markdown rendering: pulldown-cmark plus a small URL / raw-HTML filter

use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};

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
    if lower.starts_with("data:") && !lower.starts_with("data:image/") {
        return true;
    }
    false
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
        let events = parser.filter_map(sanitize_event);
        let mut html_output = String::new();
        html::push_html(&mut html_output, events);
        html_output
    }
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
}
