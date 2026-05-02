//! 设置弹窗组件 / Settings Modal Component

use crate::actions::{AppActions, EditorActions};
use crate::components::icons::CloseIcon;
use crate::config::{LARGE_FILE_THRESHOLD_BYTES, SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::services::ai::AIService;
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

    let display_class = if show { "" } else { "hidden" };

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                AppActions::hide_settings(&mut state);
            },

            div {
                class: "modal settings-modal",
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
                                        *state.auto_save_interval.write() = secs.clamp(10, 300);
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
                                    AppActions::set_ai_provider(&mut state, provider);
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
