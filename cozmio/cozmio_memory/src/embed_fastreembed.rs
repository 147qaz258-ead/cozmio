#[cfg(feature = "fastembed")]
use std::sync::RwLock;

use crate::error::MemoryError;

#[cfg(feature = "fastembed")]
use fastembed::TextEmbedding;

pub struct FastEmbedProvider {
    #[cfg(feature = "fastembed")]
    model: Option<RwLock<TextEmbedding>>,
    dimension: usize,
    available: bool,
}

impl FastEmbedProvider {
    pub fn new() -> Result<Self, MemoryError> {
        #[cfg(feature = "fastembed")]
        {
            match TextEmbedding::try_new(Default::default()) {
                Ok(model) => {
                    return Ok(Self {
                        model: Some(RwLock::new(model)),
                        dimension: 384,
                        available: true,
                    });
                }
                Err(e) => {
                    log::warn!("FastEmbed initialization failed: {}", e);
                    return Ok(Self {
                        model: None,
                        dimension: 384,
                        available: false,
                    });
                }
            }
        }

        #[cfg(not(feature = "fastembed"))]
        {
            Ok(Self {
                dimension: 384,
                available: false,
            })
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.available
    }
}

impl crate::embed_provider::EmbeddingProvider for FastEmbedProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError> {
        if !self.available {
            return Err(MemoryError::EmbeddingDisabled);
        }

        #[cfg(feature = "fastembed")]
        {
            if let Some(ref model_lock) = self.model {
                let mut model = model_lock
                    .write()
                    .map_err(|_| MemoryError::EmbeddingDisabled)?;
                let embeddings = model
                    .embed([text].as_slice(), None)
                    .map_err(|_e| MemoryError::EmbeddingDisabled)?;
                if let Some(emb) = embeddings.first() {
                    return Ok(emb.clone());
                }
            }
            return Err(MemoryError::EmbeddingDisabled);
        }

        #[cfg(not(feature = "fastembed"))]
        {
            let _ = text;
            Err(MemoryError::EmbeddingDisabled)
        }
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn is_available(&self) -> bool {
        self.available
    }
}
