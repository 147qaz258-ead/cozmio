// cozmio/cozmio_model/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Ollama API 调用失败: {0}")]
    ApiError(String),

    #[error("网络请求失败: {0}")]
    NetworkError(String),

    #[error("响应解析失败: {0}")]
    ParseError(String),

    #[error("模型不存在: {0}")]
    ModelNotFound(String),
}
