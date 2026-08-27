//! 设置弹窗组件 / Settings Modal Component

use crate::actions::{AppActions, EditorActions, SettingsActions};
use crate::components::icons::{CloseIcon, RefreshIcon};
use crate::config::{LARGE_FILE_THRESHOLD_BYTES, SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::services::ai::fetch_available_models;
use crate::state::{AIProvider, AppState};
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// 设置弹窗 / Settings Modal
#[component]
pub fn SettingsModal() -> Element {
    let mut state = use_context::<AppState>();
    let ui = state.ui();
    let ai = state.ai();
    let desktop = dioxus::desktop::use_window();
    let settings_window = desktop.window.clone();
    let show = *ui.show_settings.read();
    let lang = *ui.language.read();

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
    let session_restore_t = t("session_restore", lang);
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
    let system_prompt_t = t("system_prompt", lang);
    let system_prompt_hint_t = t("system_prompt_hint", lang);
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
        let show_now = *ui.show_settings.read();
        if show_now {
            let provider = ai.ai_config.read().provider.clone();
            if matches!(provider, AIProvider::Ollama | AIProvider::OpenRouter)
                && available_models.read().is_empty()
                && !*models_loading.read()
            {
                let base_url = ai.ai_config.read().base_url.clone();
                let api_key = ai.ai_config.read().api_key.clone();
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
                                value: "{*ui.font_size.read()}",
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
                                value: "{*ui.preview_font_size.read()}",
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
                                checked: *ui.word_wrap.read(),
                                onchange: move |_| {
                                    EditorActions::toggle_word_wrap(&mut state);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{line_numbers_t}" }
                            input {
                                r#type: "checkbox",
                                checked: *ui.line_numbers.read(),
                                onchange: move |_| {
                                    EditorActions::toggle_line_numbers(&mut state);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{sync_scroll_t}" }
                            input {
                                r#type: "checkbox",
                                checked: *ui.sync_scroll.read(),
                                onchange: move |_| {
                                    EditorActions::toggle_sync_scroll(&mut state);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{auto_save_t}" }
                            input {
                                r#type: "checkbox",
                                checked: *ui.auto_save_enabled.read(),
                                onchange: move |_| {
                                    AppActions::toggle_auto_save(&mut state);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{session_restore_t}" }
                            input {
                                r#type: "checkbox",
                                checked: *ui.session_restore_enabled.read(),
                                onchange: move |_| {
                                    let next = !*ui.session_restore_enabled.read();
                                    SettingsActions::set_session_restore(&mut state, next);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{auto_save_interval_t}" }
                            input {
                                r#type: "number",
                                min: "10",
                                max: "300",
                                value: "{*ui.auto_save_interval.read()}",
                                oninput: move |e| {
                                    if let Ok(secs) = e.value().parse::<u32>() {
                                        AppActions::set_auto_save_interval(&mut state, secs);
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
                                    selected: *ui.theme.read() == crate::state::Theme::Dark,
                                    "{dark_t}"
                                }
                                option {
                                    value: "light",
                                    selected: *ui.theme.read() == crate::state::Theme::Light,
                                    "{light_t}"
                                }
                                option {
                                    value: "system",
                                    selected: *ui.theme.read() == crate::state::Theme::System,
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
                                    selected: *ui.language.read() == crate::state::Language::ZhCN,
                                    "中文"
                                }
                                option {
                                    value: "en-US",
                                    selected: *ui.language.read() == crate::state::Language::EnUS,
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
                                value: "{*ui.sidebar_width.read()}",
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
                                checked: ai.ai_config.read().enabled,
                                onchange: move |_| {
                                    SettingsActions::toggle_ai_enabled(&mut state);
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{provider_t}" }
                            select {
                                value: "{ai.ai_config.read().provider.as_str()}",
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
                                        let base_url = ai.ai_config.read().base_url.clone();
                                        let api_key = ai.ai_config.read().api_key.clone();
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
                                value: "{ai.ai_config.read().api_key}",
                                oninput: move |e| {
                                    SettingsActions::set_ai_api_key(&mut state, e.value());
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{api_base_url_label_t}" }
                            input {
                                r#type: "text",
                                placeholder: "{api_base_url_placeholder_t}",
                                value: "{ai.ai_config.read().base_url}",
                                oninput: move |e| {
                                    SettingsActions::set_ai_base_url(&mut state, e.value());
                                },
                            }
                        }

                        div { class: "settings-row",
                            label { "{model_name_t}" }

                            div { class: "settings-model-select",
                                {
                                    let provider = ai.ai_config.read().provider.clone();
                                    let current_model = ai.ai_config.read().model.clone();
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
                                                        SettingsActions::set_ai_model(&mut state, val);
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
                                                value: "{ai.ai_config.read().model}",
                                                oninput: move |e| {
                                                    SettingsActions::set_ai_model(&mut state, e.value());
                                                },
                                            }
                                        }
                                    }
                                }

                                // 刷新按钮（仅 Ollama/OpenRouter）/ Refresh button (Ollama/OpenRouter only)
                                {
                                    let provider = ai.ai_config.read().provider.clone();
                                    let is_dropdown_provider = matches!(provider, AIProvider::Ollama | AIProvider::OpenRouter);
                                    if is_dropdown_provider {
                                        rsx! {
                                            button {
                                                class: "model-refresh-btn",
                                                title: "{fetch_models_t}",
                                                disabled: *models_loading.read(),
                                                onclick: move |_| {
                                                    let provider = ai.ai_config.read().provider.clone();
                                                    let base_url = ai.ai_config.read().base_url.clone();
                                                    let api_key = ai.ai_config.read().api_key.clone();
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
                                value: "{ai.ai_config.read().temperature}",
                                oninput: move |e| {
                                    if let Ok(temp) = e.value().parse::<f32>() {
                                        SettingsActions::set_ai_temperature(&mut state, temp);
                                    }
                                },
                            }
                        }

                        div { class: "settings-row settings-row-textarea",
                            label { "{system_prompt_t}" }
                            textarea {
                                rows: "5",
                                placeholder: "{system_prompt_hint_t}",
                                value: "{ai.ai_config.read().system_prompt}",
                                oninput: move |e| {
                                    SettingsActions::set_ai_system_prompt(&mut state, e.value());
                                },
                            }
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            SettingsActions::reset_editor_defaults(&mut state);
                        },
                        "{reset_default_t}"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let size = settings_window.inner_size();
                            let scale = settings_window.scale_factor();
                            SettingsActions::persist_and_close(
                                &mut state,
                                (size.width as f64 / scale).max(600.0),
                                (size.height as f64 / scale).max(400.0),
                            );
                        },
                        "{save_close_t}"
                    }
                }
            }
        }
    }
}
