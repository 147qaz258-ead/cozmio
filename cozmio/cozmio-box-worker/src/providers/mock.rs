use crate::model_provider::{ModelProvider, ProviderError, ProviderStatus};

/// Mock model provider that returns fixed responses (for development/testing)
pub struct MockProvider {
    status: ProviderStatus,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            status: ProviderStatus::Uninitialized,
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelProvider for MockProvider {
    fn generate(
        &self,
        prompt: &str,
        image_data: Option<&[u8]>,
    ) -> Result<(String, u64), ProviderError> {
        if self.status != ProviderStatus::Ready {
            return Err(ProviderError::NotReady(format!(
                "Mock provider not ready: {:?}",
                self.status
            )));
        }

        let start = std::time::Instant::now();

        log::info!(
            "Box inference: generating mock response [provider=mock, prompt_chars={}]",
            prompt.chars().count()
        );

        // Simulate inference time
        std::thread::sleep(std::time::Duration::from_millis(50));

        let response = "This is a mock response from cozmio-box-worker. Real GGUF model inference will be available in Phase 4.".to_string();
        let duration_ms = start.elapsed().as_millis() as u64;

        log::info!(
            "Box inference: mock completed [duration={}ms, output_chars={}]",
            duration_ms,
            response.chars().count()
        );

        Ok((response, duration_ms))
    }

    fn status(&self) -> ProviderStatus {
        self.status
    }

    fn warmup(&mut self) -> Result<(), ProviderError> {
        log::info!("[MockProvider] Warming up (mock)...");
        // Simulate loading time
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.status = ProviderStatus::Ready;
        log::info!("[MockProvider] Warmup complete, status=Ready");
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ProviderError> {
        log::info!("[MockProvider] Shutting down (mock)...");
        self.status = ProviderStatus::Shutdown;
        Ok(())
    }

    fn name(&self) -> &str {
        "mock"
    }
}
