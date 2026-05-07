use crate::model_provider::{ModelProvider, ProviderError};
use crate::providers::{LlamaCppProvider, MockProvider};
use crate::status::ModelRuntimeStatus;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Box model runtime configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub model_path: Option<String>,
    pub server_url: Option<String>,
    pub context_size: Option<u32>,
    pub threads: Option<u32>,
    pub timeout_secs: Option<u64>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "mock".to_string(),
            model_path: None,
            server_url: None,
            context_size: None,
            threads: None,
            timeout_secs: None,
        }
    }
}

impl ModelConfig {
    /// Load model config from box-model.toml
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            log::warn!(
                "Model config not found at {}, using defaults",
                config_path.display()
            );
            return Ok(ModelConfig::default());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: ModelConfig = toml::from_str(&content)?;

        log::info!(
            "Loaded model config: provider={}, model_path={:?}",
            config.provider,
            config.model_path
        );

        Ok(config)
    }

    fn config_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            Ok(Path::new("/opt/cozmio/config/box-model.toml").to_path_buf())
        }
        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA")?;
            Ok(Path::new(&local_app_data)
                .join("cozmio")
                .join("config")
                .join("box-model.toml"))
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err("Unsupported platform".into())
        }
    }
}

/// Box model runtime that manages the model provider
pub struct BoxModelRuntime {
    provider: Box<dyn ModelProvider>,
    status: Arc<RwLock<ModelRuntimeStatus>>,
}

impl BoxModelRuntime {
    pub fn new() -> Self {
        Self::with_provider_name("mock")
    }

    /// Create runtime with specific provider name (for testing)
    #[allow(dead_code)]
    pub fn with_provider_name(_provider_name: &str) -> Self {
        let config = ModelConfig::load().unwrap_or_default();
        Self::with_config(&config)
    }

    /// Create runtime from configuration
    pub fn with_config(config: &ModelConfig) -> Self {
        let provider: Box<dyn ModelProvider> = match config.provider.as_str() {
            "llama_cpp" => {
                let server_url = config
                    .server_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:8080".to_string());
                let model_path = config
                    .model_path
                    .clone()
                    .unwrap_or_else(|| "/opt/cozmio/models/current.gguf".to_string());

                log::info!(
                    "BoxModelRuntime initializing with llama_cpp provider (server_url={}, model_path={})",
                    server_url,
                    model_path
                );

                Box::new(LlamaCppProvider::new(&server_url, &model_path))
            }
            "mock" | _ => {
                log::info!("BoxModelRuntime initializing with mock provider");
                Box::new(MockProvider::new())
            }
        };

        Self {
            provider,
            status: Arc::new(RwLock::new(ModelRuntimeStatus::Uninitialized)),
        }
    }

    /// Load/warmup the model
    pub fn load(&mut self) -> Result<(), ProviderError> {
        {
            let mut s = self.status.write().unwrap();
            *s = ModelRuntimeStatus::Loading;
        }

        log::info!(
            "BoxModelRuntime: Loading model (provider={})...",
            self.provider.name()
        );

        self.provider.warmup()?;

        {
            let mut s = self.status.write().unwrap();
            *s = ModelRuntimeStatus::Ready;
        }

        log::info!("BoxModelRuntime: Model ready");
        Ok(())
    }

    /// Run inference
    /// Returns (output_text, duration_ms)
    pub fn infer(
        &self,
        context: &str,
        image_data: Option<&[u8]>,
        timeout_secs: u64,
    ) -> Result<(String, u64), ProviderError> {
        {
            let s = self.status.read().unwrap();
            if *s != ModelRuntimeStatus::Ready {
                return Err(ProviderError::NotLoaded);
            }
        }

        log::info!(
            "BoxModelRuntime: Running inference (provider={}, timeout={}s, context_chars={})",
            self.provider.name(),
            timeout_secs,
            context.chars().count()
        );

        // Note: timeout_secs parameter is ignored for llama_cpp provider
        // since it uses its own internal timeout
        let result = self.provider.generate(context, image_data);

        match &result {
            Ok((output, duration_ms)) => {
                log::info!(
                    "BoxModelRuntime: Inference successful (provider={}, duration={}ms, output_chars={})",
                    self.provider.name(),
                    duration_ms,
                    output.chars().count()
                );
            }
            Err(e) => {
                log::error!("BoxModelRuntime: Inference failed: {}", e);
            }
        }

        result
    }

    /// Get current status
    pub fn status(&self) -> ModelRuntimeStatus {
        *self.status.read().unwrap()
    }

    /// Check if model is ready
    pub fn is_ready(&self) -> bool {
        self.status() == ModelRuntimeStatus::Ready
    }

    /// Shutdown the provider
    pub fn shutdown(&mut self) -> Result<(), ProviderError> {
        log::info!("BoxModelRuntime: Shutting down provider...");
        self.provider.shutdown()?;
        let mut s = self.status.write().unwrap();
        *s = ModelRuntimeStatus::Error; // Mark as error since we're shutting down
        Ok(())
    }
}

impl Default for BoxModelRuntime {
    fn default() -> Self {
        Self::new()
    }
}
