//! Markdown 渲染服务 / Markdown Rendering Service

use ammonia::{Builder, UrlRelative};
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

fn build_sanitizer() -> Builder<'static> {
    let mut builder = Builder::default();
    builder.add_tags([
        "p",
        "br",
        "hr",
        "pre",
        "code",
        "blockquote",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "ul",
        "ol",
        "li",
        "dl",
        "dt",
        "dd",
        "table",
        "thead",
        "tbody",
        "tfoot",
        "tr",
        "th",
        "td",
        "em",
        "strong",
        "del",
        "sup",
        "sub",
        "a",
        "img",
        "span",
        "div",
        "input",
    ]);
    builder.add_generic_attributes(["id", "class", "style"]);
    builder.add_tag_attributes("a", ["href", "title"]);
    builder.add_tag_attributes("img", ["src", "alt", "title", "width", "height"]);
    builder.add_tag_attributes("input", ["type", "checked", "disabled"]);
    builder.add_tag_attributes("td", ["align", "valign"]);
    builder.add_tag_attributes("th", ["align", "valign"]);
    builder.add_tag_attributes("pre", ["class"]);
    builder.add_tag_attributes("div", ["class"]);
    builder.url_relative(UrlRelative::PassThrough);
    builder
}

/// Markdown 服务 / Markdown Service
pub struct MarkdownService;

impl MarkdownService {
    /// 创建新实例 / Create New Instance
    pub fn new() -> Self {
        Self
    }

    /// 渲染 Markdown 为 HTML / Render Markdown to HTML
    /// 输出经过 ammonia 消毒处理，防止 XSS 攻击
    /// Output is sanitized by ammonia to prevent XSS attacks
    pub fn render(&self, content: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES); // 表格 / Tables
        options.insert(Options::ENABLE_STRIKETHROUGH); // 删除线 / Strikethrough
        options.insert(Options::ENABLE_TASKLISTS); // 任务列表 / Task Lists
        options.insert(Options::ENABLE_SMART_PUNCTUATION); // 智能标点 / Smart Punctuation
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES); // 标题属性 / Heading Attributes
        options.insert(Options::ENABLE_FOOTNOTES); // 脚注 / Footnotes

        let processed_content = preprocess_math_formulas(content);

        let parser = Parser::new_ext(&processed_content, options);

        // 处理 Mermaid 代码块 / Process Mermaid code blocks
        let events: Vec<Event> = parser.collect();
        let processed_events = process_mermaid_blocks(events);

        let mut html_output = String::new();
        html::push_html(&mut html_output, processed_events.into_iter());

        // 使用 ammonia 消毒 HTML 输出（在追加 script 之前）
        // Sanitize HTML output with ammonia (before appending script)
        let sanitizer = build_sanitizer();
        html_output = sanitizer.clean(&html_output).to_string();

        // 添加 Mermaid 初始化脚本（已知安全，在消毒后追加）
        // Add Mermaid initialization script (known-safe, appended after sanitization)
        html_output.push_str(
            r#"<script>
if(typeof mermaid !== 'undefined') {
    mermaid.init(undefined, '.mermaid');
}
</script>"#,
        );

        html_output
    }

    /// 渲染 Markdown 并进行语法高亮 / Render Markdown with Syntax Highlighting
    /// 输出经过 ammonia 消毒处理，防止 XSS 攻击
    /// Output is sanitized by ammonia to prevent XSS attacks
    pub fn render_with_highlight(&self, content: &str) -> String {
        use crate::services::syntax_highlight::SyntaxHighlightService;

        let highlighter = SyntaxHighlightService::new();

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_FOOTNOTES);

        let processed_content = preprocess_math_formulas(content);
        let parser = Parser::new_ext(&processed_content, options);

        let events: Vec<Event> = parser.collect();
        let processed_events = highlight_code_blocks(events);

        let mut html_output = String::new();
        html::push_html(&mut html_output, processed_events.into_iter());

        // 使用 ammonia 消毒 HTML 输出（在追加 style/script 之前）
        // Sanitize HTML output with ammonia (before appending style/script)
        let sanitizer = build_sanitizer();
        html_output = sanitizer.clean(&html_output).to_string();

        // 添加代码高亮 CSS（已知安全，在消毒后追加）
        // Add code highlight CSS (known-safe, appended after sanitization)
        html_output.push_str(&format!(
            r#"<style>{}</style>"#,
            highlighter.get_code_block_css()
        ));

        // 添加 Mermaid 初始化脚本（已知安全，在消毒后追加）
        // Add Mermaid initialization script (known-safe, appended after sanitization)
        html_output.push_str(
            r#"<script>
if(typeof mermaid !== 'undefined') {
    mermaid.init(undefined, '.mermaid');
}
</script>"#,
        );

        html_output
    }
}

impl Default for MarkdownService {
    fn default() -> Self {
        Self::new()
    }
}

/// 转义 HTML 特殊字符（用于 Mermaid 内容）/ Escape HTML special chars (for Mermaid content)
fn escape_html_for_mermaid(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}

/// 处理 Mermaid 代码块 / Process Mermaid Code Blocks
fn process_mermaid_blocks(events: Vec<Event>) -> Vec<Event> {
    let mut result = Vec::new();
    let mut in_code_block = false;
    let mut is_mermaid = false;
    let mut code_content = String::new();

    for event in events {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = true;
                is_mermaid = lang.to_string().to_lowercase() == "mermaid";
                code_content.clear();
                if !is_mermaid {
                    result.push(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))));
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if is_mermaid {
                    let escaped = escape_html_for_mermaid(&code_content);
                    result.push(Event::Html(
                        format!(r#"<div class="mermaid">{}</div>"#, escaped).into(),
                    ));
                } else {
                    result.push(Event::Text(code_content.clone().into()));
                    result.push(Event::End(TagEnd::CodeBlock));
                }
                in_code_block = false;
                is_mermaid = false;
            }
            Event::Text(text) if in_code_block => {
                code_content.push_str(&text);
            }
            _ => {
                result.push(event);
            }
        }
    }

    result
}

/// 高亮代码块事件处理 / Highlight code blocks in event stream
fn highlight_code_blocks(events: Vec<Event>) -> Vec<Event> {
    let highlighter = crate::services::syntax_highlight::SyntaxHighlightService::new();
    let mut result = Vec::new();
    let mut in_code_block = false;
    let mut is_mermaid = false;
    let mut code_content = String::new();
    let mut code_lang = String::new();

    for event in events {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = true;
                code_lang = lang.to_string();
                is_mermaid = code_lang.to_lowercase() == "mermaid";
                code_content.clear();
                if is_mermaid {
                    // Mermaid 块不开始代码块标签 / Don't start code block for Mermaid
                } else {
                    // 使用高亮渲染代码块 / Use highlighter for code block
                    // 仍然开始代码块标签 / Still start code block tag
                    result.push(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))));
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if is_mermaid {
                    let escaped = escape_html_for_mermaid(&code_content);
                    result.push(Event::Html(
                        format!(r#"<div class="mermaid">{}</div>"#, escaped).into(),
                    ));
                } else if !code_lang.is_empty() {
                    // 尝试语法高亮 / Try syntax highlighting
                    let highlighted = highlighter.highlight_code_block(&code_content, &code_lang);
                    result.push(Event::Html(highlighted.into()));
                    // 跳过默认的 End 事件，因为 highlight_code_block 已生成完整 HTML
                    in_code_block = false;
                    is_mermaid = false;
                    continue;
                } else {
                    result.push(Event::Text(code_content.clone().into()));
                    result.push(Event::End(TagEnd::CodeBlock));
                }
                in_code_block = false;
                is_mermaid = false;
            }
            Event::Text(text) if in_code_block => {
                code_content.push_str(&text);
            }
            _ => {
                result.push(event);
            }
        }
    }

    result
}

/// 预处理数学公式，将 $...$ 转换为 data-attribute span 元素
/// Preprocess math formulas, convert $...$ to data-attribute span elements
///
/// 安全措施 / Safety measures:
/// - 跳过代码块 (```...```) / Skip fenced code blocks
/// - 跳过行内代码 (`...`) / Skip inline code
/// - 忽略转义的 \$ / Ignore escaped \$
pub fn preprocess_math_formulas(content: &str) -> String {
    let mut result = String::new();
    let mut in_code_block = false;

    for line in content.lines() {
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_code_block {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let mut processed = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // 跳过行内代码 `...` / Skip inline code
            if chars[i] == '`' {
                processed.push(chars[i]);
                i += 1;
                while i < chars.len() && chars[i] != '`' {
                    processed.push(chars[i]);
                    i += 1;
                }
                // 消耗闭合反引号 / Consume closing backtick
                if i < chars.len() {
                    processed.push(chars[i]);
                    i += 1;
                }
                continue;
            }

            // 跳过转义的 \$ / Skip escaped \$
            if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == '$' {
                processed.push_str("\\$");
                i += 2;
                continue;
            }

            // Check for $$...$$
            if i + 1 < chars.len() && chars[i] == '$' && chars[i + 1] == '$' {
                if let Some(end) = find_closing_delimiter(&chars, i + 2, true) {
                    let tex: String = chars[i + 2..end].iter().collect();
                    let encoded = html_encode_attribute(&tex);
                    processed.push_str(&format!("<div data-formula-block=\"{}\"></div>", encoded));
                    i = end + 2;
                    continue;
                }
            }
            // Check for $...$ (inline)
            else if chars[i] == '$' {
                if let Some(end) = find_closing_delimiter(&chars, i + 1, false) {
                    let tex: String = chars[i + 1..end].iter().collect();
                    if !tex.is_empty() && !tex.contains('\n') {
                        let encoded = html_encode_attribute(&tex);
                        processed.push_str(&format!(
                            "<span data-formula-inline=\"{}\"></span>",
                            encoded
                        ));
                        i = end + 1;
                        continue;
                    }
                }
            }
            processed.push(chars[i]);
            i += 1;
        }

        result.push_str(&processed);
        result.push('\n');
    }

    result
}

/// HTML 属性值编码 / HTML attribute value encoding
pub fn html_encode_attribute(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    result
}

/// 查找闭合定界符 / Find closing delimiter
fn find_closing_delimiter(chars: &[char], start: usize, is_block: bool) -> Option<usize> {
    let mut i = start;
    while i < chars.len() {
        if chars[i] == '$' {
            if is_block {
                // Need $$
                if i + 1 < chars.len() && chars[i + 1] == '$' {
                    return Some(i);
                }
                i += 2;
            } else {
                return Some(i);
            }
        } else {
            i += 1;
        }
    }
    None
}

/// 便捷函数：渲染 Markdown / Convenience Function: Render Markdown
pub fn render_markdown(content: &str) -> String {
    let service = MarkdownService::new();
    service.render(content)
}

/// 获取 Mermaid CDN 脚本标签（带离线检测、缓存和降级）/ Get Mermaid CDN Script Tag (with offline detection, caching and fallback)
pub fn mermaid_script() -> &'static str {
    r#"<script>
(function() {
    const CACHE_KEY = 'md_mermaid_cache';
    const CACHE_VERSION = 'v10.9.0';
    
    // 降级显示 / Fallback display
    function showFallback(showNetworkMsg) {
        var mermaidDivs = document.querySelectorAll('.mermaid');
        mermaidDivs.forEach(function(div) {
            if (!div.dataset.rendered) {
                div.style.fontFamily = 'monospace';
                div.style.whiteSpace = 'pre';
                div.style.background = '#1e1e1e';
                div.style.padding = '10px';
                div.style.borderRadius = '4px';
                div.style.border = '1px solid #444';
                var msg = showNetworkMsg ? '\n\n[图表渲染需要网络连接 / Chart rendering requires network connection]' : '';
                div.textContent = '[Chart] ' + div.textContent + msg;
                div.dataset.rendered = 'true';
            }
        });
    }
    
    // 初始化 Mermaid / Initialize Mermaid
    function initMermaid() {
        if (typeof mermaid !== 'undefined') {
            mermaid.initialize({ 
                startOnLoad: true,
                theme: 'dark',
                securityLevel: 'strict'
            });
            window._mermaidAvailable = true;
            // 重新渲染 / Re-render
            mermaid.init(undefined, '.mermaid');
        }
    }
    
    // 从缓存加载 / Load from cache
    function loadFromCache() {
        try {
            var cached = localStorage.getItem(CACHE_KEY);
            if (cached) {
                var data = JSON.parse(cached);
                if (data.version === CACHE_VERSION && data.code) {
                    var s = document.createElement('script');
                    s.textContent = data.code;
                    document.head.appendChild(s);
                    initMermaid();
                    console.log('Mermaid loaded from cache');
                    return true;
                }
            }
        } catch(e) {
            console.warn('Failed to load Mermaid from cache:', e);
        }
        return false;
    }
    
    // 从 CDN 加载并缓存 / Load from CDN and cache
    function loadFromCDN() {
        var script = document.createElement('script');
        script.src = 'https://cdn.jsdelivr.net/npm/mermaid@10.9.0/dist/mermaid.min.js';
        script.onload = function() {
            // 缓存到 localStorage / Cache to localStorage
            try {
                // 获取 mermaid 对象的源码（通过重新获取脚本内容）
                fetch('https://cdn.jsdelivr.net/npm/mermaid@10.9.0/dist/mermaid.min.js')
                    .then(r => r.text())
                    .then(code => {
                        localStorage.setItem(CACHE_KEY, JSON.stringify({
                            version: CACHE_VERSION,
                            code: code,
                            timestamp: Date.now()
                        }));
                        console.log('Mermaid cached for offline use');
                    })
                    .catch(() => {});
            } catch(e) {}
            initMermaid();
        };
        script.onerror = function() {
            console.warn('Mermaid CDN load failed');
            window._mermaidAvailable = false;
            // 尝试从缓存加载 / Try cache fallback
            if (!loadFromCache()) {
                showFallback(true);
            }
        };
        document.head.appendChild(script);
    }
    
    // 主逻辑 / Main logic
    var isOnline = navigator.onLine;
    
    if (isOnline) {
        loadFromCDN();
    } else {
        // 离线时优先使用缓存 / Use cache when offline
        if (!loadFromCache()) {
            window._mermaidAvailable = false;
            showFallback(false);
        }
    }
    
    // 监听网络状态变化 / Listen for network status changes
    window.addEventListener('online', function() {
        if (!window._mermaidAvailable) {
            loadFromCDN();
        }
    });
})();
</script>"#
}

/// 获取 KaTeX CDN 脚本标签（用于数学公式，带离线检测和缓存）/ Get KaTeX CDN Script Tag (for math formulas, with offline detection and caching)
pub fn katex_script() -> &'static str {
    r#"<script>
(function() {
    const CACHE_KEY = 'md_katex_cache';
    const CACHE_VERSION = 'v0.16.9';
    
    function renderMath() {
        if (typeof katex === 'undefined') return;
        
        // 安全地处理公式 / Safely process formulas
        // 使用 TreeWalker 遍历文本节点，避免 innerHTML 注入
        // Use TreeWalker to traverse text nodes, avoiding innerHTML injection
        var previewContent = document.querySelector('.preview-content');
        if (!previewContent) return;
        
        // 处理块级公式 $$...$$ / Process block formulas $$...$$
        var blockFormulas = previewContent.querySelectorAll('[data-formula-block]');
        blockFormulas.forEach(function(el) {
            try {
                var tex = el.getAttribute('data-formula-block');
                el.innerHTML = katex.renderToString(tex.trim(), { throwOnError: false, displayMode: true });
                el.style.textAlign = 'center';
                el.style.margin = '1em 0';
                el.removeAttribute('data-formula-block');
            } catch(e) {
                el.textContent = '$$' + el.getAttribute('data-formula-block') + '$$';
                el.removeAttribute('data-formula-block');
            }
        });
        
        // 处理行内公式 $...$ / Process inline formulas $...$
        var inlineFormulas = previewContent.querySelectorAll('[data-formula-inline]');
        inlineFormulas.forEach(function(el) {
            try {
                var tex = el.getAttribute('data-formula-inline');
                el.innerHTML = katex.renderToString(tex.trim(), { throwOnError: false });
                el.removeAttribute('data-formula-inline');
            } catch(e) {
                el.textContent = '$' + el.getAttribute('data-formula-inline') + '$';
                el.removeAttribute('data-formula-inline');
            }
        });
    }
    
    function loadFromCache() {
        try {
            var cached = localStorage.getItem(CACHE_KEY);
            if (cached) {
                var data = JSON.parse(cached);
                if (data.version === CACHE_VERSION && data.code) {
                    var s = document.createElement('script');
                    s.textContent = data.code;
                    document.head.appendChild(s);
                    window._katexAvailable = true;
                    renderMath();
                    console.log('KaTeX loaded from cache');
                    return true;
                }
            }
        } catch(e) {
            console.warn('Failed to load KaTeX from cache:', e);
        }
        return false;
    }
    
    function loadKaTeX() {
        // 加载 CSS / Load CSS
        var link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = 'https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css';
        document.head.appendChild(link);
        
        // 加载 JS / Load JS
        var script = document.createElement('script');
        script.src = 'https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js';
        script.onload = function() {
            // 缓存到 localStorage / Cache to localStorage
            try {
                fetch('https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js')
                    .then(r => r.text())
                    .then(code => {
                        localStorage.setItem(CACHE_KEY, JSON.stringify({
                            version: CACHE_VERSION,
                            code: code,
                            timestamp: Date.now()
                        }));
                        console.log('KaTeX cached for offline use');
                    })
                    .catch(() => {});
            } catch(e) {}
            
            window._katexAvailable = true;
            renderMath();
        };
        script.onerror = function() {
            console.warn('KaTeX CDN load failed');
            window._katexAvailable = false;
            // 尝试从缓存加载 / Try cache fallback
            if (!loadFromCache()) {
                // 安全显示原始 LaTeX：使用 textContent 而非 innerHTML 防止 XSS
                // Safely show raw LaTeX: use textContent instead of innerHTML to prevent XSS
                var previewContent = document.querySelector('.preview-content');
                if (previewContent) {
                    var blocks = previewContent.querySelectorAll('[data-formula-block]');
                    blocks.forEach(function(el) {
                        var tex = el.getAttribute('data-formula-block');
                        el.textContent = '$$' + tex + '$$';
                        el.removeAttribute('data-formula-block');
                    });
                    var inlines = previewContent.querySelectorAll('[data-formula-inline]');
                    inlines.forEach(function(el) {
                        var tex = el.getAttribute('data-formula-inline');
                        el.textContent = '$' + tex + '$';
                        el.removeAttribute('data-formula-inline');
                    });
                }
            }
        };
        document.head.appendChild(script);
    }
    
    var isOnline = navigator.onLine;
    
    if (isOnline) {
        loadKaTeX();
    } else {
        if (!loadFromCache()) {
            window._katexAvailable = false;
        }
    }
    
    // 监听网络状态变化 / Listen for network status changes
    window.addEventListener('online', function() {
        if (!window._katexAvailable) {
            loadKaTeX();
        }
    });
})();
</script>"#
}
