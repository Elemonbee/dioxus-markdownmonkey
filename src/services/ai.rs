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
    temperature: f32, // 温度参数 / Temperature
}

impl AIService {
    /// 创建新的 AI 服务（使用默认 120 秒超时）
    /// Create New AI Service (with default 120-second timeout)
    #[allow(dead_code)]
    pub fn new(api_key: String, base_url: Option<String>, model: Option<String>) -> Self {
        Self::with_timeout(api_key, base_url, model, DEFAULT_TIMEOUT_SECS)
    }

    /// 创建新的 AI 服务，并使用指定温度 / Create AI service with a specific temperature
    pub fn with_temperature(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
        temperature: f32,
    ) -> Self {
        let mut service = Self::with_timeout(api_key, base_url, model, DEFAULT_TIMEOUT_SECS);
        service.temperature = temperature.clamp(0.0, 1.0);
        service
    }

    /// 创建新的 AI 服务（可配置超时）
    /// Create New AI Service (configurable timeout)
    pub fn with_timeout(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
        timeout_secs: u64,
    ) -> Self {
        let normalized_base_url = Self::normalize_base_url(
            &base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        );

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            base_url: normalized_base_url,
            api_key,
            model: model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
            temperature: 0.7,
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

    /// 归一化 API Base URL，并兼容旧设置中缺失的 /v1 后缀。
    /// Normalize API base URLs and migrate older settings missing the /v1 suffix.
    pub fn normalize_base_url(base_url: &str) -> String {
        let trimmed = base_url.trim().trim_end_matches('/');
        match trimmed {
            "https://api.openai.com" => "https://api.openai.com/v1".to_string(),
            "https://api.deepseek.com" => "https://api.deepseek.com/v1".to_string(),
            "https://api.moonshot.cn" => "https://api.moonshot.cn/v1".to_string(),
            "https://openrouter.ai/api" => "https://openrouter.ai/api/v1".to_string(),
            "http://localhost:11434" => "http://localhost:11434/v1".to_string(),
            _ => trimmed.to_string(),
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
            temperature: Some(self.temperature),
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
            temperature: Some(self.temperature),
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
// AI 任务类型 / AI Task Types
// ============================================

/// AI 任务类型枚举 / AI Task Type Enum
///
/// 将 6 个独立的 builder 函数统一为一个枚举，
/// 每个变体携带自己的 system prompt 和 user prompt 模板。
/// Consolidates 6 separate builder functions into a single enum,
/// each variant carries its own system prompt and user prompt template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AITask {
    /// 续写 / Continue writing
    Continue,
    /// 优化 / Improve text
    Improve,
    /// 大纲 / Generate outline
    Outline,
    /// 翻译 / Translate text
    Translate,
    /// 语法修正 / Fix grammar
    FixGrammar,
    /// 自定义请求 / Custom prompt
    Custom,
}

impl AITask {
    /// 从字符串标识符解析任务类型 / Parse task type from string identifier
    pub fn from_str_id(s: &str) -> Self {
        match s {
            "continue" => Self::Continue,
            "improve" => Self::Improve,
            "outline" => Self::Outline,
            "translate" => Self::Translate,
            "fix_grammar" => Self::FixGrammar,
            "custom" => Self::Custom,
            _ => Self::Custom,
        }
    }

    /// 获取结果弹窗标题的 i18n key / Get i18n key for result modal title
    pub fn title_i18n_key(&self) -> &'static str {
        match self {
            Self::Continue => "ai_continue_result",
            Self::Improve => "ai_improve_result",
            Self::Outline => "ai_outline_result",
            Self::Translate => "ai_translate_result",
            Self::FixGrammar => "ai_grammar_result",
            Self::Custom => "ai_response",
        }
    }

    /// 构建消息列表 / Build message list
    ///
    /// - `content`: 编辑器当前文档内容
    /// - `input`: 用户自定义输入（大纲主题、自定义提示词等）
    pub fn build_messages(&self, content: &str, input: &str) -> Vec<Message> {
        let (system_prompt, user_content) = self.build_prompts(content, input);
        vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_content,
            },
        ]
    }

    /// 生成 system prompt 和 user content / Generate system prompt and user content
    fn build_prompts(&self, content: &str, input: &str) -> (String, String) {
        match self {
            Self::Continue => (
                "你是一个专业的写作助手。请根据用户提供的文本，自然地续写内容。续写应该与原文风格一致，内容连贯。".to_string(),
                format!("请续写以下文本：\n\n{}", content),
            ),
            Self::Improve => (
                "你是一个专业的文字编辑。请优化用户提供的文本，使其更加清晰、流畅、专业。保持原文的核心意思不变。".to_string(),
                format!("请优化以下文本：\n\n{}", content),
            ),
            Self::Outline => (
                "你是一个专业的内容策划师。请根据用户提供的主题，生成一个详细的 Markdown 格式大纲。使用 #、##、### 等标题层级。".to_string(),
                format!("请为以下主题生成一个详细的大纲：\n\n{}", if input.is_empty() { content } else { input }),
            ),
            Self::Translate => (
                "你是一个专业的翻译师。请将用户提供的文本翻译成English。保持原文的格式和风格。".to_string(),
                content.to_string(),
            ),
            Self::FixGrammar => (
                "你是一个专业的语言校对员。请修正用户提供的文本中的语法、拼写和标点错误。只返回修正后的文本，不要解释。".to_string(),
                content.to_string(),
            ),
            Self::Custom => (
                if input.is_empty() {
                    "你是一个智能助手。请根据用户的要求提供帮助。".to_string()
                } else {
                    input.to_string()
                },
                content.to_string(),
            ),
        }
    }
}

/// 格式化 AI 错误为用户友好的显示文本 / Format AI error for user-friendly display
pub fn format_ai_error(error: &AIError, prefix: &str) -> String {
    match error {
        AIError::Config(msg)
        | AIError::Authentication(msg)
        | AIError::RateLimit(msg)
        | AIError::ServiceUnavailable(msg)
        | AIError::Timeout(msg)
        | AIError::Api(msg)
        | AIError::Parse(msg) => format!("{}: {}", prefix, msg),
        AIError::Network(err) => format!(
            "{}: 网络请求失败，请检查连接后重试 / Network request failed: {}",
            prefix, err
        ),
    }
}

// ============================================
// 模型列表获取 / Model List Fetching
// ============================================

/// Ollama 模型列表响应 / Ollama model list response
#[derive(Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

/// OpenRouter 模型列表响应 / OpenRouter model list response
#[derive(Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Deserialize)]
struct OpenRouterModel {
    id: String,
}

/// 获取指定提供商的可用模型列表 / Fetch available models for a given provider
///
/// 仅支持 Ollama 和 OpenRouter。其他提供商返回 Config 错误。
/// Only supports Ollama and OpenRouter. Other providers return a Config error.
pub async fn fetch_available_models(
    provider: &AIProvider,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<String>, AIError> {
    match provider {
        AIProvider::Ollama => fetch_ollama_models(base_url).await,
        AIProvider::OpenRouter => fetch_openrouter_models(base_url, api_key).await,
        _ => Err(AIError::Config(
            "Model listing not supported for this provider".to_string(),
        )),
    }
}

/// 从 Ollama 获取本地模型列表 / Fetch local model list from Ollama
///
/// Ollama base_url 通常是 `http://localhost:11434/v1`，
/// tags 端点在根路径：`http://localhost:11434/api/tags`
async fn fetch_ollama_models(base_url: &str) -> Result<Vec<String>, AIError> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new());

    // 去掉 /v1 后缀，拼接 /api/tags
    let tags_url = base_url
        .trim_end_matches('/')
        .trim_end_matches("/v1")
        .to_string()
        + "/api/tags";

    let response = client
        .get(&tags_url)
        .send()
        .await
        .map_err(AIService::map_reqwest_error)?;

    if !response.status().is_success() {
        return Err(AIError::ServiceUnavailable(
            "无法连接到 Ollama 服务，请确认 Ollama 已启动 / Cannot connect to Ollama service"
                .to_string(),
        ));
    }

    let body: OllamaModelsResponse = response.json().await?;
    let mut models: Vec<String> = body.models.into_iter().map(|m| m.name).collect();
    models.sort();
    Ok(models)
}

/// 从 OpenRouter 获取可用模型列表 / Fetch available models from OpenRouter
async fn fetch_openrouter_models(base_url: &str, api_key: &str) -> Result<Vec<String>, AIError> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| Client::new());

    let models_url = format!("{}/models", base_url.trim_end_matches('/'));

    let mut request = client.get(&models_url);
    if !api_key.trim().is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request.send().await.map_err(AIService::map_reqwest_error)?;

    if !response.status().is_success() {
        return Err(AIError::ServiceUnavailable(
            "无法获取 OpenRouter 模型列表 / Cannot fetch OpenRouter model list".to_string(),
        ));
    }

    let body: OpenRouterModelsResponse = response.json().await?;
    let mut models: Vec<String> = body.data.into_iter().map(|m| m.id).collect();
    models.sort();
    Ok(models)
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
    fn test_base_url_adds_legacy_openai_v1_suffix() {
        let service = AIService::new(
            "key".to_string(),
            Some("https://api.openai.com".to_string()),
            Some("gpt-4o-mini".to_string()),
        );

        assert_eq!(service.base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_temperature_is_clamped() {
        let service = AIService::with_temperature(
            "key".to_string(),
            Some("https://api.openai.com/v1".to_string()),
            Some("gpt-4o-mini".to_string()),
            1.5,
        );

        assert_eq!(service.temperature, 1.0);
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

    // --- AITask tests ---

    #[test]
    fn test_ai_task_from_str_id() {
        assert_eq!(AITask::from_str_id("continue"), AITask::Continue);
        assert_eq!(AITask::from_str_id("improve"), AITask::Improve);
        assert_eq!(AITask::from_str_id("outline"), AITask::Outline);
        assert_eq!(AITask::from_str_id("translate"), AITask::Translate);
        assert_eq!(AITask::from_str_id("fix_grammar"), AITask::FixGrammar);
        assert_eq!(AITask::from_str_id("custom"), AITask::Custom);
        // Unknown falls back to Custom
        assert_eq!(AITask::from_str_id("unknown"), AITask::Custom);
    }

    #[test]
    fn test_ai_task_title_i18n_key() {
        assert_eq!(AITask::Continue.title_i18n_key(), "ai_continue_result");
        assert_eq!(AITask::Improve.title_i18n_key(), "ai_improve_result");
        assert_eq!(AITask::Outline.title_i18n_key(), "ai_outline_result");
        assert_eq!(AITask::Translate.title_i18n_key(), "ai_translate_result");
        assert_eq!(AITask::FixGrammar.title_i18n_key(), "ai_grammar_result");
        assert_eq!(AITask::Custom.title_i18n_key(), "ai_response");
    }

    #[test]
    fn test_ai_task_build_messages_continue() {
        let msgs = AITask::Continue.build_messages("Hello world", "");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "system");
        assert!(msgs[1].content.contains("Hello world"));
    }

    #[test]
    fn test_ai_task_build_messages_custom_with_input() {
        let msgs = AITask::Custom.build_messages("doc content", "Summarize this");
        assert_eq!(msgs[0].content, "Summarize this");
        assert_eq!(msgs[1].content, "doc content");
    }

    #[test]
    fn test_ai_task_build_messages_custom_empty_input() {
        let msgs = AITask::Custom.build_messages("doc content", "");
        assert_eq!(
            msgs[0].content,
            "你是一个智能助手。请根据用户的要求提供帮助。"
        );
    }

    #[test]
    fn test_ai_task_build_messages_outline_uses_input() {
        let msgs = AITask::Outline.build_messages("fallback", "My Topic");
        assert!(msgs[1].content.contains("My Topic"));
        assert!(!msgs[1].content.contains("fallback"));
    }

    #[test]
    fn test_ai_task_build_messages_outline_falls_back_to_content() {
        let msgs = AITask::Outline.build_messages("fallback content", "");
        assert!(msgs[1].content.contains("fallback content"));
    }

    #[test]
    fn test_format_ai_error_api() {
        let err = AIError::Api("test error".to_string());
        let msg = format_ai_error(&err, "Error");
        assert!(msg.contains("Error: test error"));
    }

    #[test]
    fn test_format_ai_error_config() {
        let err = AIError::Config("bad config".to_string());
        let msg = format_ai_error(&err, "Error");
        assert!(msg.contains("Error: bad config"));
    }

    #[test]
    fn test_format_ai_error_auth() {
        let err = AIError::Authentication("invalid key".to_string());
        let msg = format_ai_error(&err, "Error");
        assert!(msg.contains("Error: invalid key"));
    }
}
