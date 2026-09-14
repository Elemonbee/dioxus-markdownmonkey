//! 设置弹窗 Actions / Settings modal actions

use crate::actions::{AppActions, EditorActions};
use crate::services::ai::AIService;
use crate::services::keyring_service;
use crate::services::settings::{save_settings, AppSettings};
use crate::state::AppState;
use dioxus::prelude::{ReadableExt, WritableExt};

/// 设置 Actions / Settings actions
pub struct SettingsActions;

impl SettingsActions {
    /// 设置是否启动时恢复会话 / Set session restore on launch
    pub fn set_session_restore(state: &mut AppState, enabled: bool) {
        *state.ui().session_restore_enabled.write() = enabled;
    }

    /// 切换 AI 开关，空 URL/模型时填默认值
    /// Toggle AI; fill default URL/model when empty
    pub fn toggle_ai_enabled(state: &mut AppState) {
        let mut ai = state.ai();
        let mut config = ai.ai_config.write();
        config.enabled = !config.enabled;
        if config.base_url.is_empty() {
            config.base_url = AIService::default_base_url(&config.provider).to_string();
        }
        if config.model.is_empty() {
            config.model = AIService::default_model(&config.provider).to_string();
        }
    }

    /// 设置 API Key（仅内存；保存时再写入密钥环）
    /// Set API key in memory (persisted to keyring on save)
    pub fn set_ai_api_key(state: &mut AppState, api_key: String) {
        state.ai().ai_config.write().api_key = api_key;
    }

    /// 设置 API Base URL / Set API base URL
    pub fn set_ai_base_url(state: &mut AppState, base_url: String) {
        state.ai().ai_config.write().base_url = base_url;
    }

    /// 设置模型名 / Set model name
    pub fn set_ai_model(state: &mut AppState, model: String) {
        state.ai().ai_config.write().model = model;
    }

    /// 设置温度 / Set temperature
    pub fn set_ai_temperature(state: &mut AppState, temperature: f32) {
        state.ai().ai_config.write().temperature = temperature.clamp(0.0, 1.0);
    }

    /// 设置系统提示词 / Set system prompt
    pub fn set_ai_system_prompt(state: &mut AppState, prompt: String) {
        state.ai().ai_config.write().system_prompt = prompt;
    }

    /// 重置编辑器相关设置为默认值 / Reset editor-related settings to defaults
    pub fn reset_editor_defaults(state: &mut AppState) {
        EditorActions::set_font_size(state, 16);
        EditorActions::set_preview_font_size(state, 16);
        EditorActions::set_word_wrap(state, true);
        EditorActions::set_line_numbers(state, true);
        EditorActions::set_sync_scroll(state, true);
        Self::set_session_restore(state, true);
        AppActions::set_sidebar_width(state, 280);
    }

    /// 把 API Key 写入密钥环并持久化设置，然后关闭弹窗
    /// Store the API key in the keyring, persist settings, then close the modal
    pub fn persist_and_close(state: &mut AppState, window_width: f64, window_height: f64) {
        let (provider_name, api_key) = {
            let ai = state.ai();
            let config = ai.ai_config.read();
            (config.provider.as_str().to_string(), config.api_key.clone())
        };

        if api_key.is_empty() {
            if let Err(e) = keyring_service::delete_api_key(&provider_name) {
                tracing::debug!("No keyring entry to delete for {}: {}", provider_name, e);
            }
        } else if let Err(e) = keyring_service::store_api_key(&provider_name, &api_key) {
            tracing::warn!("Cannot store API key in keyring: {}", e);
        }

        let settings = AppSettings::from_state(state, window_width, window_height);
        if let Err(e) = save_settings(&settings) {
            tracing::error!("Failed to save settings: {}", e);
        }
        AppActions::hide_settings(state);
    }
}
