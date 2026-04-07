//! 预览组件 / Preview Component
//!
//! 遵循 PAL 架构，使用 Actions 处理理业务逻辑
//! 支持虚拟滚动行号提升大文件性能

use crate::config::PREVIEW_DEBOUNCE_MS;
use crate::services::markdown::{katex_script, mermaid_script, MarkdownService};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::{ReadableExt, *};

/// 预览组件 / Preview Component
#[component]
pub fn Preview() -> Element {
    let mut state = use_context::<AppState>();

    // 读取状态
    let show_preview = *state.show_preview.read();
    let sync_scroll = *state.sync_scroll.read();
    let content = state.content.read().clone();

    // 缓存：只在内容变化时重新渲染 HTML / Cache: only re-render HTML when content changes
    let mut cached_html = use_signal(String::new);
    let mut cached_content = use_signal(String::new);

    let content_changed = content != *cached_content.read();

    if content_changed {
        let content_clone = content.clone();

        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(PREVIEW_DEBOUNCE_MS)).await;

            if content_clone.is_empty() {
                cached_html.set(String::new());
            } else {
                let md_service = MarkdownService::new();
                let rendered = md_service.render_with_highlight(&content_clone);
                cached_html.set(rendered);
            }
            cached_content.set(content_clone);

            let _ = document::eval(
                r#"
            (function() {
                if (typeof mermaid !== 'undefined' && mermaid.run) {
                    try { mermaid.run(); } catch(e) {}
                }
                if (typeof MathJax !== 'undefined' && MathJax.typesetPromise) {
                    var preview = document.getElementById('preview-scroll');
                    if (preview) {
                        MathJax.typesetPromise([preview]).catch(function(err) {
                            console.warn('MathJax typeset error:', err);
                        });
                    }
                }
            })();
            "#,
            );
        });
    }

    let content_html = cached_html.read().clone();

    // 脚本注入标记
    let mut scripts_injected = use_signal(|| false);
    if !*scripts_injected.read() {
        scripts_injected.set(true);
        let scripts = format!("{} {}", mermaid_script(), katex_script());
        let _ = document::eval(&format!(
            "if (!document.getElementById('preview-scripts-done')) {{ var c = document.getElementById('preview-scripts'); if (c) {{ c.innerHTML = {}; c.id = 'preview-scripts-done'; }} }}",
            serde_json::to_string(&scripts).unwrap_or_default()
        ));
    }

    // i18n
    let lang = *state.language.read();
    let preview_t = t("preview", lang);
    let sync_t = t("sync_scroll_toggle", lang);

    let pane_class = if show_preview {
        "preview-pane"
    } else {
        "preview-pane preview-hidden"
    };

    let _ = use_effect(move || {
        let sync = *state.sync_scroll.read();
        let _ = document::eval(&format!(
            "if(window._mm_setSyncScroll) window._mm_setSyncScroll({});",
            sync
        ));
    });

    rsx! {
        div {
            class: "{pane_class}",

            div { class: "preview-header",
                span { "{preview_t}" }

                div { class: "preview-sync-toggle",
                    label { class: "sync-scroll-toggle",
                        input {
                            r#type: "checkbox",
                            checked: sync_scroll,
                            onchange: move |_| {
                                let current = *state.sync_scroll.read();
                                *state.sync_scroll.write() = !current;
                            },
                        },
                        span { class: "toggle-label", "{sync_t}" }
                    }
                }
            }

            div { id: "preview-scripts" }
            div {
                id: "preview-scroll",
                class: "preview-content markdown-body",
                onscroll: move |_| {
                    if !*state.sync_scroll.read() {
                        return;
                    }
                    let _ = document::eval(
                        "if(window._mm_reverseSyncScroll) window._mm_reverseSyncScroll();"
                    );
                },
                dangerous_inner_html: "{content_html}",
            }
        }
    }
}
