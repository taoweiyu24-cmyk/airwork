//! `wif-ai` — OpenAI-compatible LLM client with egress-policy enforcement.
//!
//! # Architecture
//!
//! - [`client`] — low-level HTTP client for any OpenAI-compatible endpoint.
//! - [`prompt_loader`] — template loading and `{{variable}}` substitution.
//! - [`egress`] — content-type blocking and token-budget enforcement.
//! - [`service`] — high-level orchestrator used by the application layer.
//! - [`error`] — crate-wide error type.

pub mod client;
pub mod egress;
pub mod error;
pub mod prompt_loader;
pub mod service;

// Flat convenience re-exports.
pub use client::{ChatMessage, ChatResponse, OpenAiCompatibleClient, Usage};
pub use egress::EgressFilter;
pub use error::AiError;
pub use prompt_loader::PromptTemplate;
pub use service::AiService;
