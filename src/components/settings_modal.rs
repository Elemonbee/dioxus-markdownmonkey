//! 设置弹窗组件 / Settings Modal Component

use crate::actions::{AppActions, EditorActions};
use crate::components::icons::{CloseIcon, RefreshIcon};
use crate::config::{
    AUTO_SAVE_INTERVAL_MAX_SECS, AUTO_SAVE_INTERVAL_MIN_SECS, LARGE_FILE_THRESHOLD_BYTES,
    SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH,
};
use crate::services::ai::{fetch_available_models, AIService};
use crate::services::keyring_service;
use crate::services::settings::{save_settings, AISettings, AppSettings};
use crate::state::{AIProvider, AppState};
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 设置弹窗 / Settings Modal
#[component]
pub fn SettingsModal() -> Element {
    let mut state = use_context::<AppState>();
    let show = *state.show_settings.read();
    let lang = *state.language.read();

    // i18n
    let settings_t = t("settings", lang);
    let editor_t = t("editor", lang);
    let font_size_t = t("font_size", lang);
    let preview_font_size_t = t("preview_font_size", lang);
    let word_wrap_t = t("word_wrap", lang);
    let line_numbers_t = t("line_numbers", lang);
    let sync_scroll_t = t("sync_scroll", lang);
    let auto_save_t = t("auto_save", lang);
    let auto_save_interval_t = t("auto_save_interval", lang);
    let appearance_t = t("appearance", lang);
    let theme_t = t("theme", lang);
    let dark_t = t("dark", lang);
    let light_t = t("light", lang);
    let follow_system_t = t("follow_system", lang);
    let language_t = t("language", lang);
    let sidebar_width_t = t("sidebar_width", lang);
    let ai_title_t = t("ai_assistant", lang);
    let enable_ai_t = t("enable_ai", lang);
    let enter_api_key_t = t("enter_api_key", lang);
    let model_name_t = t("model_name", lang);
    let temperature_t = t("temperature", lang);
    let api_key_label_t = t("api_key_label", lang);
    let api_base_url_label_t = t("api_base_url_label", lang);
    let api_base_url_placeholder_t = t("api_base_url_placeholder", lang);
    let provider_t = t("provider", lang);
    let provider_openai_t = t("provider_openai", lang);
    let provider_claude_t = t("provider_claude", lang);
    let provider_ollama_t = t("provider_ollama", lang);
    let provider_deepseek_t = t("provider_deepseek", lang);
    let provider_kimi_t = t("provider_kimi", lang);
    let provider_openrouter_t = t("provider_openrouter", lang);
    let large_file_threshold_t = t("large_file_threshold", lang);
    let reset_default_t = t("reset_default", lang);
    let save_close_t = t("save_close", lang);
    let spell_check_t = t("spell_check", lang);

    // 模型下拉列表 i18n / Model dropdown i18n
    let loading_models_t = t("loading_models", lang);
    let fetch_models_t = t("fetch_models", lang);
    let custom_model_t = t("custom_model", lang);
    let no_models_found_t = t("no_models_found", lang);
    let select_model_t = t("select_model", lang);

    // 模型列表局部状态（运行时，不持久化）/ Model list local state (runtime-only)
    let mut available_models: Signal<Vec<String>> = use_signal(Vec::new);
    let mut models_loading: Signal<bool> = use_signal(|| false);
    let mut models_error: Signal<Option<String>> = use_signal(|| None);
    let mut use_custom_model: Signal<bool> = use_signal(|| false);

    let display_class = if show { "" } else { "hidden" };

    // 预克隆 i18n 字符串（避免跨闭包 move）/ Pre-clone i18n strings (avoid cross-closure move)
    let no_models_for_effect = no_models_found_t.clone();
    let no_models_for_provider = no_models_found_t.clone();
    let no_models_for_refresh = no_models_found_t.clone();

    // 弹窗打开时自动获取 / Auto-fetch when modal opens
    let _ = use_effect(move || {
        let show_now = *state.show_settings.read();
        if show_now {
            let provider = state.ai_config.read().provider.clone();
            if matches!(provider, AIProvider::Ollama | AIProvider::OpenRouter)
                && available_models.read().is_empty()
                && !*models_loading.read()
            {
                let base_url = state.ai_config.read().base_url.clone();
                let api_key = state.ai_config.read().api_key.clone();
                *models_loading.write() = true;
                *models_error.write() = None;
                let no_models_t = no_models_for_effect.clone();
                spawn(async move {
                    match fetch_available_models(&provider, &base_url, &api_key).await {
                        Ok(models) => {
                            if models.is_empty() {
                                *models_error.write() = Some(no_models_t);
                                *use_custom_model.write() = true;
                            } else {
                                *use_custom_model.write() = false;
                            }
                            *available_models.write() = models;
                        }
                        Err(e) => {
                            *models_error.write() = Some(format!("{}", e));
                            *available_models.write() = Vec::new();
                            *use_custom_model.write() = true;
                        }
                    }
                    *models_loading.write() = false;
                });
            }
        }
    });

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                AppActions::hide_settings(&mut state);
            },

            div {
                class: "modal settings-modal",
                role: "dialog",
                "aria-modal": "true",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { "{settings_t}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            AppActions::hide_settings(&mut state);
                        },
                        CloseIcon { size: 20 }
                    }
                }

                div { class: "modal-body",
                    // 编辑器设置 / Editor Settings
                    section { class: "settings-section",
                        h3 { "{editor_t}" }

                        div { class: "settings-row",
                            label { "{font_size_t}" }
                            input {
                                r#type: "number",
                                min: "10",
                                max: "32",
                                value: "{*state.font_size.read()}",
                                oninput: move |e| {
                                    if let Ok(size) = e.value().parse::<u32>() {
                                        EditorActions::set_font_size(&mut state, size);
                                    }
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{preview_font_size_t}" }
                            input {
                                r#type: "number",
                                min: "10",
                                max: "32",
                                value: "{*state.preview_font_size.read()}",
                                oninput: move |e| {
                                    if let Ok(size) = e.value().parse::<u32>() {
                                        EditorActions::set_preview_font_size(&mut state, size);
                                    }
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{word_wrap_t}" }
                            input {
                                r#type: "checkbox",
                                checked: *state.word_wrap.read(),
                                onchange: move |_| {
                                    EditorActions::toggle_word_wrap(&mut state);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{line_numbers_t}" }
                            input {
                                r#type: "checkbox",
                                checked: *state.line_numbers.read(),
                                onchange: move |_| {
                                    EditorActions::toggle_line_numbers(&mut state);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{sync_scroll_t}" }
                            input {
                                r#type: "checkbox",
                                checked: *state.sync_scroll.read(),
                                onchange: move |_| {
                                    EditorActions::toggle_sync_scroll(&mut state);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{spell_check_t}" }
                            input {
                                r#type: "checkbox",
                                checked: *state.spell_check_enabled.read(),
                                onchange: move |_| {
                                    EditorActions::toggle_spell_check(&mut state);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{auto_save_t}" }
                            input {
                                r#type: "checkbox",
                                checked: *state.auto_save_enabled.read(),
                                onchange: move |_| {
                                    let current = *state.auto_save_enabled.read();
                                    *state.auto_save_enabled.write() = !current;
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{auto_save_interval_t}" }
                            input {
                                r#type: "number",
                                min: "10",
                                max: "300",
                                value: "{*state.auto_save_interval.read()}",
                                oninput: move |e| {
                                    if let Ok(secs) = e.value().parse::<u32>() {
                                        *state.auto_save_interval.write() = secs.clamp(AUTO_SAVE_INTERVAL_MIN_SECS, AUTO_SAVE_INTERVAL_MAX_SECS);
                                    }
                                },
                            }
                        }
                    }

                    // 外观设置 / Appearance Settings
                    section { class: "settings-section",
                        h3 { "{appearance_t}" }

                        div { class: "settings-row",
                            label { "{theme_t}" }
                            select {
                                onchange: move |e| {
                                    let theme = match e.value().as_str() {
                                        "light" => crate::state::Theme::Light,
                                        "system" => crate::state::Theme::System,
                                        _ => crate::state::Theme::Dark,
                                    };
                                    AppActions::set_theme(&mut state, theme);
                                },
                                option {
                                    value: "dark",
                                    selected: *state.theme.read() == crate::state::Theme::Dark,
                                    "{dark_t}"
                                }
                                option {
                                    value: "light",
                                    selected: *state.theme.read() == crate::state::Theme::Light,
                                    "{light_t}"
                                }
                                option {
                                    value: "system",
                                    selected: *state.theme.read() == crate::state::Theme::System,
                                    "{follow_system_t}"
                                }
                            }
                        }

                        div { class: "settings-row",
                            label { "{language_t}" }
                            select {
                                onchange: move |e| {
                                    let language = match e.value().as_str() {
                                        "en-US" => crate::state::Language::EnUS,
                                        _ => crate::state::Language::ZhCN,
                                    };
                                    AppActions::set_language(&mut state, language);
                                },
                                option {
                                    value: "zh-CN",
                                    selected: *state.language.read() == crate::state::Language::ZhCN,
                                    "中文"
                                }
                                option {
                                    value: "en-US",
                                    selected: *state.language.read() == crate::state::Language::EnUS,
                                    "English"
                                }
                            }
                        }

                        div { class: "settings-row",
                            label { "{sidebar_width_t}" }
                            input {
                                r#type: "range",
                                min: "200",
                                max: "400",
                                value: "{*state.sidebar_width.read()}",
                                oninput: move |e| {
                                    if let Ok(width) = e.value().parse::<u32>() {
                                        AppActions::set_sidebar_width(&mut state, width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH));
                                    }
                                },
                            }
                        }

                        div { class: "settings-row settings-row-static",
                            label { "{large_file_threshold_t}" }
                            span { "{LARGE_FILE_THRESHOLD_BYTES / 1024 / 1024} MB" }
                        }
                    }

                    // AI 设置 / AI Settings
                    section { class: "settings-section",
                        h3 { "{ai_title_t}" }

                        div { class: "settings-row",
                            label { "{enable_ai_t}" }
                            input {
                                r#type: "checkbox",
                                checked: state.ai_config.read().enabled,
                                onchange: move |_| {
                                    let current = state.ai_config.read().enabled;
                                    let mut config = state.ai_config.write();
                                    config.enabled = !current;
                                    if config.base_url.is_empty() {
                                        config.base_url = AIService::default_base_url(&config.provider).to_string();
                                    }
                                    if config.model.is_empty() {
                                        config.model = AIService::default_model(&config.provider).to_string();
                                    }
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{provider_t}" }
                            select {
                                value: "{state.ai_config.read().provider.as_str()}",
                                onchange: move |e| {
                                    let provider = match e.value().as_str() {
                                        "claude" => AIProvider::Claude,
                                        "ollama" => AIProvider::Ollama,
                                        "deepseek" => AIProvider::DeepSeek,
                                        "kimi" => AIProvider::Kimi,
                                        "openrouter" => AIProvider::OpenRouter,
                                        _ => AIProvider::OpenAI,
                                    };
                                    AppActions::set_ai_provider(&mut state, provider.clone());
                                    // 重置模型列表状态 / Reset model list state
                                    available_models.set(Vec::new());
                                    models_error.set(None);
                                    use_custom_model.set(false);
                                    if matches!(provider, AIProvider::Ollama | AIProvider::OpenRouter) {
                                        let base_url = state.ai_config.read().base_url.clone();
                                        let api_key = state.ai_config.read().api_key.clone();
                                        *models_loading.write() = true;
                                        *models_error.write() = None;
                                        let no_models_t = no_models_for_provider.clone();
                                        spawn(async move {
                                            match fetch_available_models(&provider, &base_url, &api_key).await {
                                                Ok(models) => {
                                                    if models.is_empty() {
                                                        *models_error.write() = Some(no_models_t);
                                                        *use_custom_model.write() = true;
                                                    } else {
                                                        *use_custom_model.write() = false;
                                                    }
                                                    *available_models.write() = models;
                                                }
                                                Err(e) => {
                                                    *models_error.write() = Some(format!("{}", e));
                                                    *available_models.write() = Vec::new();
                                                    *use_custom_model.write() = true;
                                                }
                                            }
                                            *models_loading.write() = false;
                                        });
                                    }
                                },
                                option { value: "openai", "{provider_openai_t}" }
                                option { value: "claude", "{provider_claude_t}" }
                                option { value: "ollama", "{provider_ollama_t}" }
                                option { value: "deepseek", "{provider_deepseek_t}" }
                                option { value: "kimi", "{provider_kimi_t}" }
                                option { value: "openrouter", "{provider_openrouter_t}" }
                            }
                        }

                        div { class: "settings-row",
                            label { "{api_key_label_t}" }
                            input {
                                r#type: "password",
                                placeholder: "{enter_api_key_t}",
                                value: "{state.ai_config.read().api_key}",
                                oninput: move |e| {
                                    let mut config = state.ai_config.write();
                                    config.api_key = e.value();
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{api_base_url_label_t}" }
                            input {
                                r#type: "text",
                                placeholder: "{api_base_url_placeholder_t}",
                                value: "{state.ai_config.read().base_url}",
                                oninput: move |e| {
                                    let mut config = state.ai_config.write();
                                    config.base_url = e.value();
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{model_name_t}" }

                            div { class: "settings-model-select",
                                {
                                    let provider = state.ai_config.read().provider.clone();
                                    let current_model = state.ai_config.read().model.clone();
                                    let is_dropdown_provider = matches!(provider, AIProvider::Ollama | AIProvider::OpenRouter);
                                    let models = available_models.read().clone();
                                    let loading = *models_loading.read();
                                    let error = models_error.read().clone();
                                    let custom = *use_custom_model.read();

                                    if is_dropdown_provider && !loading && error.is_none() && !custom && !models.is_empty() {
                                        // 下拉列表模式 / Dropdown mode
                                        rsx! {
                                            select {
                                                value: "{current_model}",
                                                onchange: move |e| {
                                                    let val = e.value();
                                                    if val == "__custom__" {
                                                        use_custom_model.set(true);
                                                    } else {
                                                        let mut config = state.ai_config.write();
                                                        config.model = val;
                                                    }
                                                },
                                                option { value: "", disabled: true, "{select_model_t}" }
                                                for model_name in models.iter() {
                                                    {
                                                        let selected = *model_name == current_model;
                                                        rsx! {
                                                            option {
                                                                value: "{model_name}",
                                                                selected: "{selected}",
                                                                "{model_name}"
                                                            }
                                                        }
                                                    }
                                                }
                                                option { value: "__custom__", "{custom_model_t}..." }
                                            }
                                        }
                                    } else if is_dropdown_provider && loading {
                                        // 加载中 / Loading
                                        rsx! {
                                            span { class: "model-loading-hint", "{loading_models_t}" }
                                        }
                                    } else {
                                        // 文本输入模式（默认或其他提供商）/ Text input mode
                                        rsx! {
                                            input {
                                                r#type: "text",
                                                placeholder: "{model_name_t}",
                                                value: "{state.ai_config.read().model}",
                                                oninput: move |e| {
                                                    let mut config = state.ai_config.write();
                                                    config.model = e.value();
                                                },
                                            }
                                        }
                                    }
                                }

                                // 刷新按钮（仅 Ollama/OpenRouter）/ Refresh button (Ollama/OpenRouter only)
                                {
                                    let provider = state.ai_config.read().provider.clone();
                                    let is_dropdown_provider = matches!(provider, AIProvider::Ollama | AIProvider::OpenRouter);
                                    if is_dropdown_provider {
                                        rsx! {
                                            button {
                                                class: "model-refresh-btn",
                                                title: "{fetch_models_t}",
                                                disabled: *models_loading.read(),
                                                onclick: move |_| {
                                                    let provider = state.ai_config.read().provider.clone();
                                                    let base_url = state.ai_config.read().base_url.clone();
                                                    let api_key = state.ai_config.read().api_key.clone();
                                                    *models_loading.write() = true;
                                                    *models_error.write() = None;
                                                    let no_models_t = no_models_for_refresh.clone();
                                                    spawn(async move {
                                                        match fetch_available_models(&provider, &base_url, &api_key).await {
                                                            Ok(models) => {
                                                                if models.is_empty() {
                                                                    *models_error.write() = Some(no_models_t);
                                                                    *use_custom_model.write() = true;
                                                                } else {
                                                                    *use_custom_model.write() = false;
                                                                }
                                                                *available_models.write() = models;
                                                            }
                                                            Err(e) => {
                                                                *models_error.write() = Some(format!("{}", e));
                                                                *available_models.write() = Vec::new();
                                                                *use_custom_model.write() = true;
                                                            }
                                                        }
                                                        *models_loading.write() = false;
                                                    });
                                                },
                                                RefreshIcon { size: 16 }
                                            }
                                        }
                                    } else {
                                        rsx! {}
                                    }
                                }
                            }

                            // 错误提示 / Error hint
                            {
                                let error = models_error.read().clone();
                                if let Some(err) = error {
                                    rsx! {
                                        div { class: "model-error-hint", "{err}" }
                                    }
                                } else {
                                    rsx! {}
                                }
                            }
                        }

                        div { class: "settings-row",
                            label { "{temperature_t}" }
                            input {
                                r#type: "number",
                                min: "0",
                                max: "1",
                                step: "0.1",
                                value: "{state.ai_config.read().temperature}",
                                oninput: move |e| {
                                    if let Ok(temp) = e.value().parse::<f32>() {
                                        let mut config = state.ai_config.write();
                                        config.temperature = temp.clamp(0.0, 1.0);
                                    }
                                },
                            }
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            EditorActions::set_font_size(&mut state, 16);
                            EditorActions::set_preview_font_size(&mut state, 16);
                            EditorActions::set_word_wrap(&mut state, false);
                            EditorActions::set_line_numbers(&mut state, true);
                            EditorActions::set_sync_scroll(&mut state, true);
                            *state.spell_check_enabled.write() = false;
                            *state.spell_check_results.write() = Vec::new();
                            AppActions::set_sidebar_width(&mut state, 280);
                        },
                        "{reset_default_t}"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let (ai_enabled, ai_provider, ai_model, api_key, base_url, system_prompt, temperature) = {
                                let config = state.ai_config.read();
                                (
                                    config.enabled,
                                    config.provider.as_str().to_string(),
                                    config.model.clone(),
                                    config.api_key.clone(),
                                    config.base_url.clone(),
                                    config.system_prompt.clone(),
                                    config.temperature,
                                )
                            };

                            if !api_key.is_empty() {
                                let provider_name = ai_provider.as_str();
                                if let Err(e) = keyring_service::store_api_key(provider_name, &api_key) {
                                    tracing::warn!("Cannot use keyring, falling back to file: {}", e);
                                }
                            }

                            let settings = AppSettings {
                                theme: match *state.theme.read() {
                                    crate::state::Theme::Dark => "dark".to_string(),
                                    crate::state::Theme::Light => "light".to_string(),
                                    crate::state::Theme::System => "system".to_string(),
                                },
                                language: match *state.language.read() {
                                    crate::state::Language::ZhCN => "zh-CN".to_string(),
                                    crate::state::Language::EnUS => "en-US".to_string(),
                                },
                                font_size: *state.font_size.read(),
                                preview_font_size: *state.preview_font_size.read(),
                                word_wrap: *state.word_wrap.read(),
                                line_numbers: *state.line_numbers.read(),
                                sync_scroll: *state.sync_scroll.read(),
                                sidebar_visible: *state.sidebar_visible.read(),
                                show_preview: *state.show_preview.read(),
                                sidebar_width: *state.sidebar_width.read(),
                                auto_save_enabled: *state.auto_save_enabled.read(),
                                auto_save_interval: *state.auto_save_interval.read(),
                                spell_check_enabled: *state.spell_check_enabled.read(),
                                // 窗口尺寸持久化 / Window size persistence
                                window_width: {
                                    let desktop = dioxus::desktop::use_window();
                                    let size = desktop.window.inner_size();
                                    let scale = desktop.window.scale_factor();
                                    (size.width as f64 / scale).max(600.0)
                                },
                                window_height: {
                                    let desktop = dioxus::desktop::use_window();
                                    let size = desktop.window.inner_size();
                                    let scale = desktop.window.scale_factor();
                                    (size.height as f64 / scale).max(400.0)
                                },
                                ai: AISettings {
                                    enabled: ai_enabled,
                                    provider: ai_provider.clone(),
                                    model: ai_model,
                                    api_key: None,
                                    base_url,
                                    system_prompt,
                                    temperature,
                                },
                            };

                            if let Err(e) = save_settings(&settings) {
                                tracing::error!("Failed to save settings: {}", e);
                            }

                            AppActions::hide_settings(&mut state);
                        },
                        "{save_close_t}"
                    }
                }
            }
        }
    }
}
