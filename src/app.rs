//! 主应用组件 / Main Application Component

use crate::actions::AppActions;
use crate::components::*;
use crate::config::{
    FILE_WATCH_ACTIVE_INTERVAL_MS, FILE_WATCH_IDLE_INTERVAL_SECS,
    FILE_WATCH_INTERNAL_WRITE_GRACE_MS,
};
use crate::services::auto_save::AutoSaveService;
use crate::services::file_watcher::FileModificationChecker;
use crate::services::keyring_service;
use crate::services::settings::load_settings;
use crate::state::AppState;
use crate::state::{AIProvider, Language, Theme};
use dioxus::prelude::*;

// 引入 CSS 样式（模块化）/ Import CSS Styles (Modular)
const ALL_CSS: &str = concat!(
    include_str!("styles/variables.css"),
    include_str!("styles/base.css"),
    include_str!("styles/toolbar.css"),
    include_str!("styles/sidebar.css"),
    include_str!("styles/editor.css"),
    include_str!("styles/modals.css"),
);

/// 检测系统主题 / Detect System Theme (Windows/macOS/Linux)
/// 失败时安全回退到深色主题 / Safely falls back to dark theme on failure
fn detect_system_theme() -> &'static str {
    match detect_system_theme_inner() {
        Some(theme) => theme,
        None => {
            tracing::warn!(
                "主题检测失败，使用默认深色主题 / Theme detection failed, using default dark theme"
            );
            "dark"
        }
    }
}

/// 内部主题检测实现 / Internal theme detection implementation
fn detect_system_theme_inner() -> Option<&'static str> {
    #[cfg(target_os = "windows")]
    {
        // Windows: 读取注册表检测系统主题 / Read registry to detect system theme
        use std::process::Command;
        if let Ok(output) = Command::new("reg")
            .args([
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "/v",
                "AppsUseLightTheme",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // 如果包含 0x0 则是深色主题 / 0x0 indicates dark theme
            if stdout.contains("0x0") || stdout.contains("0x0000") {
                return Some("dark");
            }
            // 如果包含 0x1 则是浅色主题 / 0x1 indicates light theme
            if stdout.contains("0x1") || stdout.contains("0x0001") {
                return Some("light");
            }
        }
        // 备用：检查系统颜色设置 / Fallback: check system color settings
        if let Ok(output) = Command::new("reg")
            .args([
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "/v",
                "SystemUsesLightTheme",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("0x0") {
                return Some("dark");
            }
            if stdout.contains("0x1") {
                return Some("light");
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 defaults 命令检测 / macOS: Use defaults command to detect
        use std::process::Command;
        if let Ok(output) = Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Dark") {
                return Some("dark");
            }
        }
        // 如果没有设置 AppleInterfaceStyle，说明是浅色模式 / No AppleInterfaceStyle means light mode
        return Some("light");
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: GNOME 优先读 color-scheme，再 gtk-theme；最后读常见环境变量 / Linux: GNOME color-scheme, gtk-theme, then env
        use std::process::Command;

        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if stdout.contains("prefer-dark") || stdout.contains("'dark'") {
                return Some("dark");
            }
            if stdout.contains("prefer-light") {
                return Some("light");
            }
        }

        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if stdout.contains("dark") {
                return Some("dark");
            }
        }

        fn env_lower(key: &str) -> Option<String> {
            std::env::var(key).ok().map(|s| s.to_lowercase())
        }

        if matches!(env_lower("GTK_THEME").as_deref(), Some(s) if s.contains("dark")) {
            return Some("dark");
        }
        match env_lower("COLOR_SCHEME").as_deref() {
            Some(s) if s == "dark" || s.contains("prefer-dark") => return Some("dark"),
            Some(s) if s == "light" || s.contains("prefer-light") => return Some("light"),
            _ => {}
        }
        if std::env::var("DARK_MODE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            return Some("dark");
        }
    }

    None
}

/// 主应用组件 / Main Application Component
pub fn App() -> Element {
    // 初始化全局状态，并加载保存的设置 / Initialize global state and load saved settings
    let state = use_context_provider(|| {
        let mut state = AppState::new();

        // 加载保存的设置 / Load saved settings
        let settings = load_settings();
        {
            // 应用主题 / Apply theme
            *state.theme.write() = match settings.theme.as_str() {
                "light" => Theme::Light,
                "system" => Theme::System,
                _ => Theme::Dark,
            };

            // 应用语言 / Apply language
            *state.language.write() = match settings.language.as_str() {
                "en-US" => Language::EnUS,
                _ => Language::ZhCN,
            };

            // 应用编辑器设置 / Apply editor settings
            *state.font_size.write() = settings.font_size;
            *state.preview_font_size.write() = settings.preview_font_size;
            *state.word_wrap.write() = settings.word_wrap;
            *state.line_numbers.write() = settings.line_numbers;
            *state.sync_scroll.write() = settings.sync_scroll;
            *state.sidebar_visible.write() = settings.sidebar_visible;
            *state.show_preview.write() = settings.show_preview;
            AppActions::set_sidebar_width(&mut state, settings.sidebar_width);
            *state.auto_save_enabled.write() = settings.auto_save_enabled;
            *state.auto_save_interval.write() = settings.auto_save_interval;

            // 应用拼写检查设置 / Apply spell check settings
            *state.spell_check_enabled.write() = settings.spell_check_enabled;
            if settings.spell_check_enabled {
                state.run_spell_check();
            }

            // 应用 AI 设置 / Apply AI settings
            {
                let mut config = state.ai_config.write();
                config.enabled = settings.ai.enabled;
                config.provider = match settings.ai.provider.as_str() {
                    "claude" => AIProvider::Claude,
                    "deepseek" => AIProvider::DeepSeek,
                    "kimi" => AIProvider::Kimi,
                    "ollama" => AIProvider::Ollama,
                    "openrouter" => AIProvider::OpenRouter,
                    _ => AIProvider::OpenAI,
                };
                config.model = settings.ai.model;

                // 优先从系统密钥环获取 API Key，回退到文件中的设置
                // 如果密钥环中没有，尝试从配置文件迁移
                // Try keyring first, fallback to settings file, then migrate to keyring
                config.api_key = match keyring_service::get_api_key(&settings.ai.provider) {
                    Ok(key) => key,
                    Err(_) => {
                        // 密钥环中没有，尝试迁移明文密钥
                        // No key in keyring, try to migrate plaintext key
                        let plaintext_key = settings.ai.api_key.as_deref();
                        keyring_service::migrate_api_key_if_needed(
                            &settings.ai.provider,
                            plaintext_key,
                        );

                        // 迁移后再次尝试从密钥环获取
                        match keyring_service::get_api_key(&settings.ai.provider) {
                            Ok(key) => key,
                            Err(_) => {
                                // 密钥环完全不可用，使用明文密钥但发出安全警告
                                // Keyring unavailable, use plaintext key but warn
                                if settings.ai.api_key.is_some() {
                                    tracing::warn!(
                                        "⚠️ 系统密钥环不可用，API Key 仅在内存中使用（不会写入磁盘）/ \
                                         System keyring unavailable, API Key only in memory (never written to disk)"
                                    );
                                }
                                settings.ai.api_key.unwrap_or_default()
                            }
                        }
                    }
                };

                config.base_url = settings.ai.base_url;
                config.system_prompt = settings.ai.system_prompt;
                config.temperature = settings.ai.temperature;
            }
        }

        state
    });

    // 获取当前主题 / Get current theme
    let theme = *state.theme.read();
    let theme_str = match theme {
        Theme::Light => "light",
        Theme::System => detect_system_theme(),
        Theme::Dark => "dark",
    };

    // 自动保存定时器（使用 AutoSaveService）/ Auto Save Timer (using AutoSaveService)
    {
        let state_clone = state;
        use_future(move || {
            let mut state = state_clone;
            let mut auto_saver = AutoSaveService::new();
            async move {
                loop {
                    // 检查自动保存状态 / Check auto-save status
                    let enabled = *state.auto_save_enabled.read();
                    let has_file = state.current_file.read().is_some();

                    // 动态调整休眠时间：活跃时 5s，空闲时 60s，减少 CPU 唤醒
                    // Dynamic sleep: 5s when active, 60s when idle, reduce CPU wakeups
                    let sleep_duration = if enabled && has_file {
                        std::time::Duration::from_secs(5)
                    } else {
                        std::time::Duration::from_secs(60)
                    };
                    tokio::time::sleep(sleep_duration).await;

                    // 再次检查（避免唤醒后立即执行）/ Re-check after sleep
                    if !enabled || !has_file {
                        continue;
                    }

                    // 同步设置到 AutoSaveService / Sync settings to AutoSaveService
                    let interval = *state.auto_save_interval.read();
                    let modified = *state.modified.read();

                    auto_saver.set_enabled(enabled);
                    auto_saver.set_interval(interval);

                    // 检查是否需要保存 / Check if save is needed
                    if auto_saver.should_save(modified) {
                        let path = state.current_file.read().clone();
                        let content = state.content.read().clone();

                        match auto_saver.auto_save(path.as_ref(), &content).await {
                            Ok(_) => {
                                tracing::info!("自动保存成功 / Auto save successful");
                                // 使用 mark_saved 确保同步 file_watch_refresh_seq，防止误报外部修改
                                // Use mark_saved to sync file_watch_refresh_seq, preventing false external-modification alerts
                                state.mark_saved();
                            }
                            Err(e) => {
                                tracing::error!("自动保存失败 / Auto save failed: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    // 文件外部修改检测 / File External Modification Detection
    {
        let state_clone = state;
        use_future(move || {
            let mut state = state_clone;
            let mut checker = FileModificationChecker::new();
            let mut last_file: Option<std::path::PathBuf> = None;
            let mut last_refresh_seq = 0_u64;

            async move {
                loop {
                    // 动态调整检测频率：有文件时快速检查，无文件时低频轮询
                    // Dynamic check interval: fast checks with open file, low-frequency when idle
                    let has_file = state.current_file.read().is_some();
                    let check_interval = if has_file {
                        std::time::Duration::from_millis(FILE_WATCH_ACTIVE_INTERVAL_MS)
                    } else {
                        std::time::Duration::from_secs(FILE_WATCH_IDLE_INTERVAL_SECS)
                    };
                    tokio::time::sleep(check_interval).await;

                    let current_file = state.current_file.read().clone();
                    let refresh_seq = *state.file_watch_refresh_seq.read();

                    // 如果文件改变，更新检测器 / If file changed, update checker
                    if current_file != last_file {
                        if let Some(ref path) = current_file {
                            checker.set_file(path);
                            *state.file_external_modified.write() = false;
                        } else {
                            checker.clear();
                        }
                        last_file = current_file;
                        last_refresh_seq = refresh_seq;
                        continue;
                    }

                    // 内部保存、忽略提示或重新加载后刷新监控基线
                    // Refresh watcher baseline after internal save, ignore, or reload actions
                    if refresh_seq != last_refresh_seq {
                        checker.update();
                        *state.file_external_modified.write() = false;
                        last_refresh_seq = refresh_seq;
                        continue;
                    }

                    // 最近刚保存时跳过一次外部变更提示，避免自写入误报
                    // Skip notifications shortly after internal save to avoid false positives
                    let recently_saved = state.last_saved.read().as_ref().is_some_and(|saved_at| {
                        saved_at.elapsed()
                            < std::time::Duration::from_millis(FILE_WATCH_INTERNAL_WRITE_GRACE_MS)
                    });

                    if recently_saved {
                        checker.update();
                        continue;
                    }

                    // 检查文件是否被外部修改 / Check if file was externally modified
                    if checker.check_modified() {
                        let already_notified = *state.file_external_modified.read();
                        if !already_notified {
                            tracing::warn!("文件被外部修改/File was externally modified");
                            *state.file_external_modified.write() = true;
                            // FileModifiedModal 会显示提示 / FileModifiedModal will show the prompt
                        }
                    }
                }
            }
        });
    }

    rsx! {
        // 注入 CSS 样式 / Inject CSS Styles
        style { dangerous_inner_html: "{ALL_CSS}" }

        div {
            class: "app-container",
            "data-theme": "{theme_str}",

            // 工具栏 / Toolbar
            Toolbar {}

            // 主内容区 / Main Content Area
            div { class: "app-main",
                // 侧边栏 / Sidebar
                Sidebar {}

                // 编辑器区域 / Editor Area
                div { class: "editor-area",
                    // 标签栏 / Tab Bar
                    TabBar {}

                    // 编辑器和预览容器 / Editor and Preview Container
                    div { class: "editor-panes",
                        // 编辑器窗格 / Editor Pane（Editor 组件自带 editor-pane class）
                        Editor {}
                        // 预览窗格 / Preview Pane
                        Preview {}
                    }
                }
            }

            // 状态栏 / Status Bar
            StatusBar {}
        }

        // 弹窗层 - 条件渲染 / Modal Layer - Conditional Rendering
        SettingsModal {}
        ShortcutsModal {}
        AiChatModal {}
        AiResultModal {}
        GlobalSearchModal {}
        FileModifiedModal {}
        TableEditorModal {}
        SearchModal {}
        LargeFileWarningModal {}
        CloseConfirmModal {}
    }
}
