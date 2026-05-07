use crate::error::MemoryError;

pub struct DisabledProvider;

impl crate::embed_provider::EmbeddingProvider for DisabledProvider {
    fn embed(&self, _text: &str) -> Result<Vec<f32>, MemoryError> {
        Err(MemoryError::EmbeddingDisabled)
    }

    fn dimension(&self) -> usize {
        384
    }

    fn is_available(&self) -> bool {
        false
    }
}
