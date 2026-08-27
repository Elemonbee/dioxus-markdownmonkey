//! 文档内搜索替换 Actions / In-document search and replace actions

use crate::actions::EditorActions;
use crate::state::AppState;
use crate::utils::replace::{count_matches, replace_all_in_text, replace_nth_match};
use dioxus::prelude::{ReadableExt, WritableExt};

/// 搜索替换 Actions / Search & replace actions
pub struct SearchActions;

impl SearchActions {
    /// 显示搜索条 / Show the search bar
    pub fn show(state: &mut AppState) {
        *state.ui().show_search.write() = true;
    }

    /// 隐藏搜索条 / Hide the search bar
    pub fn hide(state: &mut AppState) {
        *state.ui().show_search.write() = false;
    }

    /// 显示工作区搜索 / Show workspace search
    pub fn show_global(state: &mut AppState) {
        *state.ui().show_global_search.write() = true;
    }

    /// 按当前查询重算匹配数 / Recount matches for the current query
    fn recount(state: &mut AppState) {
        let (query, case_insensitive, use_regex) = {
            let ui = state.ui();
            let query = ui.search_query.read().clone();
            let case_insensitive = *ui.search_case_insensitive.read();
            let use_regex = *ui.search_regex.read();
            (query, case_insensitive, use_regex)
        };
        if query.is_empty() {
            let mut ui = state.ui();
            *ui.search_total.write() = 0;
            *ui.search_index.write() = 0;
            return;
        }
        let content = state.document().content.read().clone();
        let count = count_matches(&content, &query, case_insensitive, use_regex);
        let mut ui = state.ui();
        *ui.search_total.write() = count;
        *ui.search_index.write() = if count > 0 { 1 } else { 0 };
    }

    /// 更新查找词并重算 / Update the find query and recount
    pub fn set_query(state: &mut AppState, query: String) {
        *state.ui().search_query.write() = query;
        Self::recount(state);
    }

    /// 更新替换词 / Update the replacement text
    pub fn set_replace_query(state: &mut AppState, replacement: String) {
        *state.ui().replace_query.write() = replacement;
    }

    /// 切换大小写敏感并重算 / Toggle case-insensitivity and recount
    pub fn toggle_case_insensitive(state: &mut AppState) {
        let next = {
            let ui = state.ui();
            let current = *ui.search_case_insensitive.read();
            !current
        };
        *state.ui().search_case_insensitive.write() = next;
        Self::recount(state);
    }

    /// 切换正则并重算 / Toggle regex mode and recount
    pub fn toggle_regex(state: &mut AppState) {
        let next = {
            let ui = state.ui();
            let current = *ui.search_regex.read();
            !current
        };
        *state.ui().search_regex.write() = next;
        Self::recount(state);
    }

    /// 上一个匹配 / Previous match
    pub fn prev_match(state: &mut AppState) {
        let mut ui = state.ui();
        let total = *ui.search_total.read();
        if total == 0 {
            return;
        }
        let current = *ui.search_index.read();
        *ui.search_index.write() = if current <= 1 { total } else { current - 1 };
    }

    /// 下一个匹配 / Next match
    pub fn next_match(state: &mut AppState) {
        let mut ui = state.ui();
        let total = *ui.search_total.read();
        if total == 0 {
            return;
        }
        let current = *ui.search_index.read();
        *ui.search_index.write() = if current >= total { 1 } else { current + 1 };
    }

    /// flush 后替换当前匹配 / Flush then replace the current match
    pub async fn replace_current(state: &mut AppState) {
        EditorActions::flush_from_dom(state).await;
        let (query, replacement, idx, case_insensitive, use_regex) = {
            let ui = state.ui();
            let query = ui.search_query.read().clone();
            let replacement = ui.replace_query.read().clone();
            let idx = *ui.search_index.read();
            let case_insensitive = *ui.search_case_insensitive.read();
            let use_regex = *ui.search_regex.read();
            (query, replacement, idx, case_insensitive, use_regex)
        };
        if query.is_empty() || idx == 0 {
            return;
        }
        let content = state.document().content.read().clone();
        let new_content = replace_nth_match(
            &content,
            &query,
            &replacement,
            idx,
            case_insensitive,
            use_regex,
        );
        state.update_content(new_content);
        EditorActions::push_to_dom(&state.document().content.read());
        Self::recount(state);
    }

    /// flush 后全部替换 / Flush then replace all matches
    pub async fn replace_all(state: &mut AppState) {
        EditorActions::flush_from_dom(state).await;
        let (query, replacement, case_insensitive, use_regex) = {
            let ui = state.ui();
            let query = ui.search_query.read().clone();
            let replacement = ui.replace_query.read().clone();
            let case_insensitive = *ui.search_case_insensitive.read();
            let use_regex = *ui.search_regex.read();
            (query, replacement, case_insensitive, use_regex)
        };
        if query.is_empty() {
            return;
        }
        let content = state.document().content.read().clone();
        let new_content =
            replace_all_in_text(&content, &query, &replacement, case_insensitive, use_regex);
        state.update_content(new_content);
        EditorActions::push_to_dom(&state.document().content.read());
        Self::recount(state);
    }
}
