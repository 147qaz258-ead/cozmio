//! In-memory vector store as fallback when sqlite-vec is unavailable.
//! Satisfies the same interface contract as the sqlite-vec implementation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VecStore {
    pub event_id: i64,
    pub embedding: Vec<f32>,
}

pub struct InMemoryVecStore {
    vectors: Vec<VecStore>,
    dimension: usize,
}

impl InMemoryVecStore {
    pub fn new(dimension: usize) -> Self {
        Self {
            vectors: Vec::new(),
            dimension,
        }
    }

    pub fn insert(&mut self, event_id: i64, embedding: Vec<f32>) {
        debug_assert_eq!(embedding.len(), self.dimension);
        self.vectors.push(VecStore {
            event_id,
            embedding,
        });
    }

    pub fn search(&self, query: &[f32], limit: usize) -> Vec<(i64, f32)> {
        let mut results: Vec<(i64, f32)> = self
            .vectors
            .iter()
            .map(|v| {
                let dist = self.cosine_distance(&v.embedding, query);
                (v.event_id, dist)
            })
            .collect();
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }

    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        1.0 - dot / (norm_a * norm_b + 1e-8)
    }

    pub fn clear(&mut self) {
        self.vectors.clear();
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }
}
