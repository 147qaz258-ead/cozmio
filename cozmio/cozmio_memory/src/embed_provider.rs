use std::sync::Arc;

use crate::error::MemoryError;

pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError>;
    fn dimension(&self) -> usize;
    fn is_available(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    FastEmbed,
    Mock,
    Disabled,
}

/// Try to create a provider in priority order:
/// 1. FastEmbed (if available and compiles)
/// 2. Mock (for testing)
/// 3. Disabled (if nothing else works)
pub fn create_provider(
    provider_type: ProviderType,
) -> Result<Arc<dyn EmbeddingProvider>, MemoryError> {
    match provider_type {
        ProviderType::FastEmbed => {
            let fp = crate::embed_fastreembed::FastEmbedProvider::new()?;
            if fp.is_available() {
                Ok(Arc::new(fp))
            } else {
                Ok(Arc::new(crate::embed_disabled::DisabledProvider) as Arc<dyn EmbeddingProvider>)
            }
        }
        ProviderType::Mock => Ok(Arc::new(crate::embed_mock::MockProvider::new(384))),
        ProviderType::Disabled => Ok(Arc::new(crate::embed_disabled::DisabledProvider)),
    }
}
