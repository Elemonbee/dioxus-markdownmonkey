//! AI 结果弹窗组件 / AI Result Modal Component

use crate::actions::{AppActions, EditorActions};
use crate::components::icons::CloseIcon;
use crate::config::AI_COMPARE_MAX_CHARS;
use crate::services::ai::AITask;
use crate::state::AppState;
use crate::utils::i18n::t;
use dioxus::prelude::*;

/// AI 结果弹窗 / AI Result Modal
#[component]
pub fn AiResultModal() -> Element {
    let mut state = use_context::<AppState>();
    let ai = state.ai();
    let ui = state.ui();
    let show = *ai.show_ai_result.read();
    let lang = *ui.language.read();
    let ai_loading = *ai.ai_loading.read();
    let apply_ctx = ai.ai_apply_context.read().clone();
    let task = apply_ctx
        .as_ref()
        .map(|ctx| AITask::from_str_id(&ctx.task_id))
        .unwrap_or(AITask::Custom);
    let used_selection = apply_ctx
        .as_ref()
        .map(|ctx| ctx.used_selection && !ctx.source_text.is_empty())
        .unwrap_or(false);
    let can_retry = apply_ctx.as_ref().map(|ctx| ctx.is_error).unwrap_or(false);
    let source_text = apply_ctx
        .as_ref()
        .map(|ctx| ctx.source_text.clone())
        .unwrap_or_default();
    let apply_kinds = task.apply_kinds(used_selection);
    let show_compare = task.shows_compare(used_selection)
        && !ai_loading
        && !can_retry
        && source_text.chars().count() <= AI_COMPARE_MAX_CHARS;

    let close_t = t("close", lang);
    let copy_t = t("copy", lang);
    let retry_t = t("ai_retry", lang);
    let follow_up_t = t("ai_follow_up", lang);
    let stop_t = t("ai_stop", lang);
    let generating_t = t("ai_generating", lang);
    let thinking_t = t("ai_thinking", lang);
    let compare_original_t = t("ai_compare_original", lang);
    let compare_result_t = t("ai_compare_result", lang);

    let display_class = if show { "" } else { "hidden" };

    let title = ai.ai_title.read().clone();
    let result = ai.ai_result.read().clone();
    let apply_disabled = ai_loading || result.is_empty();

    rsx! {
        div {
            class: "modal-overlay {display_class}",
            onclick: move |_| {
                AppActions::hide_ai_result(&mut state);
            },

            div {
                class: "modal ai-result-modal",
                role: "dialog",
                "aria-modal": "true",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { "{title}" }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            AppActions::hide_ai_result(&mut state);
                        },
                        CloseIcon { size: 20 }
                    }
                }

                div { class: "modal-body",
                    if ai_loading && result.is_empty() {
                        div { class: "ai-result-loading", "aria-live": "polite", "{thinking_t}" }
                    }
                    if show_compare {
                        div { class: "ai-result-compare",
                            div { class: "ai-result-compare-pane",
                                div { class: "ai-result-compare-label", "{compare_original_t}" }
                                pre { "{source_text}" }
                            }
                            div { class: "ai-result-compare-pane",
                                div { class: "ai-result-compare-label", "{compare_result_t}" }
                                pre { "{result}" }
                            }
                        }
                    } else {
                        div {
                            class: "ai-result-content",
                            "aria-live": "polite",
                            pre { "{result}" }
                        }
                    }
                    if ai_loading {
                        div { class: "ai-result-generating", "aria-live": "polite", "{generating_t}" }
                    }
                }

                div { class: "modal-footer ai-result-footer",
                    if ai_loading {
                        button {
                            class: "btn-secondary",
                            onclick: move |_| {
                                AppActions::cancel_ai_generation(&mut state);
                            },
                            "{stop_t}"
                        }
                    }
                    if can_retry && !ai_loading {
                        RetryButton {
                            label: retry_t.clone(),
                            title_error: t("ai_error", lang),
                            error_prefix: t("error", lang),
                        }
                    }
                    CopyButton { result: result.clone(), copy_text: copy_t.clone() }
                    for (i, kind) in apply_kinds.iter().copied().enumerate() {
                        ApplyAiButton {
                            result: result.clone(),
                            label: t(kind.i18n_key(), lang),
                            disabled: apply_disabled,
                            primary: i == 0,
                            kind,
                        }
                    }
                    button {
                        class: "btn-secondary",
                        disabled: ai_loading,
                        onclick: move |_| {
                            AppActions::follow_up_ai_chat(&mut state);
                        },
                        "{follow_up_t}"
                    }
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            AppActions::hide_ai_result(&mut state);
                        },
                        "{close_t}"
                    }
                }
            }
        }
    }
}

/// 复制按钮组件属性 / Copy Button Props
#[derive(Props, Clone, PartialEq)]
struct CopyButtonProps {
    result: String,
    copy_text: String,
}

/// 复制按钮组件 / Copy Button Component
fn CopyButton(props: CopyButtonProps) -> Element {
    rsx! {
        button {
            class: "btn-secondary",
            onclick: move |_| {
                crate::utils::clipboard::copy_text(&props.result);
            },
            "{props.copy_text}"
        }
    }
}

/// 应用按钮属性 / Apply-button props
#[derive(Props, Clone, PartialEq)]
struct ApplyAiButtonProps {
    result: String,
    label: String,
    disabled: bool,
    primary: bool,
    kind: crate::services::ai::AiApplyKind,
}

/// 将 AI 结果写回编辑器 / Write the AI result back into the editor
fn ApplyAiButton(props: ApplyAiButtonProps) -> Element {
    let state = use_context::<AppState>();
    let result = props.result.clone();
    let kind = props.kind;
    let class = if props.primary {
        "btn-primary"
    } else {
        "btn-secondary"
    };

    rsx! {
        button {
            class: "{class}",
            disabled: props.disabled,
            onclick: move |_| {
                let result = result.clone();
                let mut state = state;
                spawn(async move {
                    EditorActions::apply_ai_kind(&mut state, kind, &result).await;
                    AppActions::hide_ai_result(&mut state);
                });
            },
            "{props.label}"
        }
    }
}

/// 重试按钮属性 / Retry button props
#[derive(Props, Clone, PartialEq)]
struct RetryButtonProps {
    label: String,
    title_error: String,
    error_prefix: String,
}

/// 重试最近一次失败的 AI 请求 / Retry the last failed AI request
fn RetryButton(props: RetryButtonProps) -> Element {
    let state = use_context::<AppState>();
    rsx! {
        button {
            class: "btn-primary",
            onclick: move |_| {
                let mut state = state;
                let title_error = props.title_error.clone();
                let error_prefix = props.error_prefix.clone();
                spawn(async move {
                    AppActions::retry_ai_task(&mut state, title_error, error_prefix).await;
                });
            },
            "{props.label}"
        }
    }
}
