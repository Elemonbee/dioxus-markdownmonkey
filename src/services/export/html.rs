//! HTML 导出 / HTML Export

use super::shared::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    use crate::services::markdown::MarkdownService;

    let html_content = MarkdownService::new().render(markdown_content);
    let html_content = rewrite_and_bundle_images(&html_content, source_dir, output_path)?;

    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Markdown Export</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            line-height: 1.6;
            color: #333;
        }}
        .markdown-body pre {{
            background: #f4f4f4;
            padding: 16px;
            border-radius: 4px;
            overflow-x: auto;
        }}
        .markdown-body code {{
            background: #f4f4f4;
            padding: 2px 6px;
            border-radius: 3px;
        }}
        .markdown-body blockquote {{
            border-left: 4px solid #ddd;
            margin: 0;
            padding-left: 16px;
            color: #666;
        }}
        .markdown-body table {{
            border-collapse: collapse;
            width: 100%;
        }}
        .markdown-body th, .markdown-body td {{
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }}
        .markdown-body th {{
            background: #f4f4f4;
        }}
        .markdown-body img {{
            max-width: 100%;
        }}
    </style>
</head>
<body>
<article class="markdown-body">
{}
</article>
</body>
</html>"#,
        html_content
    );

    fs::write(output_path, full_html)?;
    Ok(())
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
}
