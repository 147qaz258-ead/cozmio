use crate::error::MemoryError;

pub struct MockProvider {
    dimension: usize,
}

impl MockProvider {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl crate::embed_provider::EmbeddingProvider for MockProvider {
    fn embed(&self, _text: &str) -> Result<Vec<f32>, MemoryError> {
        Ok(vec![0.0; self.dimension])
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn is_available(&self) -> bool {
        true
    }
}
