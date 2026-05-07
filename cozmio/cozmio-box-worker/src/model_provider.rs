use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Model not loaded")]
    NotLoaded,
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    #[error("Timeout")]
    Timeout,
    #[error("Provider not ready: {0}")]
    NotReady(String),
    #[error("Shutdown failed: {0}")]
    ShutdownFailed(String),
}

/// Provider operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    Uninitialized,
    Loading,
    Ready,
    Busy,
    Error,
    Shutdown,
}

impl ProviderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderStatus::Uninitialized => "uninitialized",
            ProviderStatus::Loading => "loading",
            ProviderStatus::Ready => "ready",
            ProviderStatus::Busy => "busy",
            ProviderStatus::Error => "error",
            ProviderStatus::Shutdown => "shutdown",
        }
    }
}

/// Trait for model inference providers
pub trait ModelProvider: Send + Sync {
    /// Generate output from the model
    /// Returns (output_text, duration_ms)
    fn generate(
        &self,
        prompt: &str,
        image_data: Option<&[u8]>,
    ) -> Result<(String, u64), ProviderError>;

    /// Get current provider status
    fn status(&self) -> ProviderStatus;

    /// Warmup the model (prepare for inference)
    fn warmup(&mut self) -> Result<(), ProviderError>;

    /// Shutdown the model and release resources
    fn shutdown(&mut self) -> Result<(), ProviderError>;

    /// Get provider name
    fn name(&self) -> &str;
}
