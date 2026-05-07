use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Embedding disabled or unavailable")]
    EmbeddingDisabled,

    #[error("Insufficient imported data for reminder context")]
    InsufficientImportedData,

    #[error("Import error: {0}")]
    Import(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
