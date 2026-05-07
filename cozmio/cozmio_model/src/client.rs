// cozmio/cozmio_model/src/client.rs
use crate::error::ModelError;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OLLAMA_API_URL: &str = "http://localhost:11434/api/chat";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);

const SYSTEM_PROMPT: &str = r#"你是一个窗口判断器。

你的任务不是描述页面，也不是推测用户完整意图。
你的任务是：根据窗口截图和机械元信息，判断当前证据是否足以支持 agent 继续介入。

只允许两种最终结果：

1. CONTINUE
表示：当前可见证据已经足以支持 agent 继续往前一轮判断。

2. ABSTAIN
表示：当前证据不足，agent 不应继续延伸。

要求：
- 只能依据截图和提供的机械元信息作判断。
- 不要补充截图中看不见、元信息中没有的事实。
- 不要输出页面描述作为最终结果。
- 不要输出世界标签，例如"idle状态""高风险操作""用户正在做X"。
- 不要推荐具体动作，不要替用户做决定。
- 不要展示思考过程，只输出最终结果。

输出格式必须严格为：

MODE: CONTINUE 或 MODE: ABSTAIN
REASON: 一句简短理由，理由只能引用可见证据。"#;

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterventionMode {
    Continue,
    Abstain,
}

impl std::fmt::Display for InterventionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterventionMode::Continue => write!(f, "CONTINUE"),
            InterventionMode::Abstain => write!(f, "ABSTAIN"),
        }
    }
}

pub struct InterventionResult {
    pub mode: InterventionMode,
    pub reason: String,
    pub raw_output: String,
}

pub fn parse_intervention_result(raw: &str) -> Result<InterventionResult, ModelError> {
    let mode_re = Regex::new(r"(?i)MODE:\s*(CONTINUE|ABSTAIN)").unwrap();
    let reason_re = Regex::new(r"(?i)REASON:\s*(.+)").unwrap();

    let mode_cap = mode_re
        .captures(raw)
        .ok_or_else(|| ModelError::ParseError("Missing MODE line".to_string()))?;
    let mode_str = mode_cap.get(1).unwrap().as_str().to_uppercase();
    let mode = if mode_str == "CONTINUE" {
        InterventionMode::Continue
    } else {
        InterventionMode::Abstain
    };

    let reason = reason_re
        .captures(raw)
        .map(|c| c.get(1).unwrap().as_str().trim().to_string())
        .ok_or_else(|| ModelError::ParseError("Missing REASON line".to_string()))?;

    Ok(InterventionResult {
        mode,
        reason,
        raw_output: raw.to_string(),
    })
}

/// 调用模型，返回原始自由文本输出
///
/// prompt 不引导任何输出方向，让模型自然表达。
///
/// # 参数
/// - `model`: 模型名称，如 "qwen2.5-vision"
/// - `screenshot_base64`: PNG 截图的 base64 编码
/// - `title`: 窗口标题
/// - `process_name`: 进程名
/// - `user_text`: 用户提供的额外文本
///
/// # 返回
/// 模型输出的原始自由文本
pub async fn ask_model(
    model: &str,
    screenshot_base64: &str,
    title: &str,
    process_name: &str,
    user_text: &str,
) -> Result<String, ModelError> {
    let result = ask_model_impl(model, screenshot_base64, title, process_name, user_text).await;

    // 如果是模型不存在错误，尝试自动发现可用模型并重试
    if let Err(ref e) = result {
        let err_str = e.to_string();
        if err_str.contains("not found") || err_str.contains("502") {
            log::warn!("模型 {} 不可用，尝试自动发现可用模型...", model);
            if let Ok(available_model) = discover_first_model("http://localhost:11434").await {
                if available_model != model {
                    log::info!("发现可用模型: {}", available_model);
                    return ask_model_impl(
                        &available_model,
                        screenshot_base64,
                        title,
                        process_name,
                        user_text,
                    )
                    .await;
                }
            }
        }
    }

    result
}

async fn ask_model_impl(
    model: &str,
    screenshot_base64: &str,
    title: &str,
    process_name: &str,
    user_text: &str,
) -> Result<String, ModelError> {
    let prompt = format!(
        "{}\n\n窗口标题: {}\n进程名: {}\n用户输入: {}",
        SYSTEM_PROMPT, title, process_name, user_text
    );

    let request = OllamaRequest {
        model: model.to_string(),
        messages: vec![OllamaMessage {
            role: "user".to_string(),
            content: prompt,
            images: Some(vec![screenshot_base64.to_string()]),
        }],
        stream: false,
    };

    let client = Client::builder()
        .timeout(DEFAULT_TIMEOUT)
        .build()
        .map_err(|e| ModelError::NetworkError(e.to_string()))?;

    let response = client
        .post(OLLAMA_API_URL)
        .json(&request)
        .send()
        .await
        .map_err(|e| ModelError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.map_err(|e| {
            ModelError::NetworkError(format!("failed to read response body: {}", e))
        })?;
        return Err(ModelError::ApiError(format!("HTTP {}: {}", status, body)));
    }

    let ollama_resp: OllamaResponse = response
        .json()
        .await
        .map_err(|e| ModelError::ParseError(e.to_string()))?;

    Ok(ollama_resp.message.content)
}

/// 同步版本的 ask_model
pub fn ask_model_sync(
    model: &str,
    screenshot_base64: &str,
    title: &str,
    process_name: &str,
    user_text: &str,
) -> Result<String, ModelError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| ModelError::NetworkError(e.to_string()))?;
    rt.block_on(ask_model(
        model,
        screenshot_base64,
        title,
        process_name,
        user_text,
    ))
}

/// 从 Ollama /api/tags 获取已安装模型列表，返回第一个模型名称
pub async fn discover_first_model(base_url: &str) -> Result<String, ModelError> {
    let tags_url = format!("{}/api/tags", base_url.trim_end_matches('/'));

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| ModelError::NetworkError(e.to_string()))?;

    let response = client
        .get(&tags_url)
        .send()
        .await
        .map_err(|e| ModelError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ModelError::ApiError(format!(
            "failed to get model tags: {}",
            response.status()
        )));
    }

    let tags_resp: OllamaTagsResponse = response
        .json()
        .await
        .map_err(|e| ModelError::ParseError(e.to_string()))?;

    tags_resp
        .models
        .first()
        .map(|m| m.name.clone())
        .ok_or_else(|| ModelError::ModelNotFound("no models installed".to_string()))
}

/// 同步版本的 discover_first_model
pub fn discover_first_model_sync(base_url: &str) -> Result<String, ModelError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| ModelError::NetworkError(e.to_string()))?;
    rt.block_on(discover_first_model(base_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_first_model_parse_response() {
        let json = r#"{"models":[{"name":"qwen3-vl:4b","model":"qwen3-vl:4b","modified_at":"2026-04-03T18:42:09.6585768+08:00","size":3295636135}]}"#;
        let parsed: OllamaTagsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.models.first().unwrap().name, "qwen3-vl:4b");
    }
}
