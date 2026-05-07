use crate::model_provider::{ModelProvider, ProviderError, ProviderStatus};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::{Duration, Instant};
use uuid::Uuid;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

pub struct LlamaCppProvider {
    server_url: String,
    model_path: String,
    status: ProviderStatus,
    client: Client,
}

impl LlamaCppProvider {
    pub fn new(server_url: &str, model_path: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            model_path: model_path.to_string(),
            status: ProviderStatus::Uninitialized,
            client: Client::builder()
                .timeout(Duration::from_secs(1800))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub fn from_config(config: &LlamaCppConfig) -> Self {
        Self::new(&config.server_url, &config.model_path)
    }

    fn chat_url(&self) -> String {
        format!("{}/v1/chat/completions", self.server_url)
    }

    /// Performs intelligent resizing to preserve text/details while staying within token budget.
    fn smart_resize(&self, raw_bytes: &[u8]) -> Result<Vec<u8>, ProviderError> {
        let img = image::load_from_memory(raw_bytes).map_err(|e| {
            ProviderError::InferenceFailed(format!("Failed to load image for resizing: {}", e))
        })?;

        let (width, height) = img.dimensions();
        let aspect_ratio = width as f64 / height as f64;

        // Target: ~768 tokens (Qwen-VL uses 28x28 patches usually)
        // 768 * 28 * 28 = ~602,112 pixels
        let target_pixels = 602_112.0;
        
        let mut new_width = (target_pixels * aspect_ratio).sqrt();
        let mut new_height = target_pixels / new_width;

        // Align to 28px patch boundaries
        new_width = (new_width / 28.0).round() * 28.0;
        new_height = (new_height / 28.0).round() * 28.0;

        // Ensure minimum 28x28
        let final_w = (new_width as u32).max(28);
        let final_h = (new_height as u32).max(28);

        log::info!(
            "SmartResize: {}x{} -> {}x{} (ratio={:.2})",
            width, height, final_w, final_h, aspect_ratio
        );

        let resized = img.resize_exact(final_w, final_h, image::imageops::FilterType::Lanczos3);

        let mut buf = Cursor::new(Vec::new());
        resized.write_to(&mut buf, ImageFormat::Png).map_err(|e| {
            ProviderError::InferenceFailed(format!("Failed to encode resized image: {}", e))
        })?;

        Ok(buf.into_inner())
    }

    fn generate_internal(
        &self,
        prompt: &str,
        image_data: Option<&[u8]>,
    ) -> Result<(String, u64), ProviderError> {
        if self.status != ProviderStatus::Ready {
            return Err(ProviderError::NotReady(format!(
                "Provider not ready: {:?}",
                self.status
            )));
        }

        let trace_id = Uuid::new_v4();
        let start = Instant::now();

        log::info!(
            "Box inference: generating response [trace_id={}, provider=llama_cpp, model_path={}, has_image={}]",
            trace_id,
            self.model_path,
            image_data.is_some()
        );

        let b64 = if let Some(data) = image_data {
            let processed_data = self.smart_resize(data)?;
            use base64::{engine::general_purpose, Engine as _};
            Some(general_purpose::STANDARD.encode(processed_data))
        } else {
            None
        };

        let mut content = Vec::new();
        if let Some(img_b64) = b64 {
            content.push(serde_json::json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/png;base64,{}", img_b64)
                }
            }));
        }
        content.push(serde_json::json!({
            "type": "text",
            "text": prompt
        }));

        let response = self
            .client
            .post(self.chat_url())
            .json(&serde_json::json!({
                "messages": [
                    {
                        "role": "user",
                        "content": content
                    }
                ],
                "stream": false
            }))
            .send()
            .map_err(|e| ProviderError::InferenceFailed(format!("HTTP request failed: {}", e)))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            log::error!(
                "Box inference: HTTP error [trace_id={}, status={}, body={}]",
                trace_id,
                status,
                body
            );
            return Err(ProviderError::InferenceFailed(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let response_text = response.text().map_err(|e| {
            ProviderError::InferenceFailed(format!("Failed to get response text: {}", e))
        })?;

        log::info!("LlamaCppProvider Raw Response: {}", response_text);

        let chat_resp: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
            ProviderError::InferenceFailed(format!(
                "Failed to parse response ({}): {}",
                response_text, e
            ))
        })?;

        let content = chat_resp["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let output_chars = content.chars().count();

        log::info!(
            "Box inference: completed [trace_id={}, duration={}ms, output_chars={}]",
            trace_id,
            duration_ms,
            output_chars
        );

        Ok((content, duration_ms))
    }
}

#[derive(Debug, Clone)]
pub struct LlamaCppConfig {
    pub server_url: String,
    pub model_path: String,
    pub context_size: u32,
    pub threads: u32,
    pub timeout_secs: u64,
}

impl Default for LlamaCppConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8080".to_string(),
            model_path: "/opt/cozmio/models/current.gguf".to_string(),
            context_size: 2048,
            threads: 4,
            timeout_secs: 120,
        }
    }
}

impl ModelProvider for LlamaCppProvider {
    fn name(&self) -> &'static str {
        "llama_cpp"
    }

    fn generate(
        &self,
        prompt: &str,
        image_data: Option<&[u8]>,
    ) -> Result<(String, u64), ProviderError> {
        self.generate_internal(prompt, image_data)
    }

    fn status(&self) -> ProviderStatus {
        self.status
    }

    fn warmup(&mut self) -> Result<(), ProviderError> {
        log::info!(
            "LlamaCppProvider: warming up (server_url={}, model_path={})...",
            self.server_url,
            self.model_path
        );

        self.status = ProviderStatus::Loading;

        let health_url = format!("{}/health", self.server_url);
        match self.client.get(&health_url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    log::info!("LlamaCppProvider: llama-server is healthy");
                } else {
                    log::warn!(
                        "LlamaCppProvider: llama-server returned {}",
                        response.status()
                    );
                }
            }
            Err(e) => {
                log::warn!("LlamaCppProvider: llama-server not reachable: {}", e);
            }
        }

        self.status = ProviderStatus::Ready;
        log::info!("LlamaCppProvider: warmup complete, status=Ready");
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ProviderError> {
        log::info!("LlamaCppProvider: shutting down...");
        self.status = ProviderStatus::Shutdown;
        log::info!("LlamaCppProvider: shutdown complete");
        Ok(())
    }
}
