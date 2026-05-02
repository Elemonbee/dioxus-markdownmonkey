//! 状态栏组件 / Status Bar Component
//!
//! 遵循 PAL 架构：使用 Actions 处理业务逻辑

use crate::state::{AppState, SaveStatus};
use crate::utils::i18n::t;
use dioxus::prelude::{ReadableExt, *};

/// 状态栏组件 / Status Bar Component
#[component]
pub fn StatusBar() -> Element {
    // 所有 hooks 在顶部
    let state = use_context::<AppState>();

    // 读取状态
    let modified = *state.modified.read();
    let save_status_val = *state.save_status.read();
    let theme_val = *state.theme.read();
    let lang = *state.language.read();

    // 计算统计数据
    let char_count = state.char_count();
    let word_count = state.word_count();
    let read_time = state.read_time();

    // 计算文本 (i18n)
    let save_status_text = match save_status_val {
        SaveStatus::Saved => t("saved", lang),
        SaveStatus::Saving => t("saving", lang),
        SaveStatus::Unsaved => t("unsaved", lang),
    };

    let theme_text = match theme_val {
        crate::state::Theme::Dark => t("dark", lang),
        crate::state::Theme::Light => t("light", lang),
        crate::state::Theme::System => t("system", lang),
    };

    let modified_text = t("modified", lang);
    let chars_label = t("chars", lang);
    let words_label = t("words", lang);
    let read_label = t("read", lang);
    let theme_label = t("theme", lang);
    let min_label = t("min", lang);
    let encoding_text = t("encoding_utf8", lang);
    let filetype_text = t("file_type_markdown", lang);

    let spell_enabled = *state.spell_check_enabled.read();
    let spell_count = state.spell_check_results.read().len();
    let spell_text = t("spell_errors", lang);

    // CSS 计算
    let modified_display = if modified { "" } else { "display: none;" };

    rsx! {
        div { class: "statusbar",
            // 左侧状态
            div { class: "statusbar-left",
                span { class: "status-item", "{save_status_text}" }
                // 已修改标记 - 始终渲染
                span {
                    class: "status-item modified",
                    style: "{modified_display}",
                    "{modified_text}"
                }
            }

            // 中间统计
            div { class: "statusbar-center",
                span { class: "status-item", "{chars_label}: {char_count}" }
                span { class: "status-item", "{words_label}: {word_count}" }
                span { class: "status-item", "{read_label}: {read_time}{min_label}" }
                if spell_enabled && spell_count > 0 {
                    span { class: "status-item spell-errors",
                        "{spell_count} {spell_text}"
                    }
                }
            }

            // 右侧设置
            div { class: "statusbar-right",
                span { class: "status-item", "{theme_label}: {theme_text}" }
                span { class: "status-item", "{encoding_text}" }
                span { class: "status-item", "{filetype_text}" }
            }
        }
    }
}
