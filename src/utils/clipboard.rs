//! 通过 WebView 复制文本到系统剪贴板
//! Copy text to the system clipboard through the WebView

use dioxus::document;

/// 将文本写入剪贴板（fire-and-forget）
/// Write text to the clipboard (fire-and-forget)
pub fn copy_text(text: &str) {
    let encoded = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
    let script = format!(
        r#"(function(text) {{
            try {{
                if (navigator.clipboard && navigator.clipboard.writeText) {{
                    navigator.clipboard.writeText(text).catch(function() {{ fallback(text); }});
                    return;
                }}
            }} catch (e) {{}}
            fallback(text);
            function fallback(t) {{
                var ta = document.createElement('textarea');
                ta.value = t;
                ta.setAttribute('readonly', '');
                ta.style.position = 'fixed';
                ta.style.left = '-9999px';
                document.body.appendChild(ta);
                ta.select();
                try {{ document.execCommand('copy'); }} catch (e) {{}}
                document.body.removeChild(ta);
            }}
        }})({encoded});"#
    );
    let _ = document::eval(&script);
}
