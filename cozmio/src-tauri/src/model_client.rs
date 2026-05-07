#[cfg(test)]
use regex::Regex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::window_monitor::WindowSnapshot;

/// Legacy parsed model output modes, retained only for tests.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterventionMode {
    Continue,
    Abstain,
}

#[cfg(test)]
impl std::fmt::Display for InterventionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterventionMode::Continue => write!(f, "CONTINUE"),
            InterventionMode::Abstain => write!(f, "ABSTAIN"),
        }
    }
}

/// Legacy parsed model output, retained only for tests.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct ModelOutput {
    pub mode: InterventionMode,
    pub reason: String,
    /// What the agent will do if user confirms. Only set for CONTINUE.
    pub user_how: Option<String>,
}

/// Raw model output with metadata - model output is preserved as-is without parsing
#[derive(Debug, Clone)]
pub struct ModelRawOutput {
    /// The raw text output from the model - natural language, not parsed
    pub raw_text: String,
    /// Unique ID for this call, used for tracing
    pub trace_id: String,
    /// The actual model name used (config name or fallback)
    pub model_name: String,
    /// Source window title
    pub source_window: String,
    /// When the screenshot was captured (Unix timestamp)
    pub captured_at: i64,
    /// When the API call started (Unix timestamp)
    pub call_started_at: i64,
    /// How long the API call took (milliseconds)
    pub call_duration_ms: u64,
}

/// Model client for calling Ollama API
pub struct ModelClient {
    client: Client,
    config: Config,
}

impl ModelClient {
    /// Returns the model name to use for consolidation (text-only prompts).
    pub fn consolidation_model_name(&self) -> &str {
        self.config
            .consolidation_model_name
            .as_deref()
            .unwrap_or("llama3")
    }
}
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    images: Vec<String>,
    stream: bool,
}

/// Response from Ollama API
#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl ModelClient {
    /// Create a new ModelClient with the given configuration
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");
        ModelClient { client, config }
    }

    /// Call the model with a window snapshot - returns raw output without parsing
    pub fn call_raw(
        &self,
        snapshot: &WindowSnapshot,
        process_context: &crate::window_monitor::ProcessContext,
    ) -> Result<ModelRawOutput, String> {
        self.call_raw_with_context(snapshot, process_context, None)
    }

    /// Call the model with a window snapshot and compact local context.
    pub fn call_raw_with_context(
        &self,
        snapshot: &WindowSnapshot,
        process_context: &crate::window_monitor::ProcessContext,
        popup_context: Option<&str>,
    ) -> Result<ModelRawOutput, String> {
        let prompt = self.build_prompt_with_context(snapshot, Some(process_context), popup_context);
        let call_started_at = chrono::Utc::now().timestamp();
        let (response_text, model_name, call_duration_ms) =
            self.send_request_timed(&prompt, &snapshot.screenshot_base64)?;
        Ok(ModelRawOutput {
            raw_text: response_text,
            trace_id: crate::types::TraceId::new().0,
            model_name,
            source_window: snapshot.window_info.title.clone(),
            captured_at: snapshot.timestamp,
            call_started_at,
            call_duration_ms,
        })
    }

    /// Call the model with a window snapshot (legacy test helper - returns parsed ModelOutput).
    #[cfg(test)]
    pub fn call(
        &self,
        snapshot: &WindowSnapshot,
        process_context: &crate::window_monitor::ProcessContext,
    ) -> Result<ModelOutput, String> {
        let prompt = self.build_prompt_with_context(snapshot, Some(process_context), None);
        let response = self.send_request(&prompt, &snapshot.screenshot_base64)?;
        self.parse_response(&response)
    }

    /// Build a prompt from the window snapshot
    fn build_prompt(&self, snapshot: &WindowSnapshot) -> String {
        self.build_prompt_with_context(snapshot, None, None)
    }

    fn build_prompt_with_context(
        &self,
        snapshot: &WindowSnapshot,
        process_context: Option<&crate::window_monitor::ProcessContext>,
        popup_context: Option<&str>,
    ) -> String {
        let window = &snapshot.window_info;
        let process_context_block = process_context
            .map(|context| {
                format!(
                    "process_context_facts: stay_duration_seconds={}, switches_last_minute={}",
                    context.stay_duration_seconds, context.switches_in_last_minute
                )
            })
            .unwrap_or_else(|| String::from("process_context_facts: unavailable"));
        let popup_context_block = popup_context
            .map(str::trim)
            .filter(|context| !context.is_empty())
            .unwrap_or("action_log_tail: unavailable");

        format!(
            r#"你是 Cozmio 的桌面观察助手。

你看到的是用户当前屏幕的一小段现场。
你的输出会被原样交给桌面端展示。
Cozmio 只提供事实材料和工具材料，不提供结论。

请只把下面的系统材料当作事实输入，不要把它们当成用户意图、任务阶段或项目结论。
是否出现、说什么、说多少、是否接入工作流，都由你基于截图和事实材料自行判断。
不要为了迎合上下文而编造屏幕上或材料中没有出现的内容。

窗口标题: {}
进程名: {}

{}

local_context:
{}
"#,
            window.title, window.process_name, process_context_block, popup_context_block
        )
    }

    /// Send request to Ollama API for legacy parsed-output tests.
    #[cfg(test)]
    fn send_request(&self, prompt: &str, image_base64: &str) -> Result<String, String> {
        let url = format!("{}/api/generate", self.config.ollama_url);

        let request = OllamaRequest {
            model: self.config.model_name.clone(),
            prompt: prompt.to_string(),
            images: vec![image_base64.to_string()],
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                // Configured model not found (404) - try fallback to first available model
                if let Some(fallback_model) = Self::discover_first_model(&self.config.ollama_url) {
                    log::warn!(
                        "Model '{}' not found, falling back to '{}'",
                        self.config.model_name,
                        fallback_model
                    );
                    return self.send_request_with_model(prompt, image_base64, &fallback_model);
                }
            }
            return Err(format!("Ollama API error: {}", response.status()));
        }

        let ollama_resp: OllamaResponse = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(ollama_resp.response)
    }

    /// Send request and return (response_text, model_name, call_duration_ms)
    fn send_request_timed(
        &self,
        prompt: &str,
        image_base64: &str,
    ) -> Result<(String, String, u64), String> {
        let url = format!("{}/api/generate", self.config.ollama_url);

        let request = OllamaRequest {
            model: self.config.model_name.clone(),
            prompt: prompt.to_string(),
            images: vec![image_base64.to_string()],
            stream: false,
        };

        let start = std::time::Instant::now();
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        let call_duration_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                if let Some(fallback_model) = Self::discover_first_model(&self.config.ollama_url) {
                    log::warn!(
                        "Model '{}' not found, falling back to '{}'",
                        self.config.model_name,
                        fallback_model
                    );
                    let (text, _, duration) =
                        self.send_request_with_model_timed(prompt, image_base64, &fallback_model)?;
                    return Ok((text, fallback_model, duration));
                }
            }
            return Err(format!("Ollama API error: {}", response.status()));
        }

        let ollama_resp: OllamaResponse = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok((
            ollama_resp.response,
            self.config.model_name.clone(),
            call_duration_ms,
        ))
    }

    /// Send request with a specific model for legacy parsed-output tests.
    #[cfg(test)]
    fn send_request_with_model(
        &self,
        prompt: &str,
        image_base64: &str,
        model: &str,
    ) -> Result<String, String> {
        let url = format!("{}/api/generate", self.config.ollama_url);

        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            images: vec![image_base64.to_string()],
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Ollama API error (fallback): {}",
                response.status()
            ));
        }

        let ollama_resp: OllamaResponse = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(ollama_resp.response)
    }

    /// Send request with a specific model and return timing info
    fn send_request_with_model_timed(
        &self,
        prompt: &str,
        image_base64: &str,
        model: &str,
    ) -> Result<(String, String, u64), String> {
        let url = format!("{}/api/generate", self.config.ollama_url);

        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            images: vec![image_base64.to_string()],
            stream: false,
        };

        let start = std::time::Instant::now();
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        let call_duration_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            return Err(format!(
                "Ollama API error (fallback): {}",
                response.status()
            ));
        }

        let ollama_resp: OllamaResponse = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok((ollama_resp.response, model.to_string(), call_duration_ms))
    }

    /// Send text-only request (no image) and return (response_text, model_name, call_duration_ms)
    pub fn send_text_request_timed(
        &self,
        prompt: &str,
        model_name: &str,
    ) -> Result<(String, String, u64), String> {
        let url = format!("{}/api/generate", self.config.ollama_url);

        let request = OllamaRequest {
            model: model_name.to_string(),
            prompt: prompt.to_string(),
            images: vec![],
            stream: false,
        };

        let start = std::time::Instant::now();
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        let call_duration_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                if let Some(fallback_model) = Self::discover_first_model(&self.config.ollama_url) {
                    log::warn!(
                        "Model '{}' not found, falling back to '{}'",
                        model_name,
                        fallback_model
                    );
                    return self.send_text_request_timed(prompt, &fallback_model);
                }
            }
            return Err(format!("Ollama API error: {}", response.status()));
        }

        let ollama_resp: OllamaResponse = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok((ollama_resp.response, model_name.to_string(), call_duration_ms))
    }

    /// Call consolidation model with text-only prompt, returning raw text response
    pub fn call_consolidation_model(&self, prompt: &str) -> Result<(String, String, u64), String> {
        self.send_text_request_timed(prompt, self.consolidation_model_name())
    }

    /// Discover the first available model from Ollama
    fn discover_first_model(ollama_url: &str) -> Option<String> {
        let client = match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
        {
            Ok(c) => c,
            Err(_) => return None,
        };

        let url = format!("{}/api/tags", ollama_url);
        let response = match client.get(&url).send() {
            Ok(r) => r,
            Err(_) => return None,
        };

        if !response.status().is_success() {
            return None;
        }

        #[derive(serde::Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }
        #[derive(serde::Deserialize)]
        struct ModelInfo {
            name: String,
        }

        match response.json::<TagsResponse>() {
            Ok(tags) => tags.models.into_iter().next().map(|m| m.name),
            Err(_) => None,
        }
    }

    /// Parse the model response into a legacy ModelOutput struct.
    #[cfg(test)]
    fn parse_response(&self, response: &str) -> Result<ModelOutput, String> {
        let trimmed = response.trim();

        // Parse MODE line
        let mode_re = Regex::new(r"(?i)MODE:\s*(CONTINUE|ABSTAIN)").unwrap();
        let reason_re = Regex::new(r"(?i)REASON:\s*(.+)").unwrap();
        let user_how_re = Regex::new(r"(?i)USER_HOW:\s*(.+)").unwrap();

        let mode_cap = mode_re
            .captures(trimmed)
            .ok_or_else(|| "Missing MODE line - model output format error".to_string())?;
        let mode_str = mode_cap.get(1).unwrap().as_str().to_uppercase();
        let mode = if mode_str == "CONTINUE" {
            InterventionMode::Continue
        } else {
            InterventionMode::Abstain
        };

        let reason = reason_re
            .captures(trimmed)
            .map(|c| c.get(1).unwrap().as_str().trim().to_string())
            .ok_or_else(|| "Missing REASON line".to_string())?;

        let user_how = if mode == InterventionMode::Continue {
            user_how_re
                .captures(trimmed)
                .map(|c| c.get(1).unwrap().as_str().trim().to_string())
        } else {
            None
        };

        Ok(ModelOutput {
            mode,
            reason,
            user_how,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cozmio_core::WindowInfo;

    fn create_test_snapshot() -> WindowSnapshot {
        WindowSnapshot {
            screenshot_base64: "TEST_BASE64_IMAGE".to_string(),
            screenshot_width: 1920,
            screenshot_height: 1080,
            window_info: WindowInfo {
                hwnd: 1,
                title: "Test Window".to_string(),
                process_name: "test.exe".to_string(),
                process_id: 1234,
                monitor_index: 1,
                rect: cozmio_core::Rect {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                is_foreground: true,
                is_visible: true,
                z_order: 0,
            },
            timestamp: 1234567890,
        }
    }

    fn create_test_config() -> Config {
        Config {
            ollama_url: "http://localhost:11434".to_string(),
            model_name: "llava".to_string(),
            poll_interval_secs: 3,
            window_change_detection: true,
            execute_auto: true,
            request_use_native_dialog: true,
            execute_delay_secs: 1,
            memory_flywheel_enabled: true,
            consolidation_model_name: None,
            last_check_at: None,
            update_channel: "stable".to_string(),
            memory_maintenance_mode: "local".to_string(),
        }
    }

    #[test]
    fn test_model_client_new() {
        let config = create_test_config();
        let client = ModelClient::new(config.clone());
        assert_eq!(client.config.ollama_url, "http://localhost:11434");
        assert_eq!(client.config.model_name, "llava");
    }

    #[test]
    fn test_build_prompt() {
        let config = create_test_config();
        let client = ModelClient::new(config);
        let snapshot = create_test_snapshot();

        let prompt = client.build_prompt(&snapshot);

        assert!(prompt.contains("Test Window"));
        assert!(prompt.contains("test.exe"));
        // New prompt uses natural language role description, not structured CONTINUE/ABSTAIN format
        assert!(prompt.contains("桌面观察助手"));
        assert!(prompt.contains("窗口标题"));
        assert!(prompt.contains("进程名"));
    }

    #[test]
    fn test_parse_response_continue() {
        let config = create_test_config();
        let client = ModelClient::new(config);

        let response = r#"MODE: CONTINUE
REASON: 截图中明确展示协作框架定义讨论的具体建议，包含明确的决策路径和可操作选项。
USER_HOW: 帮助用户梳理框架定义的具体建议内容，提供决策路径建议。"#;

        let output = client.parse_response(response).unwrap();
        assert_eq!(output.mode, InterventionMode::Continue);
        assert!(output.reason.contains("协作框架"));
        assert!(output.user_how.is_some());
        assert!(output.user_how.unwrap().contains("帮助用户"));
    }

    #[test]
    fn test_parse_response_abstain() {
        let config = create_test_config();
        let client = ModelClient::new(config);

        let response = r#"MODE: ABSTAIN
REASON: 当前证据不足，无法支持继续介入。"#;

        let output = client.parse_response(response).unwrap();
        assert_eq!(output.mode, InterventionMode::Abstain);
    }

    #[test]
    fn test_parse_response_missing_mode() {
        let config = create_test_config();
        let client = ModelClient::new(config);

        let response = r#"REASON: some reason"#;
        let result = client.parse_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing MODE"));
    }

    #[test]
    fn test_parse_response_missing_reason() {
        let config = create_test_config();
        let client = ModelClient::new(config);

        let response = r#"MODE: CONTINUE"#;
        let result = client.parse_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing REASON"));
    }

    #[test]
    fn test_intervention_mode_display() {
        assert_eq!(InterventionMode::Continue.to_string(), "CONTINUE");
        assert_eq!(InterventionMode::Abstain.to_string(), "ABSTAIN");
    }

    #[test]
    fn test_parse_response_whitespace_only_mode() {
        let config = create_test_config();
        let client = ModelClient::new(config);
        let response = "MODE:   CONTINUE\nREASON: Test reason";
        let output = client.parse_response(response).unwrap();
        assert_eq!(output.mode, InterventionMode::Continue);
    }

    #[test]
    fn test_parse_response_lowercase_mode() {
        let config = create_test_config();
        let client = ModelClient::new(config);
        let response = "mode: abstain\nreason: Test reason";
        let output = client.parse_response(response).unwrap();
        assert_eq!(output.mode, InterventionMode::Abstain);
    }

    #[test]
    fn test_parse_response_extra_newlines() {
        let config = create_test_config();
        let client = ModelClient::new(config);
        let response = "MODE: CONTINUE\n\n\nREASON: Test reason\n\n";
        let output = client.parse_response(response).unwrap();
        assert_eq!(output.mode, InterventionMode::Continue);
    }
}
