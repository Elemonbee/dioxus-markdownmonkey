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
    let normalized = normalize_url_for_scheme_check(value);
    let lower = normalized.to_ascii_lowercase();
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

/// 去掉空白、NUL，并解码常见实体，避免 `java\nscript:` 或 `&#106;avascript:` 绕过检查
/// Strip whitespace and NUL, then decode common entities, so `java\nscript:` and `&#106;avascript:` cannot slip through
fn normalize_url_for_scheme_check(value: &str) -> String {
    let decoded = decode_basic_entities(value.trim());
    decoded
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '\0')
        .collect()
}

/// 解码 HTML 属性里常见的字符实体 / Decode common character entities in HTML attributes
fn decode_basic_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find(';') {
            if let Some(ch) = decode_entity(&after[..end]) {
                out.push(ch);
                rest = &after[end + 1..];
                continue;
            }
        }
        out.push('&');
        rest = after;
    }
    out.push_str(rest);
    out
}

/// 解码单个不带 `&` `;` 的实体名 / Decode one entity name without the surrounding `&` and `;`
fn decode_entity(entity: &str) -> Option<char> {
    let entity = entity.to_ascii_lowercase();
    match entity.as_str() {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" => Some('\''),
        "colon" => Some(':'),
        "tab" => Some('\t'),
        "newline" => Some('\n'),
        "nbsp" => Some('\u{00A0}'),
        _ => {
            let code = if let Some(hex) = entity.strip_prefix("#x") {
                u32::from_str_radix(hex, 16).ok()
            } else {
                entity
                    .strip_prefix('#')
                    .and_then(|digits| digits.parse().ok())
            }?;
            char::from_u32(code)
        }
    }
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

/// 允许出现在预览里的标签 / Tags allowed in the preview
fn is_allowed_html_tag(name: &str) -> bool {
    matches!(
        name,
        "a" | "abbr"
            | "b"
            | "blockquote"
            | "br"
            | "caption"
            | "cite"
            | "code"
            | "dd"
            | "del"
            | "details"
            | "div"
            | "dl"
            | "dt"
            | "em"
            | "figcaption"
            | "figure"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "hr"
            | "i"
            | "img"
            | "kbd"
            | "li"
            | "mark"
            | "ol"
            | "p"
            | "pre"
            | "q"
            | "s"
            | "samp"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "summary"
            | "sup"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
            | "u"
            | "ul"
            | "wbr"
            | "video"
            | "audio"
            | "source"
    )
}

/// 会连同内部内容一起丢掉的标签 / Tags dropped together with their contents
fn is_dangerous_html_tag(name: &str) -> bool {
    matches!(
        name,
        "script"
            | "style"
            | "iframe"
            | "object"
            | "embed"
            | "svg"
            | "math"
            | "noscript"
            | "template"
            | "applet"
            | "frameset"
            | "frame"
            | "form"
            | "button"
            | "textarea"
            | "select"
            | "option"
            | "link"
            | "meta"
            | "base"
            | "foreignobject"
            | "noembed"
            | "noframes"
            | "plaintext"
            | "xmp"
            | "listing"
            | "portal"
    )
}

/// 没有结束标签的元素 / Elements that have no end tag
fn is_void_html_tag(name: &str) -> bool {
    matches!(
        name,
        "br" | "hr" | "img" | "wbr" | "source" | "input" | "meta" | "link" | "base"
    )
}

/// 解析到的一个 HTML 标签 / One parsed HTML tag
struct RawHtmlTag<'a> {
    name: String,
    is_close: bool,
    self_closing: bool,
    attrs: Vec<(&'a str, Option<String>)>,
    end: usize,
}

/// 从 `<` 起解析一个标签；解析失败返回 None
/// Parse one tag starting at `<`; return None when the markup is not a tag
fn parse_html_tag(input: &str, start: usize) -> Option<RawHtmlTag<'_>> {
    let bytes = input.as_bytes();
    if start >= bytes.len() || bytes[start] != b'<' {
        return None;
    }
    let mut i = start + 1;
    let is_close = if bytes.get(i) == Some(&b'/') {
        i += 1;
        true
    } else {
        false
    };
    while bytes.get(i).is_some_and(|b| b.is_ascii_whitespace()) {
        i += 1;
    }
    let name_start = i;
    while bytes.get(i).is_some_and(|b| b.is_ascii_alphanumeric()) {
        i += 1;
    }
    if i == name_start {
        return None;
    }
    let name = input[name_start..i].to_ascii_lowercase();
    let mut attrs = Vec::new();
    let mut self_closing = false;
    if !is_close {
        loop {
            while bytes.get(i).is_some_and(|b| b.is_ascii_whitespace()) {
                i += 1;
            }
            match bytes.get(i) {
                Some(b'>') => {
                    i += 1;
                    break;
                }
                Some(b'/') if bytes.get(i + 1) == Some(&b'>') => {
                    self_closing = true;
                    i += 2;
                    break;
                }
                Some(_) => {}
                None => return None,
            }
            let attr_start = i;
            while bytes
                .get(i)
                .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b':' | b'_'))
            {
                i += 1;
            }
            if i == attr_start {
                return None;
            }
            let attr_name = &input[attr_start..i];
            while bytes.get(i).is_some_and(|b| b.is_ascii_whitespace()) {
                i += 1;
            }
            let value = if bytes.get(i) == Some(&b'=') {
                i += 1;
                while bytes.get(i).is_some_and(|b| b.is_ascii_whitespace()) {
                    i += 1;
                }
                let quote = bytes.get(i).copied();
                if matches!(quote, Some(b'"' | b'\'')) {
                    i += 1;
                    let value_start = i;
                    while bytes.get(i).is_some_and(|b| Some(*b) != quote) {
                        i += 1;
                    }
                    if bytes.get(i) != quote.as_ref() {
                        return None;
                    }
                    let value = input[value_start..i].to_string();
                    i += 1;
                    Some(value)
                } else {
                    let value_start = i;
                    while bytes
                        .get(i)
                        .is_some_and(|b| !b.is_ascii_whitespace() && *b != b'>' && *b != b'/')
                    {
                        i += 1;
                    }
                    if i == value_start {
                        return None;
                    }
                    Some(input[value_start..i].to_string())
                }
            } else {
                None
            };
            attrs.push((attr_name, value));
        }
    } else {
        while bytes.get(i).is_some_and(|b| *b != b'>') {
            i += 1;
        }
        if bytes.get(i) != Some(&b'>') {
            return None;
        }
        i += 1;
    }
    Some(RawHtmlTag {
        name,
        is_close,
        self_closing,
        attrs,
        end: i,
    })
}

/// 保留安全属性和 URL，丢掉事件处理和样式
/// Keep safe attributes and URLs, and drop event handlers and styles
fn sanitize_html_attr(
    tag: &str,
    name: &str,
    value: Option<&str>,
) -> Option<(String, Option<String>)> {
    let name = name.to_ascii_lowercase();
    if name.starts_with("on") || name == "style" || name.contains("xmlns") {
        return None;
    }
    let url_attr = matches!(name.as_str(), "href" | "src" | "cite");
    let global = matches!(name.as_str(), "title" | "class" | "id" | "dir");
    let allowed = match tag {
        "a" => url_attr && name == "href" || global,
        "img" | "source" => matches!(name.as_str(), "src" | "alt" | "width" | "height") || global,
        "video" | "audio" => {
            matches!(name.as_str(), "src" | "controls" | "width" | "height") || global
        }
        "td" | "th" => matches!(name.as_str(), "colspan" | "rowspan") || global,
        "ol" => matches!(name.as_str(), "start" | "type") || global,
        "blockquote" | "q" => name == "cite" || global,
        "details" => name == "open" || global,
        _ => global,
    };
    if !allowed {
        return None;
    }
    if matches!(name.as_str(), "controls" | "open") {
        return Some((name, None));
    }
    let raw = value.unwrap_or("");
    let cleaned = if url_attr {
        if is_dangerous_url(raw) {
            return None;
        }
        raw.to_string()
    } else if matches!(name.as_str(), "class" | "id") {
        raw.chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '_' | '-' | ':'))
            .collect()
    } else if matches!(
        name.as_str(),
        "width" | "height" | "colspan" | "rowspan" | "start"
    ) {
        let digits: String = raw.chars().filter(|ch| ch.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        digits
    } else if name == "dir" {
        match raw.trim().to_ascii_lowercase().as_str() {
            "ltr" | "rtl" | "auto" => raw.trim().to_ascii_lowercase(),
            _ => return None,
        }
    } else if name == "type" {
        match raw.trim() {
            "1" | "a" | "A" | "i" | "I" => raw.trim().to_string(),
            _ => return None,
        }
    } else {
        raw.chars().filter(|ch| !ch.is_control()).collect()
    };
    if cleaned.is_empty() && name != "alt" {
        return None;
    }
    Some((name, Some(cleaned)))
}

/// 把 Markdown 里的原始 HTML 收成安全子集
/// Reduce raw HTML embedded in Markdown to a safe subset
fn sanitize_raw_html(input: &str) -> String {
    let mut out = String::new();
    let mut skip: Vec<String> = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if skip.is_empty() && input[i..].starts_with("<!--") {
                if let Some(end) = input[i..].find("-->") {
                    i += end + 3;
                    continue;
                }
            }
            if let Some(tag) = parse_html_tag(input, i) {
                let end = tag.end;
                if !skip.is_empty() {
                    if tag.is_close && tag.name == *skip.last().unwrap() {
                        skip.pop();
                    } else if !tag.is_close
                        && !tag.self_closing
                        && is_dangerous_html_tag(&tag.name)
                        && !is_void_html_tag(&tag.name)
                    {
                        skip.push(tag.name);
                    }
                    i = end;
                    continue;
                }
                if tag.is_close {
                    if is_allowed_html_tag(&tag.name) {
                        out.push_str("</");
                        out.push_str(&tag.name);
                        out.push('>');
                    }
                } else if is_dangerous_html_tag(&tag.name) {
                    if !tag.self_closing && !is_void_html_tag(&tag.name) {
                        skip.push(tag.name);
                    }
                } else if is_allowed_html_tag(&tag.name) {
                    out.push('<');
                    out.push_str(&tag.name);
                    for (attr_name, attr_value) in &tag.attrs {
                        if let Some((safe_name, safe_value)) =
                            sanitize_html_attr(&tag.name, attr_name, attr_value.as_deref())
                        {
                            out.push(' ');
                            out.push_str(&safe_name);
                            if let Some(value) = safe_value {
                                out.push_str("=\"");
                                out.push_str(&highlight::escape_attr(&value));
                                out.push('"');
                            }
                        }
                    }
                    if tag.self_closing || is_void_html_tag(&tag.name) {
                        out.push_str(" />");
                    } else {
                        out.push('>');
                    }
                }
                i = end;
                continue;
            }
        }
        if !skip.is_empty() {
            i += 1;
            continue;
        }
        let ch = input[i..].chars().next().unwrap_or('\u{FFFD}');
        match ch {
            '<' => out.push_str("&lt;"),
            _ => out.push(ch),
        }
        i += ch.len_utf8();
    }
    out
}

/// 过滤原始 HTML，并清洗链接/图片 URL
/// Sanitize raw HTML and link/image URLs
fn sanitize_event(event: Event<'_>) -> Option<Event<'_>> {
    match event {
        Event::Html(raw) | Event::InlineHtml(raw) => {
            let cleaned = sanitize_raw_html(&raw);
            if cleaned.is_empty() {
                None
            } else {
                Some(Event::Html(cleaned.into()))
            }
        }
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
    fn keeps_safe_html_and_strips_event_handlers() {
        let html = render_markdown(
            "<details><summary>More</summary>body</details>\n\n<p onclick=\"alert(1)\">x</p>",
        );
        assert!(html.contains("<details>"));
        assert!(html.contains("<summary>"));
        assert!(html.contains("More"));
        assert!(html.contains("<p>"));
        assert!(html.contains(">x</p>") || html.contains(">x</p"));
        assert!(!html.to_lowercase().contains("onclick"));
    }

    #[test]
    fn strips_obfuscated_javascript_urls() {
        let html = render_markdown(
            "[x](java\nscript:alert(1)) <a href=\"&#106;avascript:alert(1)\">y</a>",
        );
        assert!(!html.to_lowercase().contains("javascript:"));
    }

    /// 命名实体和活动标签不能在浏览器里重新拼出脚本
    /// Named entities and active markup must not reassemble a script in the browser
    #[test]
    fn strips_entity_schemes_and_active_markup() {
        let html = render_markdown(
            "<a href=\"javascript&colon;alert(1)\">link</a>\n\n\
             <a href=\"java&Tab;script:alert(1)\">tab</a>\n\n\
             <img src=\"x\" onerror=\"alert(1)\">\n\n\
             <svg><script>alert(1)</script></svg>\n\n\
             <iframe srcdoc=\"<script>alert(1)</script>\"></iframe>\n\n\
             <math><mi>x</mi></math>\n\n\
             <object data=\"javascript:alert(1)\"></object>",
        );
        let lower = html.to_lowercase();
        assert!(html.contains(">link</a>"));
        assert!(!lower.contains("javascript"));
        assert!(!lower.contains("onerror"));
        assert!(!lower.contains("<script"));
        assert!(!lower.contains("<svg"));
        assert!(!lower.contains("<iframe"));
        assert!(!lower.contains("srcdoc"));
        assert!(!lower.contains("<math"));
        assert!(!lower.contains("<object"));
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
