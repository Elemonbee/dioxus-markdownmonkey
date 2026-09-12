//! 预览代码块语法高亮 / Preview code-block syntax highlighting
//!
//! 使用 syntect 生成带 class 的 HTML，颜色由 CSS 变量跟随主题。
//! Uses syntect to emit classed HTML; colors follow CSS variables / theme.

use crate::config::{CODE_HIGHLIGHT_MAX_BYTES, MERMAID_MAX_BYTES};
use std::sync::OnceLock;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// 惰性加载默认语法集 / Lazily load the default syntax set
fn syntax_set() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// 将围栏语言名映射到 syntect 可识别的 token
/// Map a fence language name to a token syntect understands
fn language_token(lang: &str) -> Option<String> {
    let trimmed = lang.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    let token = match lower.as_str() {
        "rs" => "rust",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "ts" | "tsx" => "typescript",
        "py" | "python3" => "python",
        "sh" | "zsh" | "shell" | "console" => "bash",
        "yml" => "yaml",
        "md" => "markdown",
        "cs" => "csharp",
        "kt" | "kts" => "kotlin",
        "hpp" | "hh" | "cc" | "cxx" => "cpp",
        "htm" => "html",
        "plaintext" | "text" | "txt" | "mermaid" => return None,
        other => other,
    };
    Some(token.to_string())
}

/// 转义 HTML 文本节点 / Escape text for an HTML text node
pub fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 转义 HTML 属性值 / Escape a value for an HTML attribute
pub fn escape_attr(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 将代码高亮为内部 HTML（不含外层 pre/code）
/// Highlight source to inner HTML (without the wrapping pre/code)
pub fn highlight_inner_html(lang: &str, code: &str) -> String {
    if code.len() > CODE_HIGHLIGHT_MAX_BYTES {
        return escape_html(code);
    }
    let Some(token) = language_token(lang) else {
        return escape_html(code);
    };
    let set = syntax_set();
    let syntax = set
        .find_syntax_by_token(&token)
        .or_else(|| set.find_syntax_by_extension(&token));
    let Some(syntax) = syntax else {
        return escape_html(code);
    };

    let mut generator = ClassedHTMLGenerator::new_with_class_style(syntax, set, ClassStyle::Spaced);
    for line in LinesWithEndings::from(code) {
        if generator
            .parse_html_for_line_which_includes_newline(line)
            .is_err()
        {
            return escape_html(code);
        }
    }
    generator.finalize()
}

/// 渲染 Mermaid 源码块（由预览 JS 再画成图）
/// Render a Mermaid source block (preview JS turns it into a diagram)
fn render_mermaid_block(code: &str) -> String {
    let escaped = escape_html(code);
    if code.len() > MERMAID_MAX_BYTES {
        return format!(
            "<pre class=\"code-block\" data-lang=\"mermaid\"><code class=\"highlight-code language-mermaid\">{escaped}</code></pre>\n"
        );
    }
    format!("<pre class=\"mermaid\">{escaped}</pre>\n")
}

/// 渲染带语言标记的高亮代码块 / Render a highlighted code block with a language label
pub fn render_highlighted_block(lang: &str, code: &str) -> String {
    let lang_clean = lang
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if lang_clean == "mermaid" {
        return render_mermaid_block(code);
    }
    let inner = highlight_inner_html(lang, code);
    let lang_attr = escape_attr(&lang_clean);
    let lang_class = if lang_clean.is_empty() {
        String::new()
    } else {
        format!(" language-{lang_attr}")
    };
    format!(
        "<pre class=\"code-block\" data-lang=\"{lang_attr}\"><code class=\"highlight-code{lang_class}\">{inner}</code></pre>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_html_in_plain_blocks() {
        let html = highlight_inner_html("", "<script>alert(1)</script>");
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn highlights_rust_keywords() {
        let html = highlight_inner_html("rust", "fn main() {}\n");
        assert!(html.contains("fn") || html.contains("keyword"));
        assert!(html.contains("<span"));
    }

    #[test]
    fn rust_alias_rs_works() {
        let html = highlight_inner_html("rs", "let x = 1;\n");
        assert!(html.contains("<span"));
    }

    #[test]
    fn unknown_language_is_escaped() {
        let html = highlight_inner_html("not-a-real-lang-xyz", "a < b");
        assert_eq!(html, "a &lt; b");
    }

    #[test]
    fn render_block_includes_language_attr() {
        let html = render_highlighted_block("python", "print(1)\n");
        assert!(html.contains("data-lang=\"python\""));
        assert!(html.contains("language-python"));
        assert!(html.contains("<pre"));
    }

    #[test]
    fn mermaid_block_is_not_syntect_highlighted() {
        let html = render_highlighted_block("mermaid", "graph TD\nA-->B\n");
        assert!(html.contains("class=\"mermaid\""));
        assert!(!html.contains("code-block"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn mermaid_block_escapes_html() {
        let html = render_highlighted_block("mermaid", "graph TD\nA[\"<script>x</script>\"]\n");
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
