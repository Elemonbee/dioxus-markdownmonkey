//! KaTeX 样式处理：只保留已打包的 woff2，并改写字体 URL
//! KaTeX CSS helpers: keep bundled woff2 only and rewrite font URLs

use regex::Regex;
use std::sync::OnceLock;

/// 去掉 woff/ttf 回退，避免 WebView 再请求不存在的文件
/// Drop woff/ttf fallbacks so the WebView does not request missing files
pub fn drop_katex_woff_ttf(css: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r#",url\(fonts/[^)]+\.woff\) format\("woff"\),url\(fonts/[^)]+\.ttf\) format\("truetype"\)"#)
            .expect("katex font fallback regex")
    });
    re.replace_all(css, "").into_owned()
}

/// 把 `url(fonts/Name.woff2)` 换成已解析的资源地址
/// Replace `url(fonts/Name.woff2)` with resolved asset URLs
pub fn replace_katex_font_urls(css: &str, pairs: &[(&str, &str)]) -> String {
    let mut out = css.to_string();
    for (name, url) in pairs {
        out = out.replace(&format!("url(fonts/{name})"), &format!("url({url})"));
    }
    out
}

/// 把公式计数器限制在预览/导出容器内
/// Scope equation counters to preview/export containers
pub fn scope_katex_counters(css: &str) -> String {
    css.replace(
        "body{counter-reset:katexEqnNo mmlEqnNo}",
        ".preview-content,.markdown-body{counter-reset:katexEqnNo mmlEqnNo}",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_woff_ttf_and_rewrites_woff2() {
        let raw = r#"@font-face{font-family:KaTeX_Main;src:url(fonts/KaTeX_Main-Regular.woff2) format("woff2"),url(fonts/KaTeX_Main-Regular.woff) format("woff"),url(fonts/KaTeX_Main-Regular.ttf) format("truetype")}body{counter-reset:katexEqnNo mmlEqnNo}"#;
        let css = scope_katex_counters(&replace_katex_font_urls(
            &drop_katex_woff_ttf(raw),
            &[(
                "KaTeX_Main-Regular.woff2",
                "/assets/KaTeX_Main-Regular.woff2",
            )],
        ));
        assert!(css.contains("@font-face"));
        assert!(css.contains("url(/assets/KaTeX_Main-Regular.woff2)"));
        assert!(!css.contains("url(fonts/"));
        assert!(!css.contains(".woff)"));
        assert!(css.contains(".preview-content,.markdown-body{counter-reset"));
    }
}
