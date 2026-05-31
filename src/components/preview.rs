//! 预览组件 / Preview Component
//!
//! 遵循 PAL 架构，使用 Actions 处理理业务逻辑
//! 支持虚拟滚动行号提升大文件性能

use crate::config::{
    PREVIEW_DEBOUNCE_MS, PREVIEW_LARGE_FILE_DEBOUNCE_MS, PREVIEW_LARGE_FILE_THRESHOLD_BYTES,
};
use crate::services::markdown::{katex_script, mermaid_script, MarkdownService};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::{ReadableExt, *};
use std::hash::{Hash, Hasher};

/// 计算内容哈希，用于 O(1) 变更检测 / Compute content hash for O(1) change detection
fn content_hash(s: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// 预览组件 / Preview Component
#[component]
pub fn Preview() -> Element {
    let mut state = use_context::<AppState>();

    // 读取状态
    let show_preview = *state.show_preview.read();
    let sync_scroll = *state.sync_scroll.read();

    // 缓存：只在内容哈希变化时重新渲染 HTML / Cache: re-render only when content hash changes
    // 使用哈希替代完整内容字符串，节省大文件内存（Signal<String> → Signal<u64>）
    // Use hash instead of full content string to save memory for large files
    let mut cached_html = use_signal(String::new);
    let mut cached_content_hash = use_signal(|| 0u64);

    // 先读哈希判断是否变化，避免无变化时的 O(n) 克隆
    // Read hash first to detect changes, avoiding O(n) clone when unchanged
    let content = state.content.read();
    let hash = content_hash(&content);
    let content_changed = hash != *cached_content_hash.read();

    if content_changed {
        // 仅在变化时克隆 / Clone only when changed
        let content_clone = content.clone();
        let content_len = content.len();
        drop(content);

        spawn(async move {
            // 大文件使用更长防抖，减少频繁渲染 / Longer debounce for large files
            let debounce_ms = if content_len > PREVIEW_LARGE_FILE_THRESHOLD_BYTES {
                PREVIEW_LARGE_FILE_DEBOUNCE_MS
            } else {
                PREVIEW_DEBOUNCE_MS
            };
            tokio::time::sleep(std::time::Duration::from_millis(debounce_ms)).await;

            if content_clone.is_empty() {
                cached_html.set(String::new());
            } else {
                let md_service = MarkdownService::new();
                let rendered = md_service.render_with_highlight(&content_clone);
                cached_html.set(rendered);
            }
            cached_content_hash.set(hash);

            let _ = document::eval(
                r#"
            (function() {
                if (typeof mermaid !== 'undefined' && mermaid.run) {
                    try { mermaid.run(); } catch(e) {}
                }
                if (window._mm_renderMath) {
                    try { window._mm_renderMath(); } catch(e) {
                        console.warn('KaTeX render error:', e);
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
        let scripts = format!("{} {}", mermaid_script(), katex_script())
            .replace("<script>", "")
            .replace("</script>", "");
        let _ = document::eval(&scripts);
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
            role: "region",
            "aria-label": "Preview",

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
