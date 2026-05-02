//! HTML 导出 / HTML Export

use super::shared::*;
use std::path::Path;

/// 导出为 HTML / Export to HTML
pub fn export_to_html(markdown_content: &str, output_path: &Path) -> Result<(), ExportError> {
    use crate::services::markdown::render_markdown;

    let html_content = render_markdown(markdown_content);

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
        pre {{
            background: #f4f4f4;
            padding: 16px;
            border-radius: 4px;
            overflow-x: auto;
        }}
        code {{
            background: #f4f4f4;
            padding: 2px 6px;
            border-radius: 3px;
        }}
        blockquote {{
            border-left: 4px solid #ddd;
            margin: 0;
            padding-left: 16px;
            color: #666;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }}
        th {{
            background: #f4f4f4;
        }}
        img {{
            max-width: 100%;
        }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
        html_content
    );

    std::fs::write(output_path, full_html)?;

    Ok(())
}
