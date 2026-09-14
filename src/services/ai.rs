//! AI 服务 - 处理与 AI API 的交互 / AI Service - Handle AI API Interactions

use crate::state::{AIProvider, AiApplyContext, Language};
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 默认请求超时（秒）/ Default request timeout (seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// AI 错误类型 / AI Error Types
#[derive(Debug)]
pub enum AIError {
    /// 网络错误 / Network error
    Network(reqwest::Error),
    /// API 错误 / API error
    Api(String),
    /// 认证失败 / Authentication error
    Authentication(String),
    /// 请求被限流 / Rate limit
    RateLimit(String),
    /// 服务暂时不可用 / Service unavailable
    ServiceUnavailable(String),
    /// 配置错误 / Config error
    Config(String),
    /// 请求超时 / Request timeout
    Timeout(String),
    /// 解析错误 / Parse error
    Parse(String),
    /// 生成已取消 / Generation cancelled
    Cancelled,
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(e) => write!(f, "网络错误/Network Error: {e}"),
            Self::Api(e) => write!(f, "API 错误/API Error: {e}"),
            Self::Authentication(e) => write!(f, "认证失败/Authentication Error: {e}"),
            Self::RateLimit(e) => write!(f, "请求被限流/Rate Limit: {e}"),
            Self::ServiceUnavailable(e) => {
                write!(f, "服务暂时不可用/Service Unavailable: {e}")
            }
            Self::Config(e) => write!(f, "配置错误/Config Error: {e}"),
            Self::Timeout(e) => write!(f, "请求超时/Request Timeout: {e}"),
            Self::Parse(e) => write!(f, "解析错误/Parse Error: {e}"),
            Self::Cancelled => write!(f, "生成已取消/Generation cancelled"),
        }
    }
}

impl std::error::Error for AIError {}

impl From<reqwest::Error> for AIError {
    fn from(e: reqwest::Error) -> Self {
        Self::Network(e)
    }
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

impl Message {
    /// 系统 system 消息 / Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// 创建 user 消息 / Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// 创建 assistant 消息 / Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    /// 角色字符串 / Role string
    #[allow(dead_code)]
    pub fn role(&self) -> &str {
        &self.role
    }

    /// 内容字符串 / Content string
    #[allow(dead_code)]
    pub fn content(&self) -> &str {
        &self.content
    }
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

    /// 构建 OpenAI 兼容请求体，统一流式与非流式参数
    /// Build an OpenAI-compatible request body shared by streaming and non-streaming calls
    fn build_openai_request(
        &self,
        messages: Vec<Message>,
        stream: bool,
        max_tokens: u32,
    ) -> AIRequest {
        AIRequest {
            model: self.model.clone(),
            messages,
            stream,
            temperature: Some(self.temperature),
            max_tokens: Some(max_tokens),
        }
    }

    /// 构建 Claude 请求体并分离 system 消息
    /// Build a Claude request body while separating system messages
    fn build_claude_request(
        &self,
        messages: Vec<Message>,
        stream: bool,
        max_tokens: u32,
    ) -> serde_json::Value {
        use serde_json::json;

        let mut system_parts = Vec::new();
        let mut claude_messages = Vec::new();
        for message in messages {
            if message.role == "system" {
                if !message.content.trim().is_empty() {
                    system_parts.push(message.content);
                }
            } else {
                claude_messages.push(json!({
                    "role": message.role,
                    "content": message.content
                }));
            }
        }

        let mut body = json!({
            "model": self.model,
            "messages": claude_messages,
            "max_tokens": max_tokens,
            "temperature": self.temperature,
            "stream": stream,
        });
        if !system_parts.is_empty() {
            body["system"] = json!(system_parts.join("\n\n"));
        }
        body
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
        let request = self.build_openai_request(messages, false, 2048);

        let response = self
            .apply_bearer_auth(
                self.client
                    .post(format!("{}/chat/completions", self.base_url))
                    .header("Content-Type", "application/json"),
            )
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
        let body = self.build_claude_request(messages, false, 2048);

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
    #[allow(dead_code)] // 测试与无取消场景的便捷入口 / Convenience for tests / non-cancellable callers
    pub async fn chat_stream<F>(
        &self,
        messages: Vec<Message>,
        on_chunk: F,
    ) -> Result<String, AIError>
    where
        F: FnMut(&str),
    {
        let (_tx, rx) = tokio::sync::watch::channel(false);
        self.chat_stream_cancellable(messages, on_chunk, || true, rx)
            .await
    }

    /// 可取消的流式聊天 / Cancellable streaming chat
    ///
    /// `should_continue` 返回 false 或 `cancel_rx` 变为 true 时提前结束；
    /// 已收到内容则返回 Ok(部分结果)，并通过 drop stream 中止 HTTP 读。
    /// Stops early when `should_continue` is false or `cancel_rx` is true;
    /// returns Ok(partial) if any content, and drops the stream to abort HTTP reads.
    pub async fn chat_stream_cancellable<F, C>(
        &self,
        messages: Vec<Message>,
        mut on_chunk: F,
        should_continue: C,
        cancel_rx: tokio::sync::watch::Receiver<bool>,
    ) -> Result<String, AIError>
    where
        F: FnMut(&str),
        C: FnMut() -> bool,
    {
        self.validate_config()?;

        // 检测是否为 Anthropic Claude API
        // Detect if Anthropic Claude API
        let is_claude = self.base_url.contains("anthropic.com");

        if is_claude {
            return self
                .chat_stream_claude_cancellable(messages, on_chunk, should_continue, cancel_rx)
                .await;
        }

        let request = self.build_openai_request(messages, true, 4096);

        let response = self
            .apply_bearer_auth(
                self.client
                    .post(format!("{}/chat/completions", self.base_url))
                    .header("Content-Type", "application/json"),
            )
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
            should_continue,
            cancel_rx,
            Self::parse_openai_sse_data,
            "No response content received from stream",
        )
        .await
    }

    /// Claude SSE 流式响应 / Claude SSE streaming response
    async fn chat_stream_claude_cancellable<F, C>(
        &self,
        messages: Vec<Message>,
        mut on_chunk: F,
        should_continue: C,
        cancel_rx: tokio::sync::watch::Receiver<bool>,
    ) -> Result<String, AIError>
    where
        F: FnMut(&str),
        C: FnMut() -> bool,
    {
        let body = self.build_claude_request(messages, true, 4096);

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
            should_continue,
            cancel_rx,
            Self::parse_claude_sse_data,
            "No response content received from Claude stream",
        )
        .await
    }

    /// 收集 SSE 内容；`cancel_rx` 可在等待下一 chunk 时中止 HTTP 读
    /// Collect SSE content; `cancel_rx` aborts the HTTP body while awaiting the next chunk
    pub(crate) async fn collect_sse_content<S, B, F, C, P>(
        mut stream: S,
        on_chunk: &mut F,
        mut should_continue: C,
        mut cancel_rx: tokio::sync::watch::Receiver<bool>,
        mut parse_data: P,
        empty_message: &str,
    ) -> Result<String, AIError>
    where
        S: Stream<Item = Result<B, reqwest::Error>> + Unpin,
        B: AsRef<[u8]>,
        F: FnMut(&str),
        C: FnMut() -> bool,
        P: FnMut(&str) -> Option<String>,
    {
        let mut full_content = String::new();
        let mut line_buffer = String::new();

        if *cancel_rx.borrow() || !should_continue() {
            return if full_content.is_empty() {
                Err(AIError::Cancelled)
            } else {
                Ok(full_content)
            };
        }

        loop {
            tokio::select! {
                biased;
                changed = cancel_rx.changed() => {
                    if changed.is_err() || *cancel_rx.borrow() {
                        drop(stream);
                        return if full_content.is_empty() {
                            Err(AIError::Cancelled)
                        } else {
                            Ok(full_content)
                        };
                    }
                }
                chunk_result = stream.next() => {
                    match chunk_result {
                        None => break,
                        Some(Err(e)) => return Err(Self::map_reqwest_error(e)),
                        Some(Ok(chunk)) => {
                            if !should_continue() || *cancel_rx.borrow() {
                                drop(stream);
                                return if full_content.is_empty() {
                                    Err(AIError::Cancelled)
                                } else {
                                    Ok(full_content)
                                };
                            }
                            line_buffer.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                            Self::drain_sse_lines(
                                &mut line_buffer,
                                &mut full_content,
                                on_chunk,
                                &mut parse_data,
                            );
                        }
                    }
                }
            }
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
        // Ollama 本地服务通常不需要 API Key / Local Ollama usually needs no API key
        if self.api_key.trim().is_empty() && !self.is_local_ollama() {
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

    /// 是否为本地 Ollama 端点 / Whether base URL points at local Ollama
    fn is_local_ollama(&self) -> bool {
        let url = self.base_url.to_ascii_lowercase();
        url.contains("localhost:11434") || url.contains("127.0.0.1:11434")
    }

    /// 为请求附加 Authorization（无 key 时跳过，兼容 Ollama）
    /// Attach Authorization header (skip when empty for Ollama compatibility)
    fn apply_bearer_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.api_key.trim().is_empty() {
            request
        } else {
            request.header("Authorization", format!("Bearer {}", self.api_key))
        }
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

/// 从文档中提取安全的选区上下文 / Extract a safe selected context from a document
pub fn selected_ai_context(content: &str, start: usize, end: usize) -> Option<String> {
    let (mut from, mut to) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    from = from.min(content.len());
    to = to.min(content.len());
    if from == to {
        return None;
    }

    while from > 0 && !content.is_char_boundary(from) {
        from -= 1;
    }
    while to < content.len() && !content.is_char_boundary(to) {
        to += 1;
    }

    let selected = &content[from..to];
    if selected.trim().is_empty() {
        None
    } else {
        Some(selected.to_string())
    }
}

/// 按字符数截断 AI 上下文并附上截断标记
/// Truncate AI context by char count and append a truncation marker
pub fn truncate_ai_context(content: &str, max_chars: usize) -> String {
    if content.chars().count() <= max_chars {
        return content.to_string();
    }
    let mut truncated: String = content.chars().take(max_chars).collect();
    truncated.push_str("\n\n[... truncated / 已截断 ...]");
    truncated
}

/// 第 n 个字符对应的 UTF-8 字节偏移（越界则返回全文长度）
/// UTF-8 byte offset of the n-th character (or the full length when out of range)
fn byte_of_nth_char(text: &str, n: usize) -> usize {
    text.char_indices()
        .nth(n)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

/// 把字节光标对齐到字符边界
/// Snap a byte cursor onto a char boundary
fn snap_cursor_bytes(text: &str, cursor_bytes: usize) -> usize {
    let mut cursor = cursor_bytes.min(text.len());
    while cursor > 0 && !text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}

/// 按字数取光标附近窗口，过长时加 `[...]` 标记
/// Take a character window around the cursor; mark with `[...]` when clipped
pub fn clip_text_window(text: &str, cursor_bytes: usize, max_chars: usize) -> String {
    let max_chars = max_chars.max(1);
    let total = text.chars().count();
    if total <= max_chars {
        return text.to_string();
    }

    let cursor = snap_cursor_bytes(text, cursor_bytes);
    let cursor_chars = text[..cursor].chars().count();
    let half = max_chars / 2;
    let mut start_chars = cursor_chars.saturating_sub(half);
    if start_chars + max_chars > total {
        start_chars = total.saturating_sub(max_chars);
    }
    let end_chars = start_chars + max_chars;
    let start_byte = byte_of_nth_char(text, start_chars);
    let end_byte = byte_of_nth_char(text, end_chars);

    let mut out = String::new();
    if start_byte > 0 {
        out.push_str("[...]\n");
    }
    out.push_str(&text[start_byte..end_byte]);
    if end_byte < text.len() {
        out.push_str("\n[...]");
    }
    out
}

/// 按字数保留文本尾部，过长时加 `[...]` 标记
/// Keep the tail of text within a character budget; mark with `[...]` when clipped
pub fn clip_chars_tail(text: &str, max_chars: usize) -> String {
    let max_chars = max_chars.max(1);
    let total = text.chars().count();
    if total <= max_chars {
        return text.to_string();
    }
    let start_byte = byte_of_nth_char(text, total - max_chars);
    let mut out = String::from("[...]\n");
    out.push_str(&text[start_byte..]);
    out
}

/// 聊天附带的文档上下文：有选区则截选区尾部，否则取光标附近窗口
/// Chat document context: clip the selection tail when present, else a window around the cursor
pub fn clip_chat_document_context(
    text: &str,
    selection: Option<&str>,
    cursor_bytes: usize,
    max_chars: usize,
) -> String {
    match selection {
        Some(sel) if !sel.is_empty() => clip_chars_tail(sel, max_chars),
        _ => clip_text_window(text, cursor_bytes, max_chars),
    }
}

/// 在文档中定位应被 AI 结果替换的字节范围
/// Locate the byte range that an AI result should replace
pub fn find_ai_replace_range(content: &str, ctx: &AiApplyContext) -> Option<(usize, usize)> {
    if ctx.source_text.is_empty() {
        return None;
    }

    let mut start = ctx.source_start.min(content.len());
    let mut end = ctx.source_end.min(content.len());
    while start > 0 && !content.is_char_boundary(start) {
        start -= 1;
    }
    while end < content.len() && !content.is_char_boundary(end) {
        end += 1;
    }

    if start < end && content[start..end] == ctx.source_text {
        return Some((start, end));
    }
    content
        .find(&ctx.source_text)
        .map(|i| (i, i + ctx.source_text.len()))
}

/// 在选区结束后插入时补上合适的换行
/// Choose a newline gap when inserting after a selection
pub fn ai_insert_after_payload(content: &str, offset: usize, text: &str) -> String {
    let offset = offset.min(content.len());
    let gap = if offset == 0 || content[..offset].ends_with("\n\n") {
        ""
    } else if content[..offset].ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    format!("{gap}{text}")
}

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

/// AI 结果写回文档的方式 / How an AI result is written back into the document
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiApplyKind {
    /// 插在选区后面 / Insert after the selection
    InsertAfterSelection,
    /// 替换选区 / Replace the selection
    ReplaceSelection,
    /// 插入当前光标 / Insert at the cursor
    InsertCursor,
    /// 追加到文末 / Append to the document
    Append,
    /// 替换全文 / Replace the whole document
    ReplaceDocument,
}

impl AiApplyKind {
    /// 按钮文案的 i18n key / i18n key for the apply-button label
    pub fn i18n_key(self) -> &'static str {
        match self {
            Self::InsertAfterSelection => "ai_insert_after_selection",
            Self::ReplaceSelection => "ai_replace_selection",
            Self::InsertCursor => "ai_insert_cursor",
            Self::Append => "append",
            Self::ReplaceDocument => "replace_doc",
        }
    }
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

    /// 该任务在选区/全文下的主写回动作
    /// Primary apply action for this task given selection vs full document
    pub fn primary_apply_kind(self, used_selection: bool) -> AiApplyKind {
        match (self, used_selection) {
            (Self::Continue, true) | (Self::Outline, true) => AiApplyKind::InsertAfterSelection,
            (Self::Continue, false) => AiApplyKind::Append,
            (Self::Improve | Self::Translate | Self::FixGrammar, true) => {
                AiApplyKind::ReplaceSelection
            }
            (Self::Improve | Self::Translate | Self::FixGrammar, false) => {
                AiApplyKind::ReplaceDocument
            }
            (Self::Outline, false) | (Self::Custom, _) => AiApplyKind::InsertCursor,
        }
    }

    /// 结果弹窗上展示的写回动作（第一项为主按钮）
    /// Apply actions shown on the result modal (first item is primary)
    pub fn apply_kinds(self, used_selection: bool) -> Vec<AiApplyKind> {
        let primary = self.primary_apply_kind(used_selection);
        let rest: &[AiApplyKind] = match (self, used_selection) {
            (Self::Continue, true) => &[
                AiApplyKind::InsertCursor,
                AiApplyKind::Append,
                AiApplyKind::ReplaceSelection,
            ],
            (Self::Continue, false) => &[AiApplyKind::InsertCursor, AiApplyKind::ReplaceDocument],
            (Self::Improve | Self::Translate | Self::FixGrammar, true) => &[
                AiApplyKind::InsertAfterSelection,
                AiApplyKind::InsertCursor,
                AiApplyKind::Append,
            ],
            (Self::Improve | Self::Translate | Self::FixGrammar, false) => {
                &[AiApplyKind::InsertCursor, AiApplyKind::Append]
            }
            (Self::Outline, true) => &[AiApplyKind::InsertCursor, AiApplyKind::Append],
            (Self::Outline, false) => &[AiApplyKind::Append],
            (Self::Custom, true) => &[
                AiApplyKind::InsertAfterSelection,
                AiApplyKind::Append,
                AiApplyKind::ReplaceSelection,
            ],
            (Self::Custom, false) => &[AiApplyKind::Append, AiApplyKind::ReplaceDocument],
        };
        let mut kinds = Vec::with_capacity(1 + rest.len());
        kinds.push(primary);
        kinds.extend(rest.iter().copied().filter(|kind| *kind != primary));
        kinds
    }

    /// 是否适合原文/结果对照（原地改写任务）
    /// Whether original-vs-result compare is useful (in-place rewrite tasks)
    pub fn shows_compare(self, used_selection: bool) -> bool {
        used_selection && matches!(self, Self::Improve | Self::Translate | Self::FixGrammar)
    }

    /// 预设任务必须基于选区；聊天（Custom）不强制
    /// Preset tasks require a selection; chat (Custom) does not
    pub fn requires_selection(self) -> bool {
        matches!(
            self,
            Self::Continue | Self::Improve | Self::Outline | Self::Translate | Self::FixGrammar
        )
    }

    /// 只有聊天才带上会话历史和全局 system，避免续写历史把翻译带跑偏
    /// Only chat attaches transcript and the global system prompt, so a prior Continue cannot hijack Translate
    pub fn uses_chat_session(self) -> bool {
        matches!(self, Self::Custom)
    }

    /// 构建消息列表 / Build message list
    ///
    /// - `content`: 编辑器当前文档内容
    /// - `input`: 用户自定义输入（大纲主题、自定义提示词等）
    #[allow(dead_code)] // 测试与无历史调用的便捷入口 / Convenience for tests / no-history callers
    pub fn build_messages(&self, content: &str, input: &str) -> Vec<Message> {
        self.build_messages_with_history(content, input, &[], "")
    }

    /// 构建带会话历史与可选全局 system 的消息列表
    /// Build messages with conversation history and optional global system prompt
    ///
    /// `history` 仅含 user/assistant；`global_system` 会拼到任务 system 前缀
    /// `history` is user/assistant only; `global_system` is prefixed onto the task system prompt
    pub fn build_messages_with_history(
        &self,
        content: &str,
        input: &str,
        history: &[crate::state::ChatTurn],
        global_system: &str,
    ) -> Vec<Message> {
        self.build_messages_localized(
            content,
            input,
            history,
            global_system,
            Language::ZhCN,
            Language::EnUS,
        )
    }

    /// 按 UI 语言与翻译目标构建带历史的消息
    /// Build history-aware messages using UI language and translate target
    pub fn build_messages_localized(
        &self,
        content: &str,
        input: &str,
        history: &[crate::state::ChatTurn],
        global_system: &str,
        ui_lang: Language,
        translate_target: Language,
    ) -> Vec<Message> {
        const MAX_HISTORY_TURNS: usize = 10;

        let (task_system, user_content) =
            self.build_prompts(content, input, ui_lang, translate_target);
        let system_prompt = if global_system.trim().is_empty() {
            task_system
        } else {
            format!("{}\n\n{}", global_system.trim(), task_system)
        };

        let mut messages = Vec::with_capacity(2 + history.len().min(MAX_HISTORY_TURNS));
        messages.push(Message::system(system_prompt));

        // 只保留最近 N 轮 / Keep only the most recent N turns
        let start = history.len().saturating_sub(MAX_HISTORY_TURNS);
        for turn in &history[start..] {
            let role = turn.role.as_str();
            if role == "assistant" {
                messages.push(Message::assistant(turn.content.clone()));
            } else if role == "user" {
                messages.push(Message::user(turn.content.clone()));
            }
        }

        messages.push(Message::user(user_content));
        messages
    }

    /// 生成本轮写入历史的用户文本摘要 / User-turn text stored into history
    #[allow(dead_code)]
    pub fn history_user_summary(&self, content: &str, input: &str) -> String {
        self.history_user_summary_localized(content, input, Language::ZhCN, Language::EnUS)
    }

    /// 按 UI 语言生成本轮写入历史的用户摘要
    /// Localized user-turn text stored into history
    pub fn history_user_summary_localized(
        &self,
        content: &str,
        input: &str,
        ui_lang: Language,
        translate_target: Language,
    ) -> String {
        if matches!(self, Self::Custom) {
            return input.trim().to_string();
        }
        let (_, user_content) = self.build_prompts(content, input, ui_lang, translate_target);
        user_content
    }

    /// 生成 system prompt 和 user content / Generate system prompt and user content
    fn build_prompts(
        &self,
        content: &str,
        input: &str,
        ui_lang: Language,
        translate_target: Language,
    ) -> (String, String) {
        let zh = matches!(ui_lang, Language::ZhCN);
        let target_name = match (zh, translate_target) {
            (true, Language::EnUS) => "英文",
            (true, Language::ZhCN) => "简体中文",
            (false, Language::EnUS) => "English",
            (false, Language::ZhCN) => "Simplified Chinese",
        };
        match self {
            Self::Continue => {
                if zh {
                    (
                        "你是一个专业的写作助手。请根据用户提供的文本，自然地续写内容。续写应该与原文风格一致，内容连贯。".to_string(),
                        format!("请续写以下文本：\n\n{}", content),
                    )
                } else {
                    (
                        "You are a professional writing assistant. Continue the user's text naturally, matching its style and staying coherent.".to_string(),
                        format!("Continue the following text:\n\n{}", content),
                    )
                }
            }
            Self::Improve => {
                if zh {
                    (
                        "你是一个专业的文字编辑。请优化用户提供的文本，使其更加清晰、流畅、专业。保持原文的核心意思不变。".to_string(),
                        format!("请优化以下文本：\n\n{}", content),
                    )
                } else {
                    (
                        "You are a professional editor. Improve the user's text so it is clearer, smoother, and more professional, without changing the core meaning.".to_string(),
                        format!("Improve the following text:\n\n{}", content),
                    )
                }
            }
            Self::Outline => {
                let topic = if input.is_empty() { content } else { input };
                if zh {
                    (
                        "你是一个专业的内容策划师。请根据用户提供的主题，生成一个详细的 Markdown 格式大纲。使用 #、##、### 等标题层级。".to_string(),
                        format!("请为以下主题生成一个详细的大纲：\n\n{}", topic),
                    )
                } else {
                    (
                        "You are a professional content planner. Create a detailed Markdown outline from the user's topic using #, ##, and ### headings.".to_string(),
                        format!("Create a detailed outline for the following topic:\n\n{}", topic),
                    )
                }
            }
            Self::Translate => {
                if zh {
                    (
                        format!(
                            "你是一个专业的翻译师。请将用户提供的文本翻译成{}。保持原文的格式和风格。只返回译文，不要续写，不要解释。",
                            target_name
                        ),
                        format!("请将以下文本翻译成{}：\n\n{}", target_name, content),
                    )
                } else {
                    (
                        format!(
                            "You are a professional translator. Translate the user's text into {}. Preserve formatting and style. Return only the translation; do not continue the text or add explanations.",
                            target_name
                        ),
                        format!("Translate the following text into {}:\n\n{}", target_name, content),
                    )
                }
            }
            Self::FixGrammar => {
                if zh {
                    (
                        "你是一个专业的语言校对员。请修正用户提供的文本中的语法、拼写和标点错误。只返回修正后的文本，不要解释。".to_string(),
                        content.to_string(),
                    )
                } else {
                    (
                        "You are a professional proofreader. Fix grammar, spelling, and punctuation in the user's text. Return only the corrected text with no explanation.".to_string(),
                        content.to_string(),
                    )
                }
            }
            Self::Custom => {
                let system = if zh {
                    "你是 Markdown 写作助手。根据用户的问题和当前提供的文档上下文回答。上下文可能只是选区或光标附近的片段；不要把上下文当成续写或翻译任务，除非用户明确要求。".to_string()
                } else {
                    "You are a Markdown writing assistant. Answer the user's question using the attached document context. The context may be a selection or a window around the cursor. Do not continue or translate unless the user asks.".to_string()
                };
                let user = if content.trim().is_empty() {
                    input.to_string()
                } else if zh {
                    format!("【文档上下文】\n{}\n\n【问题】\n{}", content, input)
                } else {
                    format!("Document context:\n{}\n\nQuestion:\n{}", content, input)
                };
                (system, user)
            }
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
        AIError::Cancelled => format!("{}: 已取消生成 / Generation cancelled", prefix),
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

    /// OpenAI 流式与非流式请求都应携带配置温度
    /// OpenAI streaming and non-streaming bodies both carry the configured temperature
    #[test]
    fn test_openai_request_bodies_use_configured_temperature() {
        let service = AIService::with_temperature(
            "key".to_string(),
            Some("https://api.openai.com/v1".to_string()),
            Some("gpt-test".to_string()),
            0.35,
        );
        for stream in [false, true] {
            let request = service.build_openai_request(vec![Message::user("hello")], stream, 2048);
            let body = serde_json::to_value(request).unwrap();
            let temperature = body["temperature"].as_f64().unwrap();
            assert!((temperature - 0.35).abs() < 0.0001);
            assert_eq!(body["stream"], serde_json::json!(stream));
        }
    }

    /// Claude 流式与非流式请求都应携带温度并正确拆分 system
    /// Claude streaming and non-streaming bodies carry temperature and separate system correctly
    #[test]
    fn test_claude_request_bodies_use_temperature_and_system() {
        let service = AIService::with_temperature(
            "key".to_string(),
            Some("https://api.anthropic.com/v1".to_string()),
            Some("claude-test".to_string()),
            0.4,
        );
        for stream in [false, true] {
            let body = service.build_claude_request(
                vec![Message::system("global"), Message::user("hello")],
                stream,
                2048,
            );
            let temperature = body["temperature"].as_f64().unwrap();
            assert!((temperature - 0.4).abs() < 0.0001);
            assert_eq!(body["stream"], serde_json::json!(stream));
            assert_eq!(body["system"], "global");
            assert_eq!(body["messages"][0]["role"], "user");
        }
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
    fn test_validate_config_allows_ollama_without_api_key() {
        let service = AIService::new(
            String::new(),
            Some("http://localhost:11434/v1".to_string()),
            Some("llama3".to_string()),
        );

        assert!(service.validate_config().is_ok());
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

    #[test]
    fn test_selected_ai_context_extracts_selection() {
        let selected = selected_ai_context("Hello 世界", 6, 12).unwrap();
        assert_eq!(selected, "世界");
    }

    #[test]
    fn test_selected_ai_context_ignores_empty_selection() {
        assert_eq!(selected_ai_context("Hello", 2, 2), None);
        assert_eq!(selected_ai_context("Hello   world", 5, 8), None);
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
        assert!(msgs[0].content.contains("写作助手"));
        assert!(msgs[1].content.contains("doc content"));
        assert!(msgs[1].content.contains("Summarize this"));
        assert_eq!(
            AITask::Custom.history_user_summary("doc content", "Summarize this"),
            "Summarize this"
        );
    }

    #[test]
    fn test_ai_task_build_messages_custom_empty_input() {
        let msgs = AITask::Custom.build_messages("doc content", "");
        assert!(msgs[0].content.contains("写作助手"));
        assert!(msgs[1].content.contains("【文档上下文】"));
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

    /// 多轮历史会插入到 system 与最新 user 之间
    /// Multi-turn history is inserted between system and the latest user message
    #[test]
    fn test_ai_task_build_messages_with_history() {
        use crate::state::ChatTurn;
        let history = vec![
            ChatTurn::user("first question"),
            ChatTurn::assistant("first answer"),
        ];
        let msgs =
            AITask::Custom.build_messages_with_history("doc", "Summarize", &history, "Be concise.");
        assert_eq!(msgs.len(), 4);
        assert_eq!(msgs[0].role, "system");
        assert!(msgs[0].content.contains("Be concise."));
        assert_eq!(msgs[1].role, "user");
        assert_eq!(msgs[1].content, "first question");
        assert_eq!(msgs[2].role, "assistant");
        assert_eq!(msgs[2].content, "first answer");
        assert_eq!(msgs[3].role, "user");
        assert!(msgs[3].content.contains("doc"));
        assert!(msgs[3].content.contains("Summarize"));
        assert_ne!(msgs[3].content, "doc");
    }

    /// 空全局提示词不得改变原任务消息
    /// An empty global prompt must preserve the original task messages
    #[test]
    fn test_empty_global_system_preserves_existing_behavior() {
        let original = AITask::Improve.build_messages("body", "");
        let with_empty = AITask::Improve.build_messages_with_history("body", "", &[], "   ");
        assert_eq!(original[0].role(), with_empty[0].role());
        assert_eq!(original[0].content(), with_empty[0].content());
        assert_eq!(original[1].content(), with_empty[1].content());
    }

    /// 英文 UI 应生成英文任务提示词 / English UI should produce English task prompts
    #[test]
    fn test_ai_task_build_messages_english_prompts() {
        use crate::state::Language;
        let msgs = AITask::Continue.build_messages_localized(
            "Hello world",
            "",
            &[],
            "",
            Language::EnUS,
            Language::ZhCN,
        );
        assert!(msgs[0].content.contains("writing assistant"));
        assert!(msgs[1].content.contains("Continue the following text"));
        assert!(msgs[1].content.contains("Hello world"));
    }

    /// 翻译目标语言应写入 system prompt / Translate target language should appear in the system prompt
    #[test]
    fn test_ai_task_translate_target_language() {
        use crate::state::Language;
        let to_zh = AITask::Translate.build_messages_localized(
            "Hello",
            "",
            &[],
            "",
            Language::EnUS,
            Language::ZhCN,
        );
        assert!(to_zh[0].content.contains("Simplified Chinese"));
        assert!(to_zh[1]
            .content
            .contains("Translate the following text into Simplified Chinese"));
        assert!(to_zh[1].content.contains("Hello"));

        let to_en = AITask::Translate.build_messages_localized(
            "你好",
            "",
            &[],
            "",
            Language::ZhCN,
            Language::EnUS,
        );
        assert!(to_en[0].content.contains("英文"));
        assert!(to_en[0].content.contains("不要续写"));
        assert!(to_en[1].content.contains("请将以下文本翻译成英文"));
        assert!(to_en[1].content.contains("你好"));
        assert!(!to_en[1].content.contains("请续写"));
        assert!(!AITask::Translate.uses_chat_session());
        assert!(AITask::Custom.uses_chat_session());
    }

    /// 超长上下文按字符截断 / Long context is truncated by char count
    #[test]
    fn test_truncate_ai_context() {
        let text = "你好世界abcd";
        assert_eq!(truncate_ai_context(text, 100), text);
        let truncated = truncate_ai_context(text, 4);
        assert!(truncated.starts_with("你好世界"));
        assert!(truncated.contains("truncated"));
    }

    /// 替换范围优先使用记录的选区，否则回退搜索
    /// Replace range prefers the recorded span, then falls back to search
    #[test]
    fn test_find_ai_replace_range() {
        let content = "你好世界";
        let mut ctx = AiApplyContext {
            task_id: "improve".into(),
            used_selection: true,
            source_start: "你".len(),
            source_end: "你好世".len(),
            source_text: "好世".into(),
            request_content: "好世".into(),
            request_input: String::new(),
            is_error: false,
        };
        assert_eq!(
            find_ai_replace_range(content, &ctx),
            Some(("你".len(), "你好世".len()))
        );

        ctx.source_start = 0;
        ctx.source_end = 1;
        assert_eq!(
            find_ai_replace_range(content, &ctx),
            Some(("你".len(), "你好世".len()))
        );

        ctx.source_text.clear();
        assert_eq!(find_ai_replace_range(content, &ctx), None);
    }

    /// 选区后续写主动作是插在选区后；优化/翻译/语法是替换原文
    /// Continue-after-selection inserts after; improve/translate/grammar replace in place
    #[test]
    fn test_ai_task_apply_kinds() {
        assert_eq!(
            AITask::Continue.primary_apply_kind(true),
            AiApplyKind::InsertAfterSelection
        );
        assert_eq!(
            AITask::Continue.primary_apply_kind(false),
            AiApplyKind::Append
        );
        assert_eq!(
            AITask::Improve.primary_apply_kind(true),
            AiApplyKind::ReplaceSelection
        );
        assert_eq!(
            AITask::Translate.primary_apply_kind(false),
            AiApplyKind::ReplaceDocument
        );
        assert_eq!(
            AITask::Outline.primary_apply_kind(true),
            AiApplyKind::InsertAfterSelection
        );
        assert_eq!(
            AITask::FixGrammar.primary_apply_kind(true),
            AiApplyKind::ReplaceSelection
        );
        assert_eq!(
            AITask::Custom.primary_apply_kind(false),
            AiApplyKind::InsertCursor
        );
        assert_eq!(
            AITask::Custom.primary_apply_kind(true),
            AiApplyKind::InsertCursor
        );
        assert_eq!(
            AITask::Continue.apply_kinds(true)[0],
            AiApplyKind::InsertAfterSelection
        );
        assert!(!AITask::Continue.shows_compare(true));
        assert!(AITask::Improve.shows_compare(true));
        assert!(!AITask::Improve.shows_compare(false));
        assert!(!AITask::Custom.shows_compare(true));
        assert!(AITask::Continue.requires_selection());
        assert!(AITask::Improve.requires_selection());
        assert!(AITask::Outline.requires_selection());
        assert!(AITask::Translate.requires_selection());
        assert!(AITask::FixGrammar.requires_selection());
        assert!(!AITask::Custom.requires_selection());
    }

    /// 聊天上下文按字数裁剪：选区取尾部，否则取光标窗口
    /// Chat context is clipped by char budget: selection tail, else a cursor window
    #[test]
    fn test_clip_chat_document_context() {
        let doc = "abcdefghij";
        assert_eq!(clip_text_window(doc, 0, 20), doc);
        let window = clip_text_window(doc, 5, 4);
        assert!(window.contains("defg"));
        assert!(window.contains("[...]"));

        let tail = clip_chars_tail("一二三四五六七八九十", 3);
        assert!(tail.ends_with("八九十"));
        assert!(tail.starts_with("[...]"));

        let from_sel = clip_chat_document_context(doc, Some("xyz123"), 0, 3);
        assert!(from_sel.ends_with("123"));
        let from_cursor = clip_chat_document_context(doc, None, 0, 3);
        assert!(from_cursor.contains("abc"));
    }

    /// 插在选区后应补空行，已有空行则不再加
    /// Insert-after adds a blank line unless one is already present
    #[test]
    fn test_ai_insert_after_payload() {
        assert_eq!(
            ai_insert_after_payload("你好世界", "你好".len(), "续"),
            "\n\n续"
        );
        assert_eq!(
            ai_insert_after_payload("你好\n", "你好\n".len(), "续"),
            "\n续"
        );
        assert_eq!(
            ai_insert_after_payload("你好\n\n", "你好\n\n".len(), "续"),
            "续"
        );
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

    /// 取消信号应打断挂起的 SSE 等待 / Cancel signal must interrupt a pending SSE wait
    #[tokio::test]
    async fn test_sse_cancel_aborts_pending_stream() {
        use futures_util::stream;
        use std::time::Duration;

        let (tx, rx) = tokio::sync::watch::channel(false);
        let pending = stream::pending::<Result<Vec<u8>, reqwest::Error>>();

        let task = tokio::spawn(async move {
            AIService::collect_sse_content(pending, &mut |_| {}, || true, rx, |_| None, "empty")
                .await
        });

        tokio::time::sleep(Duration::from_millis(30)).await;
        let _ = tx.send(true);

        let result = tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .expect("cancel should finish promptly")
            .expect("join ok");
        assert!(matches!(result, Err(AIError::Cancelled)));
    }
}
