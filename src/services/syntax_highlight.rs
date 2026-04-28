//! 语法高亮服务 / Syntax Highlighting Service
//!
//! 使用 syntect 进行代码块语法高亮 / Uses syntect for code block syntax highlighting

use std::sync::OnceLock;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// 全局语法集 / Global syntax set
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();

/// 全局主题集 / Global theme set
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

/// 语法高亮服务 / Syntax Highlighting Service
pub struct SyntaxHighlightService {
    syntax_set: &'static SyntaxSet,
    theme_set: &'static ThemeSet,
    current_theme: String,
}

impl SyntaxHighlightService {
    /// 创建新的语法高亮服务 / Create new syntax highlighting service
    pub fn new() -> Self {
        let syntax_set = SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);

        let theme_set = THEME_SET.get_or_init(ThemeSet::load_defaults);

        Self {
            syntax_set,
            theme_set,
            current_theme: "base16-ocean.dark".to_string(),
        }
    }

    /// 高亮代码块 / Highlight code block
    pub fn highlight_code(&self, code: &str, language: &str) -> String {
        // 尝试获取语法 / Try to get syntax
        let syntax = self.get_syntax(language);

        // 获取主题 / Get theme（无可用主题时不 panic，退化为转义纯文本 / No panic if themes missing; fall back to escaped text）
        let Some(theme) = self
            .theme_set
            .themes
            .get(&self.current_theme)
            .or_else(|| {
                tracing::warn!(
                    "语法高亮主题 '{}' 未找到，尝试默认主题 / Theme '{}' missing, trying defaults",
                    self.current_theme,
                    self.current_theme
                );
                self.theme_set.themes.get("base16-ocean.dark")
            })
            .or_else(|| self.theme_set.themes.values().next())
        else {
            tracing::error!(
                "ThemeSet 为空，跳过代码高亮 / ThemeSet is empty, skipping code highlight"
            );
            return self.escape_html(code);
        };

        match syntax {
            Some(syntax) => {
                // 使用 syntect 生成 HTML / Use syntect to generate HTML
                match highlighted_html_for_string(code, self.syntax_set, &syntax, theme) {
                    Ok(html) => html,
                    Err(_) => self.escape_html(code),
                }
            }
            None => {
                // 没有找到语法，使用纯文本 / No syntax found, use plain text
                self.escape_html(code)
            }
        }
    }

    /// 获取语法 / Get syntax
    fn get_syntax(&self, language: &str) -> Option<syntect::parsing::SyntaxReference> {
        let language_lower = language.to_lowercase();

        // 常见语言别名映射 / Common language alias mapping
        let language_name = match language_lower.as_str() {
            // JavaScript 变体 / JavaScript variants
            "js" | "javascript" | "ecmascript" => "JavaScript",
            "ts" | "typescript" => "TypeScript",
            "jsx" => "JavaScript (JSX)",
            "tsx" => "TypeScript (TSX)",

            // Python
            "py" | "python" => "Python",

            // Rust
            "rs" | "rust" => "Rust",

            // C/C++
            "c" => "C",
            "cpp" | "c++" | "cc" => "C++",
            "h" | "hpp" => "C++",

            // Web
            "html" => "HTML",
            "css" => "CSS",
            "scss" | "sass" => "SCSS",
            "less" => "Less",

            // Data formats
            "json" => "JSON",
            "yaml" | "yml" => "YAML",
            "toml" => "TOML",
            "xml" => "XML",

            // Shell
            "sh" | "shell" | "bash" => "Bash",
            "zsh" => "Bash",
            "powershell" | "ps1" => "PowerShell",
            "bat" | "cmd" => "Batch File",

            // Other languages
            "java" => "Java",
            "kotlin" | "kt" => "Kotlin",
            "go" => "Go",
            "swift" => "Swift",
            "objc" | "objectivec" => "Objective-C",
            "scala" => "Scala",
            "ruby" | "rb" => "Ruby",
            "php" => "PHP",
            "csharp" | "cs" | "c#" => "C#",
            "fsharp" | "fs" | "f#" => "F#",
            "vb" | "visualbasic" => "Visual Basic",
            "lua" => "Lua",
            "perl" | "pl" => "Perl",
            "r" => "R",
            "sql" => "SQL",
            "dart" => "Dart",
            "elixir" => "Elixir",
            "erlang" => "Erlang",
            "haskell" => "Haskell",
            "lisp" => "Lisp",
            "clojure" => "Clojure",
            "ocaml" => "OCaml",
            "reason" => "Reason",
            "zig" => "Zig",
            "nim" => "Nim",
            "v" | "vlang" => "V",
            "crystal" => "Crystal",
            "d" => "D",
            "fortran" => "Fortran",
            "groovy" => "Groovy",
            "julia" => "Julia",
            "matlab" => "Matlab",
            "pascal" => "Pascal",
            "protobuf" => "Protocol Buffer",
            "solidity" => "Solidity",
            "vhdl" => "VHDL",
            "verilog" => "Verilog",

            // Markup
            "md" | "markdown" => "Markdown",
            "tex" | "latex" => "LaTeX",
            "rst" => "reStructuredText",
            "asciidoc" | "adoc" => "AsciiDoc",

            // Config
            "ini" => "INI",
            "conf" | "config" => "Config File",
            "dockerfile" | "docker" => "Dockerfile",
            "makefile" | "make" => "Makefile",
            "cmake" => "CMake",
            "gradle" => "Gradle",
            "maven" | "pom" => "XML",

            // Other
            "diff" | "patch" => "Diff",
            "log" => "Log file",
            "regex" => "Regular Expression",
            _ => language,
        };

        // 首先尝试精确匹配 / First try exact match
        if let Some(syntax) = self.syntax_set.find_syntax_by_name(language_name) {
            return Some(syntax.clone());
        }

        // 尝试通过扩展名匹配 / Try to match by extension
        if let Some(syntax) = self
            .syntax_set
            .find_syntax_by_extension(language_lower.as_str())
        {
            return Some(syntax.clone());
        }

        // 尝试模糊匹配 / Try fuzzy match
        for syntax in self.syntax_set.syntaxes() {
            if syntax.name.to_lowercase().contains(&language_lower) {
                return Some(syntax.clone());
            }
        }

        None
    }

    /// 转义 HTML / Escape HTML
    fn escape_html(&self, code: &str) -> String {
        let mut result = String::with_capacity(code.len() * 2);
        for c in code.chars() {
            match c {
                '&' => result.push_str("\x26amp;"),
                '<' => result.push_str("\x26lt;"),
                '>' => result.push_str("\x26gt;"),
                '"' => result.push_str("\x26quot;"),
                '\'' => result.push_str("\x26#39;"),
                _ => result.push(c),
            }
        }
        result
    }

    /// 生成带样式的代码块 HTML / Generate styled code block HTML
    pub fn highlight_code_block(&self, code: &str, language: &str) -> String {
        let highlighted = self.highlight_code(code, language);

        format!(
            r#"<pre class="code-block" data-language="{}"><code>{}</code></pre>"#,
            language, highlighted
        )
    }

    /// 获取内联 CSS 样式 / Get inline CSS styles
    pub fn get_code_block_css(&self) -> &'static str {
        r#"
.code-block {
    background: #1e1e1e;
    border-radius: 6px;
    padding: 16px;
    overflow-x: auto;
    font-family: 'Fira Code', 'JetBrains Mono', 'Consolas', 'Monaco', monospace;
    font-size: 14px;
    line-height: 1.5;
    margin: 12px 0;
    border: 1px solid #333;
}

.code-block code {
    background: transparent;
    padding: 0;
}

/* 滚动条样式 / Scrollbar styles */
.code-block::-webkit-scrollbar {
    height: 8px;
}

.code-block::-webkit-scrollbar-track {
    background: #2d2d2d;
    border-radius: 4px;
}

.code-block::-webkit-scrollbar-thumb {
    background: #555;
    border-radius: 4px;
}

.code-block::-webkit-scrollbar-thumb:hover {
    background: #666;
}

/* 代码高亮颜色 / Code highlighting colors */
.code-block .syntect-Color-FFFFFFFF { color: #ffffff; }
.code-block .syntect-Color-FFD75F00 { color: #d75f00; }
.code-block .syntect-Color-FFFF8700 { color: #ff8700; }
.code-block .syntect-Color-FFAF8700 { color: #af8700; }
.code-block .syntect-Color-FF87AF00 { color: #87af00; }
.code-block .syntect-Color-FF87FF00 { color: #87ff00; }
.code-block .syntect-Color-FFAFAF00 { color: #afaf00; }
.code-block .syntect-Color-FF008700 { color: #008700; }
.code-block .syntect-Color-FF00AF00 { color: #00af00; }
.code-block .syntect-Color-FF00FF00 { color: #00ff00; }
.code-block .syntect-Color-FF0087AF { color: #0087af; }
.code-block .syntect-Color-FF00AFAF { color: #00afaf; }
.code-block .syntect-Color-FF00FFFF { color: #00ffff; }
.code-block .syntect-Color-FF0087FF { color: #0087ff; }
.code-block .syntect-Color-FF00AFFF { color: #00afff; }
.code-block .syntect-Color-FF00D7FF { color: #00d7ff; }
.code-block .syntect-Color-FF875FFF { color: #875fff; }
.code-block .syntect-Color-FF8787FF { color: #8787ff; }
.code-block .syntect-Color-FF875FD7 { color: #875fd7; }
.code-block .syntect-Color-FFD75FFF { color: #d75fff; }
.code-block .syntect-Color-FFD787FF { color: #d787ff; }
.code-block .syntect-Color-FFFF0087 { color: #ff0087; }
.code-block .syntect-Color-FFFF5F87 { color: #ff5f87; }
.code-block .syntect-Color-FFFF00D7 { color: #ff00d7; }
.code-block .syntect-Color-FFFF5FD7 { color: #ff5fd7; }
.code-block .syntect-Color-FFFF87D7 { color: #ff87d7; }
"#
    }
}

impl Default for SyntaxHighlightService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlight_service_creation() {
        let service = SyntaxHighlightService::new();
        assert!(!service.theme_set.themes.is_empty());
    }

    #[test]
    fn test_highlight_javascript() {
        let service = SyntaxHighlightService::new();
        let code = "const x = 42;";
        let result = service.highlight_code(code, "javascript");
        assert!(result.contains("const") || result.contains("42"));
    }

    #[test]
    fn test_highlight_rust() {
        let service = SyntaxHighlightService::new();
        let code = "fn main() { println!(\"Hello\"); }";
        let result = service.highlight_code(code, "rust");
        assert!(result.contains("fn") || result.contains("println"));
    }

    #[test]
    fn test_highlight_python() {
        let service = SyntaxHighlightService::new();
        let code = "def hello():\n    print('world')";
        let result = service.highlight_code(code, "python");
        assert!(result.contains("def") || result.contains("print"));
    }

    #[test]
    fn test_highlight_unknown_language() {
        let service = SyntaxHighlightService::new();
        let code = "some random text";
        let result = service.highlight_code(code, "unknown_lang_xyz");
        // 应该返回转义后的文本 / Should return escaped text
        assert!(result.contains("some random text"));
    }

    #[test]
    fn test_escape_html() {
        let service = SyntaxHighlightService::new();
        let code = "<script>alert('xss')</script>";
        let result = service.escape_html(code);
        // 检查 HTML 实体已转义 / Check HTML entities are escaped
        assert!(result.contains("\x26lt;script\x26gt;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_code_block_html() {
        let service = SyntaxHighlightService::new();
        let code = "let x = 1;";
        let result = service.highlight_code_block(code, "rust");
        assert!(result.contains(r#"<pre class="code-block""#));
        assert!(result.contains(r#"data-language="rust""#));
    }

    #[test]
    fn test_supported_languages() {
        let service = SyntaxHighlightService::new();
        assert!(service.theme_set.themes.contains_key("base16-ocean.dark"));
    }

    #[test]
    fn test_language_aliases() {
        let service = SyntaxHighlightService::new();
        // 测试别名 / Test aliases
        let _result1 = service.highlight_code("code", "js");
        let _result2 = service.highlight_code("code", "ts");
        let _result3 = service.highlight_code("code", "py");
    }
}
