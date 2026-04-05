use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// An AI provider configuration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProfile {
    pub id: Ulid,
    pub name: String,
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub is_default: bool,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}
