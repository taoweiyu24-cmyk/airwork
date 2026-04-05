use serde::{Deserialize, Serialize};
use wif_domain::AiProfile;

use crate::error::AiError;

/// A single message in the chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Token usage statistics returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// The result of a chat completion request.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub usage: Option<Usage>,
}

// ── Internal serde types for API wire format ──────────────────────────────────

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct ApiResponse {
    choices: Vec<ApiChoice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct ApiChoice {
    message: ApiMessage,
}

#[derive(Deserialize)]
struct ApiMessage {
    content: String,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// HTTP client for any OpenAI-compatible chat completions endpoint.
pub struct OpenAiCompatibleClient {
    base_url: String,
    api_key: String,
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    http: reqwest::Client,
}

impl OpenAiCompatibleClient {
    /// Construct a client from an [`AiProfile`].
    pub fn new(profile: &AiProfile) -> Self {
        Self {
            base_url: profile.base_url.trim_end_matches('/').to_string(),
            api_key: profile.api_key.clone(),
            model: profile.model.clone(),
            max_tokens: profile.max_tokens,
            temperature: profile.temperature,
            http: reqwest::Client::new(),
        }
    }

    /// Send a list of messages and return the assistant's reply.
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse, AiError> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let body = ChatRequest {
            model: &self.model,
            messages: &messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AiError::RequestFailed(format!("HTTP {status}: {text}")));
        }

        let api_resp: ApiResponse = response
            .json()
            .await
            .map_err(|e| AiError::RequestFailed(format!("Failed to parse response: {e}")))?;

        let content = api_resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(ChatResponse {
            content,
            usage: api_resp.usage,
        })
    }
}
