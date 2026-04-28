//! AppState 与类型的单元测试 / Unit tests for AppState and state types

use dioxus::prelude::*;
use std::path::PathBuf;

use crate::state::app_state::AppState;
use crate::state::types::History as UndoHistory;
use crate::state::types::{
    AIConfig, AIProvider, Language, OutlineItem, SaveStatus, SidebarTab, TabInfo, Theme,
    LARGE_FILE_HISTORY_THRESHOLD, LARGE_FILE_MAX_HISTORY, MAX_HISTORY_SIZE,
};

// ========== History 测试 / History Tests ==========

#[test]
fn test_history_default() {
    let history = UndoHistory::default();
    assert!(history.past.is_empty());
    assert!(history.future.is_empty());
}

#[test]
fn test_history_push() {
    let mut history = UndoHistory::default();
    history.push("state1".to_string());
    history.push("state2".to_string());
    assert_eq!(history.past.len(), 2);
    assert_eq!(history.past[0].as_ref(), "state1");
    assert_eq!(history.past[1].as_ref(), "state2");
}

#[test]
fn test_history_is_different() {
    let mut history = UndoHistory::default();
    assert!(history.is_different("hello"));
    assert!(!history.is_different("hello")); // 相同内容 / Same content
    assert!(history.is_different("world"));
    assert!(!history.is_different("world")); // 相同内容 / Same content
}

#[test]
fn test_history_max_size() {
    let mut history = UndoHistory::default();
    // 插入超过最大容量的记录 / Insert more than max capacity
    for i in 0..(MAX_HISTORY_SIZE + 10) {
        history.push(format!("state_{}", i));
    }
    assert_eq!(history.past.len(), MAX_HISTORY_SIZE);
    // 最早的记录被移除 / Oldest records removed
    assert_eq!(history.past[0].as_ref(), format!("state_{}", 10));
}

#[test]
fn test_history_large_file_memory_optimization() {
    // 测试大文件时历史容量自动减少 / Test that history capacity is reduced for large files
    let mut history = UndoHistory::default();

    // 创建超过阈值的内容 (100KB) / Create content exceeding threshold
    let large_content: String = "x".repeat(LARGE_FILE_HISTORY_THRESHOLD + 1);

    // 插入大文件历史记录 / Insert large file history records
    for _i in 0..(LARGE_FILE_MAX_HISTORY + 5) {
        history.push(large_content.clone());
    }

    // 大文件时历史容量应该限制为 LARGE_FILE_MAX_HISTORY / History should be limited to LARGE_FILE_MAX_HISTORY
    assert_eq!(history.past.len(), LARGE_FILE_MAX_HISTORY);
    assert!(history.past.len() < MAX_HISTORY_SIZE);
}

#[test]
fn test_history_small_file_full_capacity() {
    // 测试小文件时历史容量保持最大 / Test that small files keep full history capacity
    let mut history = UndoHistory::default();

    // 创建小于阈值的内容 / Create content below threshold
    let small_content = "small content".to_string();
    assert!(small_content.len() < LARGE_FILE_HISTORY_THRESHOLD);

    // 插入小文件历史记录 / Insert small file history records
    for i in 0..MAX_HISTORY_SIZE {
        history.push(format!("{}_{}", small_content, i));
    }

    // 小文件时历史容量应该保持最大 / History should be at full capacity for small files
    assert_eq!(history.past.len(), MAX_HISTORY_SIZE);
}

// ========== TabInfo 测试 / TabInfo Tests ==========

#[test]
fn test_tab_info_new() {
    let tab = TabInfo::new("Test Tab");
    assert_eq!(tab.title, "Test Tab");
    assert!(tab.path.is_none());
    assert!(tab.content.is_empty());
    assert!(!tab.modified);
}

#[test]
fn test_tab_info_from_file() {
    let path = PathBuf::from("/docs/test.md");
    let tab = TabInfo::from_file(path.clone(), "# Hello".to_string());
    assert_eq!(tab.title, "test");
    assert_eq!(tab.path, Some(path));
    assert_eq!(tab.content, "# Hello");
    assert!(!tab.modified);
}

#[test]
fn test_tab_info_from_file_no_extension() {
    let path = PathBuf::from("/docs/README");
    let tab = TabInfo::from_file(path, "content".to_string());
    assert_eq!(tab.title, "README");
}

// ========== OutlineItem 测试 / OutlineItem Tests ==========

#[test]
fn test_outline_parsing() {
    let content = "# Title\n\n## Section 1\n\n### Subsection\n\n## Section 2";
    let items: Vec<OutlineItem> = content
        .lines()
        .enumerate()
        .filter_map(|(line_idx, line)| {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                if hash_count <= 6 && hash_count > 0 {
                    let text = trimmed[hash_count..].trim().to_string();
                    if !text.is_empty() {
                        return Some(OutlineItem {
                            level: hash_count as u8,
                            text,
                            line: line_idx,
                        });
                    }
                }
            }
            None
        })
        .collect();

    assert_eq!(items.len(), 4);
    assert_eq!(items[0].level, 1);
    assert_eq!(items[0].text, "Title");
    assert_eq!(items[1].level, 2);
    assert_eq!(items[1].text, "Section 1");
    assert_eq!(items[2].level, 3);
    assert_eq!(items[2].text, "Subsection");
    assert_eq!(items[3].level, 2);
    assert_eq!(items[3].text, "Section 2");
}

// ========== Theme 测试 / Theme Tests ==========

#[test]
fn test_theme_default() {
    assert_eq!(Theme::default(), Theme::Dark);
}

#[test]
fn test_language_default() {
    assert_eq!(Language::default(), Language::ZhCN);
}

#[test]
fn test_save_status_default() {
    assert_eq!(SaveStatus::default(), SaveStatus::Saved);
}

// ========== AIConfig 测试 / AIConfig Tests ==========

#[test]
fn test_ai_config_default() {
    let config = AIConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.provider, AIProvider::OpenAI);
    assert_eq!(config.model, "gpt-4o-mini");
    assert!(config.api_key.is_empty());
    assert_eq!(config.temperature, 0.7);
}

#[test]
fn test_ai_provider_default() {
    assert_eq!(AIProvider::default(), AIProvider::OpenAI);
}

// ========== SidebarTab 测试 / SidebarTab Tests ==========

#[test]
fn test_sidebar_tab_default() {
    assert_eq!(SidebarTab::default(), SidebarTab::Outline);
}

// ========== update_content → update_outline 集成测试 ==========

/// 在 Dioxus 作用域内运行依赖 Signal 的测试 / Run Signal-dependent tests inside a Dioxus scope
fn with_runtime<F: FnOnce()>(f: F) {
    let vdom = dioxus::prelude::VirtualDom::prebuilt(|| {
        rsx! { div {} }
    });
    let scope_id = dioxus::prelude::ScopeId::ROOT;
    vdom.in_scope(scope_id, f);
}

#[test]
fn test_update_content_updates_outline() {
    with_runtime(|| {
        let mut state = AppState::new();
        assert!(state.outline_items.read().is_empty());

        state.update_content("# Title\n\n## Section 1\n\nSome text\n\n## Section 2".to_string());

        let items = state.outline_items.read();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].level, 1);
        assert_eq!(items[0].text, "Title");
        assert_eq!(items[0].line, 0);
        assert_eq!(items[1].level, 2);
        assert_eq!(items[1].text, "Section 1");
        assert_eq!(items[1].line, 2);
        assert_eq!(items[2].level, 2);
        assert_eq!(items[2].text, "Section 2");
        assert_eq!(items[2].line, 6);
    });
}

#[test]
fn test_update_content_empty_clears_outline() {
    with_runtime(|| {
        let mut state = AppState::new();
        state.update_content("# Title\n## Sub".to_string());
        assert_eq!(state.outline_items.read().len(), 2);

        state.update_content(String::new());
        assert!(state.outline_items.read().is_empty());
    });
}

#[test]
fn test_update_content_no_headings() {
    with_runtime(|| {
        let mut state = AppState::new();
        state.update_content("Just some text\nwith no headings".to_string());
        assert!(state.outline_items.read().is_empty());
    });
}

#[test]
fn test_update_content_nested_headings() {
    with_runtime(|| {
        let mut state = AppState::new();
        state.update_content("# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6".to_string());

        let items = state.outline_items.read();
        assert_eq!(items.len(), 6);
        for i in 0..6 {
            assert_eq!(items[i].level, (i + 1) as u8);
        }
    });
}

#[test]
fn test_update_content_skips_empty_heading() {
    with_runtime(|| {
        let mut state = AppState::new();
        state.update_content("# \n## Valid\n###".to_string());

        let items = state.outline_items.read();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text, "Valid");
    });
}

#[test]
fn test_update_content_dedup_same_content() {
    with_runtime(|| {
        let mut state = AppState::new();
        state.update_content("# Title".to_string());
        assert_eq!(state.history.read().past.len(), 1);

        state.update_content("# Title".to_string());
        assert_eq!(state.history.read().past.len(), 1);
    });
}

#[test]
fn test_update_content_tracks_modified() {
    with_runtime(|| {
        let mut state = AppState::new();
        assert!(!*state.modified.read());

        state.update_content("# Changed".to_string());
        assert!(*state.modified.read());
        assert_eq!(*state.save_status.read(), SaveStatus::Unsaved);
    });
}

#[test]
fn test_update_content_undo_redo_outline() {
    with_runtime(|| {
        let mut state = AppState::new();

        state.update_content("# V1".to_string());
        assert_eq!(state.outline_items.read().len(), 1);

        state.update_content("# V2\n## Sub".to_string());
        assert_eq!(state.outline_items.read().len(), 2);

        let undone = state.undo();
        assert!(undone);
        {
            let items = state.outline_items.read();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].text, "V1");
        }

        let redone = state.redo();
        assert!(redone);
        {
            let items = state.outline_items.read();
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].text, "V2");
            assert_eq!(items[1].text, "Sub");
        }
    });
}

#[test]
fn test_update_content_heading_line_numbers() {
    with_runtime(|| {
        let mut state = AppState::new();
        state.update_content("line0\nline1\n# Title\n\n## Sub\n".to_string());

        let items = state.outline_items.read();
        assert_eq!(items[0].line, 2);
        assert_eq!(items[1].line, 4);
    });
}
