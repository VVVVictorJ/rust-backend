use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiServiceError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("api error: {0}")]
    ApiError(String),
    #[error("env error: {0}")]
    EnvError(String),
    #[error("parse error: {0}")]
    ParseError(String),
}

/// Qwen API 请求消息
#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Qwen API 请求体
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    max_tokens: u32,
}

/// Qwen API 响应结构
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

/// AI 分析结果
#[derive(Debug)]
pub struct AiAnalysisResult {
    /// 解析后的结构化 JSON（如果解析成功）
    pub response_json: Option<JsonValue>,
    /// 原始文本响应
    pub raw_response: String,
}

/// 创建用于调用 Qwen API 的 HTTP 客户端（不需要代理）
pub fn create_ai_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
}

/// 读取系统 prompt
pub fn load_system_prompt() -> String {
    include_str!("../asset/prompt/盘中启动信号全息回溯与趋势诊断系统 Prompt.md").to_string()
}

/// 调用 Qwen API 进行趋势分析
pub async fn call_qwen_analysis(
    client: &Client,
    user_payload: &JsonValue,
) -> Result<AiAnalysisResult, AiServiceError> {
    let api_url = std::env::var("QWEN_API_URL")
        .unwrap_or_else(|_| "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_string());
    let api_key = std::env::var("QWEN_API_KEY")
        .map_err(|_| AiServiceError::EnvError("QWEN_API_KEY not set".to_string()))?;
    let model = std::env::var("QWEN_MODEL")
        .unwrap_or_else(|_| "qwen3-max-2026-01-23".to_string());

    let system_prompt = load_system_prompt();

    let user_content = serde_json::to_string_pretty(user_payload)
        .map_err(AiServiceError::SerdeJson)?;

    let request_body = ChatCompletionRequest {
        model,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_content,
            },
        ],
        temperature: 0.3,
        max_tokens: 8192,
    };

    tracing::info!("Calling Qwen API for stock analysis...");

    let response = client
        .post(&api_url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(AiServiceError::Http)?;

    let status = response.status();
    let response_text = response.text().await.map_err(AiServiceError::Http)?;

    if !status.is_success() {
        tracing::error!("Qwen API returned error status {}: {}", status, response_text);
        return Err(AiServiceError::ApiError(format!(
            "API returned status {status}: {response_text}"
        )));
    }

    // 解析 API 响应
    let api_response: ChatCompletionResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            tracing::error!("Failed to parse Qwen API response: {} | raw: {}", e, response_text);
            AiServiceError::ParseError(format!("Failed to parse API response: {e}"))
        })?;

    let raw_content = api_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    tracing::info!("Qwen API returned {} chars of content", raw_content.len());

    // 尝试从返回的内容中提取 JSON
    let response_json = extract_json_from_response(&raw_content);

    Ok(AiAnalysisResult {
        response_json,
        raw_response: raw_content,
    })
}

/// 从 AI 响应中提取 JSON
/// 处理可能包含 markdown 代码块包裹的情况
fn extract_json_from_response(content: &str) -> Option<JsonValue> {
    let trimmed = content.trim();

    // 尝试直接解析
    if let Ok(json) = serde_json::from_str::<JsonValue>(trimmed) {
        return Some(json);
    }

    // 尝试从 ```json ... ``` 代码块中提取
    if let Some(start) = trimmed.find("```json") {
        let json_start = start + 7;
        if let Some(end) = trimmed[json_start..].find("```") {
            let json_str = trimmed[json_start..json_start + end].trim();
            if let Ok(json) = serde_json::from_str::<JsonValue>(json_str) {
                return Some(json);
            }
        }
    }

    // 尝试从 ``` ... ``` 代码块中提取
    if let Some(start) = trimmed.find("```") {
        let json_start = start + 3;
        // 跳过可能的语言标识符行
        let json_start = if let Some(newline_pos) = trimmed[json_start..].find('\n') {
            json_start + newline_pos + 1
        } else {
            json_start
        };
        if let Some(end) = trimmed[json_start..].find("```") {
            let json_str = trimmed[json_start..json_start + end].trim();
            if let Ok(json) = serde_json::from_str::<JsonValue>(json_str) {
                return Some(json);
            }
        }
    }

    // 尝试找到第一个 { 和最后一个 } 之间的内容
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if start < end {
            let json_str = &trimmed[start..=end];
            if let Ok(json) = serde_json::from_str::<JsonValue>(json_str) {
                return Some(json);
            }
        }
    }

    tracing::warn!("Failed to extract JSON from AI response");
    None
}
