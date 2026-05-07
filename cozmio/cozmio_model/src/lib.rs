// cozmio/cozmio_model/src/lib.rs
pub mod client;
pub mod error;

pub use client::{
    ask_model, ask_model_sync, discover_first_model, discover_first_model_sync,
    parse_intervention_result, InterventionMode, InterventionResult,
};
pub use error::ModelError;
