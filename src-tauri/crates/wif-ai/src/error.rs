/// AI-specific errors for the wif-ai crate.
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("AI provider not configured")]
    NoProvider,

    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Content blocked by egress policy: {0}")]
    EgressBlocked(String),

    #[error("Token budget exceeded: used {used}, max {max}")]
    TokenBudgetExceeded { used: u32, max: u32 },

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
