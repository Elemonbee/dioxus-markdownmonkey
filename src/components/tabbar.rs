//! 标签栏组件 / Tab Bar Component
//!
//! 遵循 PAL 架构：使用 Actions 处理业务逻辑

use crate::actions::shortcut_actions::ShortcutActions;
use crate::actions::FileActions;
use crate::components::icons::{CloseIcon, PlusIcon};
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::{ReadableExt, *};

/// 标签栏组件 / Tab Bar Component
#[component]
pub fn TabBar() -> Element {
    // 所有 hooks 在顶部
    let mut state = use_context::<AppState>();
    let doc = state.document();
    let ui = state.ui();

    // 使用 use_effect 初始化标签，避免在渲染中修改状态
    let tabs_empty = doc.tabs.read().is_empty();
    use_effect(move || {
        if tabs_empty {
            state.init_first_tab();
        }
    });

    // i18n
    let lang = *ui.language.read();
    let new_tab_t = t("new_tab", lang);
    let aria_open_tabs_t = t("aria_open_tabs", lang);
    let modifier = ShortcutActions::primary_modifier_label();

    // 获取标签数据
    let tabs_data: Vec<(usize, crate::state::TabId, String, bool, bool)> = {
        let tabs = doc.tabs.read();
        let current_index = *doc.current_tab_index.read();
        tabs.iter()
            .enumerate()
            .map(|(i, tab)| {
                (
                    i,
                    tab.id,
                    tab.title.clone(),
                    tab.modified,
                    i == current_index,
                )
            })
            .collect()
    };

    rsx! {
        div { class: "tabbar", role: "tablist", "aria-label": "{aria_open_tabs_t}",
            for (index, tab_id, title, modified, is_active) in tabs_data {
                TabItem {
                    key: "{tab_id}",
                    index: index,
                    title: title,
                    modified: modified,
                    is_active: is_active,
                }
            }

            // 新建标签按钮
            button {
                class: "tab-new",
                title: "{new_tab_t} ({modifier}+N)",
                onclick: move |_| {
                    let mut state = state;
                    spawn(async move {
                        FileActions::new_tab_flushed(&mut state).await;
                    });
                },
                PlusIcon { size: 16 }
            }
        }
    }
}

/// 标签项属性
#[derive(Props, Clone, PartialEq)]
struct TabItemProps {
    index: usize,
    title: String,
    modified: bool,
    is_active: bool,
}

/// 标签项组件
fn TabItem(props: TabItemProps) -> Element {
    // hooks 在顶部
    let state = use_context::<AppState>();
    let ui = state.ui();

    // i18n
    let lang = *ui.language.read();
    let close_tab_t = t("close_tab", lang);

    // 计算 CSS
    let tab_class = if props.is_active { "tab active" } else { "tab" };
    let modified_display = if props.modified { "" } else { "display: none;" };
    let index = props.index;

    rsx! {
        div {
            class: "{tab_class}",
            role: "tab",
            aria_selected: "{props.is_active}",
            tabindex: if props.is_active { "0" } else { "-1" },
            onclick: move |_| {
                let mut state = state;
                spawn(async move {
                    FileActions::switch_tab_flushed(&mut state, index).await;
                });
            },

            span { class: "tab-title", "{props.title}" }
            // 修改标记 - 始终渲染
            span {
                class: "tab-modified",
                style: "{modified_display}",
                "●"
            }
            button {
                class: "tab-close",
                title: "{close_tab_t}",
                "aria-label": "{close_tab_t} {props.title}",
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                    let mut state = state;
                    spawn(async move {
                        FileActions::request_close_tab_flushed(&mut state, index).await;
                    });
                },
                CloseIcon { size: 14 }
            }
        }
    }
}
