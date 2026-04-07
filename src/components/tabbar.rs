//! 标签栏组件 / Tab Bar Component
//!
//! 遵循 PAL 架构：使用 Actions 处理业务逻辑

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

    // 使用 use_effect 初始化标签，避免在渲染中修改状态
    let tabs_empty = state.tabs.read().is_empty();
    use_effect(move || {
        if tabs_empty {
            state.init_first_tab();
        }
    });

    // i18n
    let lang = *state.language.read();
    let new_tab_t = t("new_tab", lang);

    // 获取标签数据
    let tabs_data: Vec<(usize, String, bool, bool)> = {
        let tabs = state.tabs.read();
        let current_index = *state.current_tab_index.read();
        tabs.iter()
            .enumerate()
            .map(|(i, tab)| (i, tab.title.clone(), tab.modified, i == current_index))
            .collect()
    };

    rsx! {
        div { class: "tabbar",
            for (index, title, modified, is_active) in tabs_data {
                TabItem {
                    key: "{index}",
                    index: index,
                    title: title,
                    modified: modified,
                    is_active: is_active,
                }
            }

            // 新建标签按钮
            button {
                class: "tab-new",
                title: "{new_tab_t} (Ctrl+N)",
                onclick: move |_| {
                    FileActions::new_tab(&mut state);
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
    let mut state = use_context::<AppState>();

    // i18n
    let lang = *state.language.read();
    let close_tab_t = t("close_tab", lang);

    // 计算 CSS
    let tab_class = if props.is_active { "tab active" } else { "tab" };
    let modified_display = if props.modified { "" } else { "display: none;" };
    let index = props.index;

    rsx! {
        div {
            class: "{tab_class}",
            onclick: move |_| {
                FileActions::switch_tab(&mut state, index);
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
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                    let should_confirm = {
                        let tabs = state.tabs.read();
                        tabs.get(index).map(|tab| tab.modified).unwrap_or(false)
                    } || *state.modified.read();
                    if should_confirm {
                        *state.pending_close_tab_index.write() = Some(index);
                        *state.show_close_confirm.write() = true;
                    } else {
                        state.close_tab(index);
                    }
                },
                CloseIcon { size: 14 }
            }
        }
    }
}
