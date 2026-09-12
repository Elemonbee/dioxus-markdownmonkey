//! HTML 导出 / HTML Export

use super::shared::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// HTML 导出外观选项 / HTML export appearance options
#[derive(Debug, Clone)]
pub struct HtmlExportOptions {
    /// `html lang` 属性 / `html lang` attribute
    pub language: String,
    /// 是否使用深色样式 / Whether to use dark styles
    pub dark: bool,
}

impl Default for HtmlExportOptions {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            dark: false,
        }
    }
}

/// 导出为 HTML / Export to HTML
#[allow(dead_code)] // 经 ExportService 与无资源导出路径使用 / Via ExportService / no-assets path
pub fn export_to_html(markdown_content: &str, output_path: &Path) -> Result<(), ExportError> {
    export_to_html_with_assets(markdown_content, output_path, None)
}

/// 导出 HTML，并将本地图片复制到 `{stem}_files/` 旁路目录
/// Export HTML and copy local images into a `{stem}_files/` sidecar folder
pub fn export_to_html_with_assets(
    markdown_content: &str,
    output_path: &Path,
    source_dir: Option<&Path>,
) -> Result<(), ExportError> {
    export_to_html_with_options(
        markdown_content,
        output_path,
        source_dir,
        &HtmlExportOptions::default(),
    )
}

/// 按语言与主题导出 HTML / Export HTML with language and theme
pub fn export_to_html_with_options(
    markdown_content: &str,
    output_path: &Path,
    source_dir: Option<&Path>,
    options: &HtmlExportOptions,
) -> Result<(), ExportError> {
    use crate::services::markdown::MarkdownService;

    let html_content = MarkdownService::new().render(markdown_content);
    let html_content = rewrite_and_bundle_images(&html_content, source_dir, output_path)?;
    let lang = sanitize_html_lang(&options.language);
    let (fg, bg, muted, border, code_bg, syn_keyword, syn_string, syn_number, syn_fn, syn_type) =
        if options.dark {
            (
                "#e6edf3", "#0d1117", "#8b949e", "#30363d", "#161b22", "#89b4fa", "#a6e3a1",
                "#fab387", "#74c7ec", "#cba6f7",
            )
        } else {
            (
                "#333333", "#ffffff", "#666666", "#dddddd", "#f4f4f4", "#005cc5", "#22863a",
                "#b35c00", "#0088aa", "#6f42c1",
            )
        };

    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="color-scheme" content="{color_scheme}">
    <title>Markdown Export</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            line-height: 1.6;
            color: {fg};
            background: {bg};
        }}
        .markdown-body pre {{
            background: {code_bg};
            padding: 16px;
            border-radius: 4px;
            overflow-x: auto;
        }}
        .markdown-body code {{
            background: {code_bg};
            padding: 2px 6px;
            border-radius: 3px;
        }}
        .markdown-body blockquote {{
            border-left: 4px solid {border};
            margin: 0;
            padding-left: 16px;
            color: {muted};
        }}
        .markdown-body table {{
            border-collapse: collapse;
            width: 100%;
        }}
        .markdown-body th, .markdown-body td {{
            border: 1px solid {border};
            padding: 8px;
            text-align: left;
        }}
        .markdown-body th {{
            background: {code_bg};
        }}
        .markdown-body img {{
            max-width: 100%;
        }}
        .markdown-body pre.code-block {{
            position: relative;
        }}
        .markdown-body pre.code-block[data-lang]:not([data-lang=""])::before {{
            content: attr(data-lang);
            float: right;
            font-size: 11px;
            color: {muted};
        }}
        .highlight-code .keyword, .highlight-code .storage {{ color: {syn_keyword}; }}
        .highlight-code .string {{ color: {syn_string}; }}
        .highlight-code .comment {{ color: {muted}; font-style: italic; }}
        .highlight-code .constant, .highlight-code .constant-numeric {{ color: {syn_number}; }}
        .highlight-code .entity, .highlight-code .entity-name-function {{ color: {syn_fn}; }}
        .highlight-code .storage-type, .highlight-code .support-type {{ color: {syn_type}; }}
        .math-display {{ display: block; margin: 1em 0; text-align: center; overflow-x: auto; }}
        pre.mermaid {{ background: transparent; text-align: center; }}
        pre.mermaid svg {{ max-width: 100%; height: auto; }}
    </style>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.css">
</head>
<body>
<article class="markdown-body">
{html_content}
</article>
<script src="https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.js"></script>
<script src="https://cdn.jsdelivr.net/npm/mermaid@11.4.1/dist/mermaid.min.js"></script>
<script>
(function() {{
    var dark = {export_dark};
    var root = document.querySelector('.markdown-body');
    if (!root) return;
    if (window.katex) {{
        root.querySelectorAll('.math-inline, .math-display').forEach(function(el) {{
            try {{
                window.katex.render(el.textContent || '', el, {{
                    throwOnError: false,
                    displayMode: el.classList.contains('math-display'),
                    output: 'html',
                    trust: false
                }});
            }} catch (e) {{}}
        }});
    }}
    if (window.mermaid) {{
        window.mermaid.initialize({{ startOnLoad: false, securityLevel: 'strict', theme: dark ? 'dark' : 'default' }});
        window.mermaid.run({{ querySelector: '.markdown-body pre.mermaid', suppressErrors: true }});
    }}
}})();
</script>
</body>
</html>"#,
        color_scheme = if options.dark { "dark" } else { "light" },
        export_dark = if options.dark { "true" } else { "false" },
    );

    fs::write(output_path, full_html)?;
    Ok(())
}

/// 将语言标签限制为安全的 HTML lang 值 / Restrict language tags to a safe HTML lang value
fn sanitize_html_lang(language: &str) -> &'static str {
    match language {
        "en-US" | "en" => "en",
        _ => "zh-CN",
    }
}

/// 资源旁路目录名：`note.html` → `note_files`
/// Sidecar asset folder name: `note.html` → `note_files`
fn assets_dir_for_output(output_path: &Path) -> PathBuf {
    let stem = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    output_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}_files"))
}

/// 将 HTML 中本地 `<img src>` 复制到旁路目录并改写相对路径
/// Copy local `<img src>` assets into the sidecar folder and rewrite relative paths
pub fn rewrite_and_bundle_images(
    html: &str,
    source_dir: Option<&Path>,
    output_path: &Path,
) -> Result<String, ExportError> {
    static IMG_SRC_RE: OnceLock<Regex> = OnceLock::new();
    let re = IMG_SRC_RE.get_or_init(|| {
        Regex::new(r#"(?i)(<img\b[^>]*?\bsrc\s*=\s*)(?:"([^"]+)"|'([^']+)')"#)
            .expect("img src regex")
    });

    let Some(base) = source_dir.filter(|p| p.is_dir()) else {
        return Ok(html.to_string());
    };

    let mut assets_dir: Option<PathBuf> = None;
    let mut copied: HashMap<PathBuf, String> = HashMap::new();
    let mut counter = 0usize;
    let mut out = String::with_capacity(html.len());
    let mut last = 0;

    for cap in re.captures_iter(html) {
        let full = cap.get(0).unwrap();
        let prefix = cap.get(1).unwrap().as_str();
        let (src, quote) = if let Some(d) = cap.get(2) {
            (d.as_str(), "\"")
        } else if let Some(s) = cap.get(3) {
            (s.as_str(), "'")
        } else {
            out.push_str(full.as_str());
            last = full.end();
            continue;
        };

        out.push_str(&html[last..full.start()]);
        last = full.end();

        if is_remote_or_data_url(src) {
            out.push_str(full.as_str());
            continue;
        }

        let abs = resolve_image_path(base, src);
        let Some(abs) = abs else {
            out.push_str(full.as_str());
            continue;
        };
        if !abs.is_file() {
            tracing::warn!("HTML export: image not found: {:?}", abs);
            out.push_str(full.as_str());
            continue;
        }

        if assets_dir.is_none() {
            let dir = assets_dir_for_output(output_path);
            fs::create_dir_all(&dir)?;
            assets_dir = Some(dir);
        }
        let dir = assets_dir.as_ref().unwrap();

        let rel_name = if let Some(existing) = copied.get(&abs) {
            existing.clone()
        } else {
            let file_name = abs
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("image.bin");
            let unique = format!("{counter}_{file_name}");
            counter += 1;
            let dest = dir.join(&unique);
            fs::copy(&abs, &dest)?;
            let folder = dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("export_files");
            let rel = format!("{folder}/{unique}");
            copied.insert(abs, rel.clone());
            rel
        };

        out.push_str(prefix);
        out.push_str(quote);
        out.push_str(&rel_name);
        out.push_str(quote);
    }

    out.push_str(&html[last..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bundle_local_image_rewrites_src() {
        let dir = TempDir::new().unwrap();
        let img = dir.path().join("pic.png");
        fs::write(&img, [0x89, 0x50, 0x4E, 0x47]).unwrap();
        let out = dir.path().join("out.html");

        let html = r#"<p><img src="pic.png" alt="x"></p>"#;
        let rewritten = rewrite_and_bundle_images(html, Some(dir.path()), &out).unwrap();
        assert!(rewritten.contains("out_files/"));
        assert!(rewritten.contains("pic.png"));
        assert!(!rewritten.contains("src=\"pic.png\""));

        let assets = dir.path().join("out_files");
        assert!(assets.is_dir());
        let entries: Vec<_> = fs::read_dir(&assets).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_skips_remote_images() {
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("out.html");
        let html = r#"<img src="https://example.com/a.png">"#;
        let rewritten = rewrite_and_bundle_images(html, Some(dir.path()), &out).unwrap();
        assert_eq!(rewritten, html);
        assert!(!dir.path().join("out_files").exists());
    }

    #[test]
    fn test_export_includes_highlighted_rust() {
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("hl.html");
        export_to_html("```rust\nfn main() {}\n```", &out).unwrap();
        let html = fs::read_to_string(&out).unwrap();
        assert!(html.contains("code-block"));
        assert!(html.contains("data-lang=\"rust\""));
        assert!(html.contains("highlight-code") || html.contains("<span"));
    }

    #[test]
    fn test_export_includes_mermaid_and_math() {
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("diagram.html");
        export_to_html("```mermaid\ngraph TD\nA-->B\n```\n\n$a^2+b^2=c^2$", &out).unwrap();
        let html = fs::read_to_string(&out).unwrap();
        assert!(html.contains("class=\"mermaid\""));
        assert!(html.contains("math-inline"));
        assert!(html.contains("katex.min.js"));
        assert!(html.contains("mermaid.min.js"));
    }

    #[test]
    fn test_export_uses_language_and_dark_theme() {
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("styled.html");
        export_to_html_with_options(
            "# Hello",
            &out,
            None,
            &HtmlExportOptions {
                language: "en-US".to_string(),
                dark: true,
            },
        )
        .unwrap();
        let html = fs::read_to_string(&out).unwrap();
        assert!(html.contains("lang=\"en\""));
        assert!(html.contains("color-scheme\" content=\"dark\""));
        assert!(html.contains("#0d1117"));
    }
}
