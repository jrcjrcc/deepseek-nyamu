//! LLM 提供商抽象层模块。
//!
//! 本模块定义了统一的 LLM API 提供商接口，支持 OpenAI 兼容 API 和 Anthropic Messages API
//! 两种协议。包含以下核心组件：
//!
//! - [`Provider`]：提供商枚举，派发到具体的 LLM API 后端（支持 19 个提供商，包括 Anthropic）。
//! - [`ProviderClient`]：具体的 HTTP 客户端，封装请求构造、指数退避重试和流式响应解析。
//! - [`ProviderRegistry`]：提供商注册表，管理多个 `Provider` 实例的注册、查询和默认设置。
//! - [`ChatRequest`] / [`ChatMessage`] / [`ChatResult`]：通用聊天请求/响应数据结构。
//! - [`StreamEvent`]：流式事件枚举，包含文本块（`Chunk`）、思维链推理（`reasoning`）、
//!   工具调用增量（`ToolCallDelta`）、工具调用就绪（`ToolCallReady`）、完成（`Done`）
//!   和用量统计（`Usage`）。
//! - [`RetryConfig`] / [`with_retry`]：指数退避重试辅助函数，支持可配置的最大重试次数和延迟范围。
//! - [`parse_openai_sse`]：OpenAI 兼容 SSE 流解析器，将字节流解析为结构化的 `StreamEvent`。
//! - [`parse_openai_chunk`]：单个 SSE 数据块的 JSON 解析函数。
//!
//! Provider abstraction layer for LLM API clients.
//!
//! Defines [`Provider`] — an enum dispatching to concrete LLM API backends —
//! along with retry/backoff helpers and SSE stream parsing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use futures::StreamExt;
use nyamu_config::{ConfigToml, ProviderKind, ResolvedRuntimeOptions};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::warn;

// ─── Re-exports ───────────────────────────────────────────────────────

pub use nyamu_agent::{ModelFamily, ModelInfo, ModelRegistry, ModelResolution};

// ─── Stream event types ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Chunk { content: String, reasoning: Option<String> },
    ToolCallDelta { index: usize, id: Option<String>, name: Option<String>, arguments_chunk: String },
    ToolCallReady { id: String, name: String, arguments: String },
    Done { finish_reason: String },
    Usage { input_tokens: u64, output_tokens: u64, cache_hit: u64, cache_miss: u64 },
}

// ─── Generic request types ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub system: Option<String>,
    pub tools: Option<Vec<Value>>,
    pub tool_choice: String,
    pub stream: bool,
    pub max_tokens: u32,
    pub reasoning_effort: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ApiToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: ApiToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub struct ChatResult {
    pub content: String,
    pub reasoning: Option<String>,
    pub tool_calls: Vec<ApiToolCall>,
    pub finish_reason: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_hit: u64,
    pub cache_miss: u64,
    pub model: String,
}

// ─── Retry helper ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_retries: 3, base_delay_ms: 1000, max_delay_ms: 30_000 }
    }
}

pub async fn with_retry<T, F, Fut>(retry: &RetryConfig, operation: &str, f: F) -> Result<T>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<T>> + Send,
{
    let mut last_err = None;
    for attempt in 1..=retry.max_retries {
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                warn!("operation '{operation}' attempt {attempt}/{} failed: {e}", retry.max_retries);
                last_err = Some(e);
                if attempt < retry.max_retries {
                    let delay = retry.base_delay_ms.saturating_mul(1u64 << (attempt - 1)).min(retry.max_delay_ms);
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("retry '{operation}' exhausted")))
}

// ─── Provider enum ────────────────────────────────────────────────────

/// An LLM provider instance, wrapping a concrete API client.
#[derive(Clone)]
pub enum Provider {
    Deepseek(ProviderClient),
    Openai(ProviderClient),
    Openrouter(ProviderClient),
    NvidiaNim(ProviderClient),
    Atlascloud(ProviderClient),
    WanjieArk(ProviderClient),
    Volcengine(ProviderClient),
    XiaomiMimo(ProviderClient),
    Novita(ProviderClient),
    Fireworks(ProviderClient),
    Siliconflow(ProviderClient),
    SiliconflowCN(ProviderClient),
    Arcee(ProviderClient),
    Moonshot(ProviderClient),
    Sglang(ProviderClient),
    Vllm(ProviderClient),
    Ollama(ProviderClient),
    Huggingface(ProviderClient),
    Anthropic(ProviderClient),
}

impl Provider {
    /// Create a provider from resolved runtime options.
    pub fn from_options(kind: ProviderKind, options: ResolvedRuntimeOptions) -> Self {
        let client = ProviderClient::new(options, kind);
        use ProviderKind::*;
        match kind {
            Deepseek => Self::Deepseek(client),
            NvidiaNim => Self::NvidiaNim(client),
            Openai => Self::Openai(client),
            Atlascloud => Self::Atlascloud(client),
            WanjieArk => Self::WanjieArk(client),
            Volcengine => Self::Volcengine(client),
            Openrouter => Self::Openrouter(client),
            XiaomiMimo => Self::XiaomiMimo(client),
            Novita => Self::Novita(client),
            Fireworks => Self::Fireworks(client),
            Siliconflow => Self::Siliconflow(client),
            SiliconflowCN => Self::SiliconflowCN(client),
            Arcee => Self::Arcee(client),
            Moonshot => Self::Moonshot(client),
            Sglang => Self::Sglang(client),
            Vllm => Self::Vllm(client),
            Ollama => Self::Ollama(client),
            Huggingface => Self::Huggingface(client),
        }
    }

    /// Provider kind string.
    pub fn name(&self) -> &'static str {
        use Provider::*;
        match self {
            Deepseek(_) => "deepseek",
            NvidiaNim(_) => "nvidia-nim",
            Openai(_) => "openai",
            Atlascloud(_) => "atlascloud",
            WanjieArk(_) => "wanjie-ark",
            Volcengine(_) => "volcengine",
            Openrouter(_) => "openrouter",
            XiaomiMimo(_) => "xiaomi-mimo",
            Novita(_) => "novita",
            Fireworks(_) => "fireworks",
            Siliconflow(_) => "siliconflow",
            SiliconflowCN(_) => "siliconflow-cn",
            Arcee(_) => "arcee",
            Moonshot(_) => "moonshot",
            Sglang(_) => "sglang",
            Vllm(_) => "vllm",
            Ollama(_) => "ollama",
            Huggingface(_) => "huggingface",
            Anthropic(_) => "anthropic",
        }
    }

    /// Stream chat.
    pub async fn stream_chat(
        &self,
        request: &ChatRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamEvent>> + Send + Unpin>> {
        match self {
            Provider::Anthropic(c) => c.stream_chat_via_anthropic(request).await,
            _ => {
                let c = self.client();
                c.stream_chat(request).await
            }
        }
    }

    fn client(&self) -> &ProviderClient {
        use Provider::*;
        match self {
            Deepseek(c) | NvidiaNim(c) | Openai(c) | Atlascloud(c)
            | WanjieArk(c) | Volcengine(c) | Openrouter(c) | XiaomiMimo(c)
            | Novita(c) | Fireworks(c) | Siliconflow(c) | SiliconflowCN(c)
            | Arcee(c) | Moonshot(c) | Sglang(c) | Vllm(c) | Ollama(c)
            | Huggingface(c) | Anthropic(c) => c,
        }
    }
}

// ─── Provider client ──────────────────────────────────────────────────

/// Concrete HTTP client for an LLM provider.
#[derive(Clone)]
pub struct ProviderClient {
    pub options: ResolvedRuntimeOptions,
    kind: ProviderKind,
    client: Client,
    retry: RetryConfig,
}

impl ProviderClient {
    pub fn new(options: ResolvedRuntimeOptions, kind: ProviderKind) -> Self {
        Self {
            options,
            kind,
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap_or_default(),
            retry: RetryConfig::default(),
        }
    }

    /// Stream via OpenAI-compatible API.
    pub async fn stream_chat(
        &self,
        chat_req: &ChatRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamEvent>> + Send + Unpin>> {
        let url = format!("{}/chat/completions", self.options.base_url.trim_end_matches('/'));
        let name = self.kind.as_str();

        let mut body = serde_json::json!({
            "model": chat_req.model,
            "messages": chat_req.messages,
            "stream": chat_req.stream,
            "max_tokens": chat_req.max_tokens,
            "tool_choice": chat_req.tool_choice,
        });
        if let Some(system) = &chat_req.system {
            body["system"] = Value::String(system.clone());
        }
        if let Some(tools) = &chat_req.tools {
            body["tools"] = Value::Array(tools.clone());
        }
        if let Some(effort) = &chat_req.reasoning_effort {
            body["reasoning_effort"] = Value::String(effort.clone());
        }

        let retry = self.retry.clone();
        let options = self.options.clone();
        let url_clone = url.clone();
        let body_clone = body.clone();

        let response = with_retry(&retry, &format!("{name}/chat"), || async {
            let client = Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .map_err(|e| anyhow::anyhow!("build client: {e}"))?;
            let mut rb = client.post(&url_clone)
                .header("Content-Type", "application/json");
            if let Some(ref key) = options.api_key {
                rb = rb.header("Authorization", format!("Bearer {}", key));
            }
            for (k, v) in &options.http_headers {
                rb = rb.header(k.as_str(), v.as_str());
            }
            rb.json(&body_clone)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("request failed: {e}"))
        })
        .await
        .with_context(|| format!("{name} stream_chat failed"))?;

        if !response.status().is_success() {
            let status = response.status();
            let resp_body = response.text().await.unwrap_or_default();
            anyhow::bail!("{name} API error {status}: {resp_body}");
        }

        let byte_stream = response.bytes_stream().map(|chunk| {
            chunk.map_err(|e| anyhow::anyhow!("stream error: {e}"))
        });
        Ok(Box::new(parse_openai_sse(byte_stream)))
    }

    /// Stream via Anthropic Messages API.
    pub async fn stream_chat_via_anthropic(
        &self,
        chat_req: &ChatRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<StreamEvent>> + Send + Unpin>> {
        let url = format!("{}/v1/messages", self.options.base_url.trim_end_matches('/'));
        let name = "anthropic";

        let mut anthropic_messages = Vec::new();
        let mut system_text = String::new();
        for msg in &chat_req.messages {
            match msg.role.as_str() {
                "system" => { if let Some(ref c) = msg.content { system_text.push_str(c); system_text.push('\n'); } }
                "user" | "assistant" => {
                    let mut content = Vec::new();
                    if let Some(ref c) = msg.content {
                        content.push(serde_json::json!({"type": "text", "text": c}));
                    }
                    if let Some(ref tcs) = msg.tool_calls {
                        for tc in tcs {
                            content.push(serde_json::json!({
                                "type": "tool_use", "id": tc.id, "name": tc.function.name,
                                "input": serde_json::from_str::<Value>(&tc.function.arguments).unwrap_or(Value::Null),
                            }));
                        }
                    }
                    anthropic_messages.push(serde_json::json!({"role": msg.role, "content": content}));
                }
                "tool" => {
                    anthropic_messages.push(serde_json::json!({
                        "role": "user",
                        "content": [{"type": "tool_result", "tool_use_id": msg.tool_call_id, "content": msg.content}],
                    }));
                }
                _ => {}
            }
        }

        let mut body = serde_json::json!({
            "model": chat_req.model,
            "messages": anthropic_messages,
            "max_tokens": chat_req.max_tokens,
            "stream": chat_req.stream,
        });
        if !system_text.is_empty() {
            body["system"] = Value::String(system_text.trim().to_string());
        }
        if let Some(ref tools) = chat_req.tools {
            body["tools"] = Value::Array(tools.clone());
        }

        let retry = self.retry.clone();
        let options = self.options.clone();

        let response = with_retry(&retry, &format!("{name}/messages"), || async {
            let client = Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .map_err(|e| anyhow::anyhow!("build client: {e}"))?;
            let mut rb = client.post(&url)
                .header("Content-Type", "application/json")
                .header("anthropic-version", "2023-06-01");
            if let Some(ref key) = options.api_key {
                rb = rb.header("x-api-key", key.as_str());
            }
            rb.json(&body)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("request failed: {e}"))
        })
        .await
        .with_context(|| "anthropic stream_chat failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let resp_body = response.text().await.unwrap_or_default();
            anyhow::bail!("{name} API error {status}: {resp_body}");
        }

        let stream = response.bytes_stream().map(|chunk| {
            let bytes = chunk.map_err(|e| anyhow::anyhow!("stream error: {e}"))?;
            Ok(StreamEvent::Chunk {
                content: String::from_utf8_lossy(&bytes).to_string(),
                reasoning: None,
            })
        });
        Ok(Box::new(stream))
    }
}

// ─── Provider registry ─────────────────────────────────────────────────

/// Registry of available [`Provider`] instances, keyed by provider name string.
#[derive(Clone)]
pub struct ProviderRegistry {
    providers: Arc<Mutex<HashMap<String, Provider>>>,
    default_kind: Arc<Mutex<ProviderKind>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(Mutex::new(HashMap::new())),
            default_kind: Arc::new(Mutex::new(ProviderKind::Deepseek)),
        }
    }

    pub async fn register(&self, kind: ProviderKind, provider: Provider) {
        self.providers.lock().await.insert(kind.as_str().to_string(), provider);
    }

    pub async fn get(&self, kind: ProviderKind) -> Option<Provider> {
        self.providers.lock().await.get(kind.as_str()).cloned()
    }

    pub async fn default(&self) -> Option<Provider> {
        let kind = *self.default_kind.lock().await;
        self.get(kind).await
    }

    pub async fn set_default(&self, kind: ProviderKind) {
        *self.default_kind.lock().await = kind;
    }

    /// Register built-in providers from config.
    pub async fn register_builtins(&self, config: &ConfigToml) {
        let secrets = nyamu_secrets::Secrets::file_backed();
        for kind in &[
            ProviderKind::Deepseek,
            ProviderKind::NvidiaNim,
            ProviderKind::Openai,
            ProviderKind::Openrouter,
        ] {
            let cli = nyamu_config::CliRuntimeOverrides {
                provider: Some(*kind),
                ..Default::default()
            };
            let options = config.resolve_runtime_options_with_secrets(&cli, &secrets);
            let provider = Provider::from_options(*kind, options);
            self.register(*kind, provider).await;
        }
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── SSE parsing for OpenAI-compatible streams ─────────────────────────

fn parse_openai_sse(
    byte_stream: impl futures::Stream<Item = Result<bytes::Bytes, anyhow::Error>> + Send + Unpin + 'static,
) -> impl futures::Stream<Item = Result<StreamEvent, anyhow::Error>> + Send + Unpin {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    struct SseStream {
        byte_stream: Pin<Box<dyn futures::Stream<Item = Result<bytes::Bytes, anyhow::Error>> + Send>>,
        buffer: String,
        current_event_data: String,
        /// When a chunk has both tool_calls delta and finish_reason,
        /// we save the finish_reason here so [DONE] can use it.
        pending_finish_reason: String,
    }

    impl futures::Stream for SseStream {
        type Item = Result<StreamEvent, anyhow::Error>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            // Always try to process buffered data first before polling the byte stream
            loop {
                // Try to process a line from the buffer
                if let Some(line_end) = self.buffer.find('\n') {
                    let line = self.buffer[..line_end].to_string();
                    self.buffer = self.buffer[line_end + 1..].to_string();
                    let trimmed = line.trim();

                    if trimmed.is_empty() {
                        if !self.current_event_data.is_empty() {
                            let data = std::mem::take(&mut self.current_event_data);
                            if data == "[DONE]" {
                                let fr = std::mem::take(&mut self.pending_finish_reason);
                                let reason = if fr.is_empty() { "stop".to_string() } else { fr };
                                return Poll::Ready(Some(Ok(StreamEvent::Done { finish_reason: reason })));
                            } else {
                                // Save finish_reason from this chunk if present
                                // (used when tool_calls + finish_reason come together)
                                if let Ok(val) = serde_json::from_str::<Value>(&data) {
                                    if let Some(fr) = val.pointer("/choices/0/finish_reason").and_then(Value::as_str) {
                                        if !fr.is_empty() && fr != "null" {
                                            self.pending_finish_reason = fr.to_string();
                                        }
                                    }
                                }
                                return Poll::Ready(Some(parse_openai_chunk(&data)));
                            }
                        }
                    } else if let Some(data) = trimmed.strip_prefix("data: ") {
                        self.current_event_data = data.to_string();
                    }
                    // Continue loop to process more lines
                    continue;
                }

                // Buffer is empty — need more data from the byte stream
                match Pin::new(&mut self.byte_stream).poll_next(cx) {
                    Poll::Ready(Some(Ok(chunk))) => {
                        self.buffer.push_str(&String::from_utf8_lossy(&chunk));
                        // Continue loop to process the new data
                        continue;
                    }
                    Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                    Poll::Ready(None) => {
                        // Stream ended; flush any remaining event data
                        if !self.current_event_data.is_empty() {
                            let data = std::mem::take(&mut self.current_event_data);
                            return Poll::Ready(Some(parse_openai_chunk(&data)));
                        }
                        return Poll::Ready(None);
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
    }

    SseStream {
        byte_stream: Box::pin(byte_stream),
        buffer: String::new(),
        current_event_data: String::new(),
        pending_finish_reason: String::new(),
    }
}

fn parse_openai_chunk(data: &str) -> Result<StreamEvent, anyhow::Error> {
    let v: Value = serde_json::from_str(data)
        .with_context(|| format!("failed to parse SSE chunk: {data:.100}"))?;

    if let Some(usage) = v.get("usage") {
        if !usage.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            return Ok(StreamEvent::Usage {
                input_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0),
                output_tokens: usage["completion_tokens"].as_u64().unwrap_or(0),
                cache_hit: usage["prompt_cache_hit_tokens"].as_u64().unwrap_or(0),
                cache_miss: usage["prompt_cache_miss_tokens"].as_u64().unwrap_or(0),
            });
        }
    }

    if let Some(choices) = v["choices"].as_array() {
        if let Some(choice) = choices.first() {
            if let Some(delta) = choice["delta"].as_object() {
                // Process tool_calls BEFORE finish_reason — the final arguments chunk
                // arrives in the same SSE event as finish_reason="tool_calls"
                if let Some(tcs) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                    if let Some(tc) = tcs.first() {
                        return Ok(StreamEvent::ToolCallDelta {
                            index: tc["index"].as_i64().unwrap_or(0) as usize,
                            id: tc["id"].as_str().map(|s| s.to_string()),
                            name: tc["function"]["name"].as_str().map(|s| s.to_string()),
                            arguments_chunk: tc["function"]["arguments"].as_str().unwrap_or("").to_string(),
                        });
                    }
                }

                let content = delta.get("content").and_then(|c| c.as_str()).unwrap_or("");
                let reasoning = delta.get("reasoning_content")
                    .or_else(|| delta.get("reasoning"))
                    .and_then(|r| r.as_str())
                    .map(|s| s.to_string());

                if !content.is_empty() || reasoning.is_some() {
                    return Ok(StreamEvent::Chunk { content: content.to_string(), reasoning });
                }
            }

            // Only return Done when there's no delta content to process
            let finish_reason = choice["finish_reason"].as_str().unwrap_or("");
            if !finish_reason.is_empty() && finish_reason != "null" {
                return Ok(StreamEvent::Done { finish_reason: finish_reason.to_string() });
            }
        }
    }

    Ok(StreamEvent::Chunk { content: String::new(), reasoning: None })
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_kind_str() {
        assert_eq!(ProviderKind::Deepseek.as_str(), "deepseek");
    }

    #[test]
    fn test_parse_text_chunk() {
        let data = r#"{"choices":[{"delta":{"content":"Hello"},"finish_reason":null,"index":0}]}"#;
        let event = parse_openai_chunk(data).unwrap();
        assert!(matches!(event, StreamEvent::Chunk { .. }));
    }

    #[test]
    fn test_parse_done_chunk() {
        let data = r#"{"choices":[{"delta":{},"finish_reason":"stop","index":0}]}"#;
        let event = parse_openai_chunk(data).unwrap();
        assert!(matches!(event, StreamEvent::Done { .. }));
    }

    #[test]
    fn test_parse_reasoning_chunk() {
        let data = r#"{"choices":[{"delta":{"reasoning_content":"Thinking..."},"finish_reason":null,"index":0}]}"#;
        let event = parse_openai_chunk(data).unwrap();
        match event {
            StreamEvent::Chunk { content: _, reasoning } => {
                assert_eq!(reasoning, Some("Thinking...".to_string()));
            }
            _ => panic!("expected Chunk with reasoning"),
        }
    }

    #[test]
    fn test_parse_usage_chunk() {
        let data = r#"{"usage":{"prompt_tokens":10,"completion_tokens":20,"prompt_cache_hit_tokens":5,"prompt_cache_miss_tokens":5}}"#;
        let event = parse_openai_chunk(data).unwrap();
        assert!(matches!(event, StreamEvent::Usage { .. }));
    }
}
