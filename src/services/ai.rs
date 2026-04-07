//! AI 服务 - 处理与 AI API 的交互 / AI Service - Handle AI API Interactions

use crate::state::AIProvider;
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 默认请求超时（秒）/ Default request timeout (seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// AI 错误类型 / AI Error Types
#[derive(Error, Debug)]
pub enum AIError {
    #[error("网络错误/Network Error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("API 错误/API Error: {0}")]
    Api(String),

    #[error("认证失败/Authentication Error: {0}")]
    Authentication(String),

    #[error("请求被限流/Rate Limit: {0}")]
    RateLimit(String),

    #[error("服务暂时不可用/Service Unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("配置错误/Config Error: {0}")]
    Config(String),

    #[error("请求超时/Request Timeout: {0}")]
    Timeout(String),

    #[error("解析错误/Parse Error: {0}")]
    Parse(String),
}

/// AI 请求 / AI Request
#[derive(Serialize)]
struct AIRequest {
    model: String,          // 模型名称 / Model Name
    messages: Vec<Message>, // 消息列表 / Message List
    stream: bool,           // 是否流式 / Is Streaming
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>, // 温度参数 / Temperature Parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>, // 最大令牌数 / Max Tokens
}

/// 消息 / Message
#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    role: String,    // 角色 (system/user/assistant) / Role
    content: String, // 内容 / Content
}

/// AI 响应 / AI Response
#[derive(Deserialize)]
struct AIResponse {
    choices: Vec<Choice>, // 选择项列表 / Choice List
}

/// 选择项 / Choice
#[derive(Deserialize)]
struct Choice {
    #[allow(dead_code)]
    message: Option<Message>, // 消息 / Message
    delta: Option<Delta>, // 增量 / Delta
    #[serde(rename = "finish_reason")]
    _finish_reason: Option<String>, // 完成原因 / Finish Reason
}

/// 流式响应增量 / Streaming Response Delta
#[derive(Deserialize)]
struct Delta {
    content: Option<String>, // 内容 / Content
}

/// AI 服务 / AI Service
pub struct AIService {
    client: Client,   // HTTP 客户端 / HTTP Client
    base_url: String, // API 基础 URL / API Base URL
    api_key: String,  // API 密钥 / API Key
    model: String,    // 模型名称 / Model Name
}

impl AIService {
    /// 创建新的 AI 服务（使用默认 120 秒超时）
    /// Create New AI Service (with default 120-second timeout)
    pub fn new(api_key: String, base_url: Option<String>, model: Option<String>) -> Self {
        Self::with_timeout(api_key, base_url, model, DEFAULT_TIMEOUT_SECS)
    }

    /// 创建新的 AI 服务（可配置超时）
    /// Create New AI Service (configurable timeout)
    pub fn with_timeout(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
        timeout_secs: u64,
    ) -> Self {
        let normalized_base_url = base_url
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
            .trim_end_matches('/')
            .to_string();

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            base_url: normalized_base_url,
            api_key,
            model: model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
        }
    }

    /// 获取默认 base URL / Get Default Base URL
    pub fn default_base_url(provider: &AIProvider) -> &'static str {
        match provider {
            AIProvider::OpenAI => "https://api.openai.com/v1",
            AIProvider::Claude => "https://api.anthropic.com/v1",
            AIProvider::DeepSeek => "https://api.deepseek.com/v1",
            AIProvider::Kimi => "https://api.moonshot.cn/v1",
            AIProvider::Ollama => "http://localhost:11434/v1",
            AIProvider::OpenRouter => "https://openrouter.ai/api/v1",
        }
    }

    /// 获取默认模型 / Get Default Model
    pub fn default_model(provider: &AIProvider) -> &'static str {
        match provider {
            AIProvider::OpenAI => "gpt-4o-mini",
            AIProvider::Claude => "claude-3-haiku-20240307",
            AIProvider::DeepSeek => "deepseek-chat",
            AIProvider::Kimi => "moonshot-v1-8k",
            AIProvider::Ollama => "llama3",
            AIProvider::OpenRouter => "openai/gpt-4o-mini",
        }
    }

    /// 发送聊天请求 / Send Chat Request
    /// 自动检测是否为 Claude 提供商并使用对应的 API 格式
    /// Auto-detect Claude provider and use appropriate API format
    #[allow(dead_code)]
    pub async fn chat(&self, messages: Vec<Message>) -> Result<String, AIError> {
        self.validate_config()?;

        // 检测是否为 Anthropic Claude API（通过 base_url 判断）
        // Detect Anthropic Claude API (via base_url)
        let is_claude = self.base_url.contains("anthropic.com");

        if is_claude {
            self.chat_claude(messages).await
        } else {
            self.chat_openai_compatible(messages).await
        }
    }

    /// OpenAI 兼容格式的聊天请求 / OpenAI-compatible chat request
    #[allow(dead_code)]
    async fn chat_openai_compatible(&self, messages: Vec<Message>) -> Result<String, AIError> {
        let request = AIRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            temperature: Some(0.7),
            max_tokens: Some(2048),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(Self::map_reqwest_error)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Self::map_http_error(status, error_text));
        }

        let ai_response: AIResponse = response.json().await?;

        ai_response
            .choices
            .first()
            .and_then(|c| c.message.as_ref())
            .map(|m| m.content.clone())
            .ok_or_else(|| AIError::Parse("No response content".to_string()))
    }

    /// Anthropic Claude API 格式的聊天请求 / Anthropic Claude API format chat request
    /// Claude 使用 x-api-key 头和不同的请求体格式
    /// Claude uses x-api-key header and different request body format
    #[allow(dead_code)]
    async fn chat_claude(&self, messages: Vec<Message>) -> Result<String, AIError> {
        use serde_json::json;

        // 分离 system 消息和普通消息 / Separate system messages from regular messages
        let mut system_content = String::new();
        let mut claude_messages = Vec::new();

        for msg in messages {
            if msg.role == "system" {
                system_content = msg.content;
            } else {
                claude_messages.push(json!({
                    "role": msg.role,
                    "content": msg.content
                }));
            }
        }

        let mut body = json!({
            "model": self.model,
            "messages": claude_messages,
            "max_tokens": 2048,
        });

        if !system_content.is_empty() {
            body["system"] = json!(system_content);
        }

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(Self::map_reqwest_error)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Self::map_http_error(status, error_text));
        }

        // Claude 响应格式: {"content": [{"type": "text", "text": "..."}]}
        // Claude response format
        let response_json: serde_json::Value = response.json().await?;

        response_json
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AIError::Parse("No response content from Claude".to_string()))
    }

    /// 发送流式聊天请求 / Send Streaming Chat Request
    ///
    /// 通过 `on_chunk` 回调逐步返回内容，适合实时显示 AI 响应
    /// Returns content incrementally via `on_chunk` callback for real-time display
    pub async fn chat_stream<F>(
        &self,
        messages: Vec<Message>,
        mut on_chunk: F,
    ) -> Result<String, AIError>
    where
        F: FnMut(&str),
    {
        self.validate_config()?;

        // 检测是否为 Anthropic Claude API
        // Detect if Anthropic Claude API
        let is_claude = self.base_url.contains("anthropic.com");

        if is_claude {
            return self.chat_stream_claude(messages, on_chunk).await;
        }

        let request = AIRequest {
            model: self.model.clone(),
            messages,
            stream: true,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(Self::map_reqwest_error)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Self::map_http_error(status, error_text));
        }

        Self::collect_sse_content(
            response.bytes_stream(),
            &mut on_chunk,
            Self::parse_openai_sse_data,
            "No response content received from stream",
        )
        .await
    }

    /// Claude SSE 流式响应 / Claude SSE streaming response
    async fn chat_stream_claude<F>(
        &self,
        messages: Vec<Message>,
        mut on_chunk: F,
    ) -> Result<String, AIError>
    where
        F: FnMut(&str),
    {
        use serde_json::json;

        let mut system_content = String::new();
        let mut claude_messages = Vec::new();

        for msg in messages {
            if msg.role == "system" {
                system_content = msg.content;
            } else {
                claude_messages.push(json!({
                    "role": msg.role,
                    "content": msg.content
                }));
            }
        }

        let mut body = json!({
            "model": self.model,
            "messages": claude_messages,
            "max_tokens": 4096,
            "stream": true,
        });

        if !system_content.is_empty() {
            body["system"] = json!(system_content);
        }

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(Self::map_reqwest_error)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Self::map_http_error(status, error_text));
        }

        Self::collect_sse_content(
            response.bytes_stream(),
            &mut on_chunk,
            Self::parse_claude_sse_data,
            "No response content received from Claude stream",
        )
        .await
    }

    async fn collect_sse_content<S, B, F, P>(
        mut stream: S,
        on_chunk: &mut F,
        mut parse_data: P,
        empty_message: &str,
    ) -> Result<String, AIError>
    where
        S: Stream<Item = Result<B, reqwest::Error>> + Unpin,
        B: AsRef<[u8]>,
        F: FnMut(&str),
        P: FnMut(&str) -> Option<String>,
    {
        let mut full_content = String::new();
        let mut line_buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(Self::map_reqwest_error)?;
            line_buffer.push_str(&String::from_utf8_lossy(chunk.as_ref()));
            Self::drain_sse_lines(
                &mut line_buffer,
                &mut full_content,
                on_chunk,
                &mut parse_data,
            );
        }

        Self::process_sse_line(
            line_buffer.trim(),
            &mut full_content,
            on_chunk,
            &mut parse_data,
        );

        if full_content.is_empty() {
            return Err(AIError::Parse(empty_message.to_string()));
        }

        Ok(full_content)
    }

    fn drain_sse_lines<F, P>(
        line_buffer: &mut String,
        full_content: &mut String,
        on_chunk: &mut F,
        parse_data: &mut P,
    ) where
        F: FnMut(&str),
        P: FnMut(&str) -> Option<String>,
    {
        while let Some(newline_pos) = line_buffer.find('\n') {
            let line = line_buffer[..newline_pos].trim().to_string();
            *line_buffer = line_buffer[newline_pos + 1..].to_string();
            Self::process_sse_line(&line, full_content, on_chunk, parse_data);
        }
    }

    fn process_sse_line<F, P>(
        line: &str,
        full_content: &mut String,
        on_chunk: &mut F,
        parse_data: &mut P,
    ) where
        F: FnMut(&str),
        P: FnMut(&str) -> Option<String>,
    {
        if line.is_empty() || line.starts_with(':') {
            return;
        }

        let Some(data) = line.strip_prefix("data: ") else {
            return;
        };

        if data == "[DONE]" {
            return;
        }

        if let Some(content) = parse_data(data) {
            on_chunk(&content);
            full_content.push_str(&content);
        }
    }

    fn parse_openai_sse_data(data: &str) -> Option<String> {
        let parsed = serde_json::from_str::<AIResponse>(data).ok()?;
        parsed.choices.first()?.delta.as_ref()?.content.clone()
    }

    fn parse_claude_sse_data(data: &str) -> Option<String> {
        let parsed = serde_json::from_str::<serde_json::Value>(data).ok()?;
        if parsed.get("type").and_then(|t| t.as_str()) != Some("content_block_delta") {
            return None;
        }

        parsed
            .get("delta")
            .and_then(|delta| delta.get("text"))
            .and_then(|text| text.as_str())
            .map(|text| text.to_string())
    }

    fn validate_config(&self) -> Result<(), AIError> {
        if self.api_key.trim().is_empty() {
            return Err(AIError::Config(
                "缺少 API Key，请先在设置中配置 / Missing API key; configure it in settings"
                    .to_string(),
            ));
        }

        if self.base_url.trim().is_empty() {
            return Err(AIError::Config(
                "缺少 API Base URL / Missing API base URL".to_string(),
            ));
        }

        if self.model.trim().is_empty() {
            return Err(AIError::Config(
                "缺少模型名称 / Missing model name".to_string(),
            ));
        }

        Ok(())
    }

    fn map_reqwest_error(error: reqwest::Error) -> AIError {
        if error.is_timeout() {
            AIError::Timeout(
                "请求超过超时时间，请检查网络或稍后重试 / Request timed out".to_string(),
            )
        } else if error.is_connect() {
            AIError::ServiceUnavailable(format!(
                "无法连接到 AI 服务，请检查网络或服务地址 / Unable to connect to AI service: {}",
                error
            ))
        } else {
            AIError::Network(error)
        }
    }

    fn map_http_error(status: u16, error_text: String) -> AIError {
        let detail = if error_text.trim().is_empty() {
            format!("HTTP {}", status)
        } else {
            error_text
        };

        match status {
            400 => AIError::Api(format!(
                "请求格式无效，请检查模型和参数配置 / Invalid request: {}",
                detail
            )),
            401 | 403 => AIError::Authentication(format!(
                "API Key 无效或无权限 / Invalid credentials or permission denied: {}",
                detail
            )),
            404 => AIError::Api(format!(
                "接口或模型不存在 / Endpoint or model not found: {}",
                detail
            )),
            408 => AIError::Timeout(format!(
                "上游服务响应超时 / Upstream request timeout: {}",
                detail
            )),
            429 => AIError::RateLimit(format!(
                "请求过于频繁或额度已用尽 / Rate limited or quota exceeded: {}",
                detail
            )),
            500..=599 => AIError::ServiceUnavailable(format!(
                "AI 服务暂时不可用 / AI service temporarily unavailable: {}",
                detail
            )),
            _ => AIError::Api(detail),
        }
    }
}

// ============================================
// 消息构建函数 / Message Builder Functions
// ============================================

/// 构建续写消息 / Build continue messages
pub fn build_continue_messages(text: &str) -> Vec<Message> {
    vec![
        Message {
            role: "system".to_string(),
            content: "你是一个专业的写作助手。请根据用户提供的文本，自然地续写内容。续写应该与原文风格一致，内容连贯。".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: format!("请续写以下文本：\n\n{}", text),
        },
    ]
}

/// 构建优化消息 / Build improve messages
pub fn build_improve_messages(text: &str) -> Vec<Message> {
    vec![
        Message {
            role: "system".to_string(),
            content: "你是一个专业的文字编辑。请优化用户提供的文本，使其更加清晰、流畅、专业。保持原文的核心意思不变。".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: format!("请优化以下文本：\n\n{}", text),
        },
    ]
}

/// 构建大纲消息 / Build outline messages
pub fn build_outline_messages(topic: &str) -> Vec<Message> {
    vec![
        Message {
            role: "system".to_string(),
            content: "你是一个专业的内容策划师。请根据用户提供的主题，生成一个详细的 Markdown 格式大纲。使用 #、##、### 等标题层级。".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: format!("请为以下主题生成一个详细的大纲：\n\n{}", topic),
        },
    ]
}

/// 构建翻译消息 / Build translate messages
pub fn build_translate_messages(text: &str, target_lang: &str) -> Vec<Message> {
    vec![
        Message {
            role: "system".to_string(),
            content: format!(
                "你是一个专业的翻译师。请将用户提供的文本翻译成{}。保持原文的格式和风格。",
                target_lang
            ),
        },
        Message {
            role: "user".to_string(),
            content: text.to_string(),
        },
    ]
}

/// 构建语法修正消息 / Build grammar fix messages
pub fn build_grammar_messages(text: &str) -> Vec<Message> {
    vec![
        Message {
            role: "system".to_string(),
            content: "你是一个专业的语言校对员。请修正用户提供的文本中的语法、拼写和标点错误。只返回修正后的文本，不要解释。".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: text.to_string(),
        },
    ]
}

/// 构建自定义请求消息 / Build custom prompt messages
pub fn build_custom_messages(prompt: &str, text: &str) -> Vec<Message> {
    vec![
        Message {
            role: "system".to_string(),
            content: prompt.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: text.to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url_is_normalized() {
        let service = AIService::new(
            "key".to_string(),
            Some("https://api.openai.com/v1/".to_string()),
            Some("gpt-4o-mini".to_string()),
        );

        assert_eq!(service.base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_validate_config_requires_api_key() {
        let service = AIService::new(
            String::new(),
            Some("https://api.openai.com/v1".to_string()),
            Some("gpt-4o-mini".to_string()),
        );

        assert!(matches!(service.validate_config(), Err(AIError::Config(_))));
    }

    #[test]
    fn test_map_http_error_authentication() {
        let err = AIService::map_http_error(401, "bad key".to_string());
        assert!(matches!(err, AIError::Authentication(_)));
    }

    #[test]
    fn test_map_http_error_rate_limit() {
        let err = AIService::map_http_error(429, "quota".to_string());
        assert!(matches!(err, AIError::RateLimit(_)));
    }

    #[test]
    fn test_map_http_error_service_unavailable() {
        let err = AIService::map_http_error(503, "busy".to_string());
        assert!(matches!(err, AIError::ServiceUnavailable(_)));
    }
}
