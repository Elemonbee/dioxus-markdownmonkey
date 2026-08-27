//! 预览组件 / Preview Component
//!
//! 遵循 PAL 架构，使用 Actions 处理理业务逻辑
//! 支持虚拟滚动行号提升大文件性能

use crate::actions::EditorActions;
use crate::config::{
    PREVIEW_DEBOUNCE_MS, PREVIEW_LARGE_FILE_DEBOUNCE_MS, PREVIEW_LARGE_FILE_THRESHOLD_BYTES,
};
use crate::services::markdown::MarkdownService;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::{ReadableExt, WritableExt, *};
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
    let doc = state.document();
    let ui = state.ui();

    // 读取状态（领域视图）/ Read state via domain views
    let show_preview = *ui.show_preview.read();
    let sync_scroll = *ui.sync_scroll.read();

    // 缓存：只在内容哈希变化时重新渲染 HTML / Cache: re-render only when content hash changes
    // 使用哈希替代完整内容字符串，节省大文件内存（Signal<String> → Signal<u64>）
    // Use hash instead of full content string to save memory for large files
    let mut cached_html = use_signal(String::new);
    let mut cached_content_hash = use_signal(|| 0u64);
    // 渲染世代号：丢弃过期的防抖任务，避免慢任务覆盖新内容
    // Render generation: discard stale debounce tasks so slow work cannot overwrite newer HTML
    let mut render_generation = use_signal(|| 0u64);
    // 防抖/渲染进行中提示 / Debounce/render-in-progress indicator
    let mut is_rendering = use_signal(|| false);

    // 先读哈希判断是否变化，避免无变化时的 O(n) 克隆
    // Read hash first to detect changes, avoiding O(n) clone when unchanged
    let content = doc.content.read();
    let hash = content_hash(&content);
    let content_changed = hash != *cached_content_hash.read();

    if content_changed {
        // 仅在变化时克隆 / Clone only when changed
        let content_clone = content.clone();
        let content_len = content.len();
        drop(content);

        // 立即占住哈希：打开文件等场景会连续触发多次重渲染，若等渲染完成再写哈希，
        // 防抖任务会被不断抬世代号而永远无法落地，预览会一直空白。
        // Claim hash immediately: opening a file triggers many re-renders; if we only
        // write the hash after render, debounce tasks keep getting superseded and the
        // preview stays blank forever.
        cached_content_hash.set(hash);

        let next_gen = *render_generation.read() + 1;
        render_generation.set(next_gen);
        is_rendering.set(!content_clone.is_empty());

        spawn(async move {
            // 大文件使用更长防抖，减少频繁渲染 / Longer debounce for large files
            let debounce_ms = if content_len > PREVIEW_LARGE_FILE_THRESHOLD_BYTES {
                PREVIEW_LARGE_FILE_DEBOUNCE_MS
            } else {
                PREVIEW_DEBOUNCE_MS
            };
            tokio::time::sleep(std::time::Duration::from_millis(debounce_ms)).await;

            // 防抖后若已有更新任务，丢弃本次结果 / Drop this result if a newer task was scheduled
            if *render_generation.read() != next_gen {
                return;
            }

            let rendered = if content_clone.is_empty() {
                String::new()
            } else {
                let md_service = MarkdownService::new();
                md_service.render(&content_clone)
            };

            // 渲染完成后再次校验世代号 / Re-check generation after potentially slow render
            if *render_generation.read() != next_gen {
                return;
            }

            cached_html.set(rendered);
            is_rendering.set(false);
        });
    }

    let content_html = cached_html.read().clone();
    let show_rendering = *is_rendering.read() && !doc.content.read().is_empty();

    // i18n
    let lang = *ui.language.read();
    let preview_t = t("preview", lang);
    let sync_t = t("sync_scroll_toggle", lang);
    let aria_preview_t = t("aria_preview", lang);
    let rendering_t = t("preview_rendering", lang);

    let pane_class = if show_preview {
        "preview-pane"
    } else {
        "preview-pane preview-hidden"
    };

    let _ = use_effect(move || {
        let sync = *ui.sync_scroll.read();
        let _ = document::eval(&format!(
            "if(window._mm_setSyncScroll) window._mm_setSyncScroll({});",
            sync
        ));
    });

    rsx! {
        div {
            class: "{pane_class}",
            role: "region",
            "aria-label": "{aria_preview_t}",

            div { class: "preview-header",
                span { "{preview_t}" }

                div { class: "preview-sync-toggle",
                    label { class: "sync-scroll-toggle",
                        input {
                            r#type: "checkbox",
                            checked: sync_scroll,
                            onchange: move |_| {
                                EditorActions::toggle_sync_scroll(&mut state);
                            },
                        },
                        span { class: "toggle-label", "{sync_t}" }
                    }
                }
            }

            div {
                id: "preview-scroll",
                class: "preview-content markdown-body",
                onscroll: move |_| {
                    if !*ui.sync_scroll.read() {
                        return;
                    }
                    let _ = document::eval(
                        "if(window._mm_reverseSyncScroll) window._mm_reverseSyncScroll();"
                    );
                },
                if show_rendering && content_html.is_empty() {
                    div {
                        class: "preview-rendering",
                        role: "status",
                        "aria-live": "polite",
                        "{rendering_t}"
                    }
                } else {
                    if show_rendering {
                        div {
                            class: "preview-rendering preview-rendering-banner",
                            role: "status",
                            "aria-live": "polite",
                            "{rendering_t}"
                        }
                    }
                    div {
                        class: "preview-html-root",
                        dangerous_inner_html: "{content_html}",
                    }
                }
            }
        }
    }
}
