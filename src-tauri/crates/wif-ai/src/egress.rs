use wif_domain::EgressPolicy;

use crate::error::AiError;

/// Enforces an [`EgressPolicy`] before content is sent to an AI provider.
pub struct EgressFilter {
    policy: EgressPolicy,
}

impl EgressFilter {
    /// Create a new filter wrapping `policy`.
    pub fn new(policy: EgressPolicy) -> Self {
        Self { policy }
    }

    /// Validate `content` against the policy.
    ///
    /// Returns the (possibly truncated) content if allowed, or an [`AiError`]
    /// if the content type is blocked or the token budget is exceeded.
    pub fn check(&self, content: &str, content_type: &str) -> Result<String, AiError> {
        if !self.policy.allows_content_type(content_type) {
            return Err(AiError::EgressBlocked(format!(
                "content type '{content_type}' is not allowed by egress policy"
            )));
        }

        let estimated = Self::estimate_tokens(content);
        if !self.policy.is_within_budget(estimated) {
            return Err(AiError::TokenBudgetExceeded {
                used: estimated,
                max: self.policy.max_tokens,
            });
        }

        Ok(content.to_string())
    }

    /// Rough token estimate: one token per four characters.
    pub fn estimate_tokens(text: &str) -> u32 {
        (text.len() / 4) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter() -> EgressFilter {
        EgressFilter::new(EgressPolicy {
            sensitivity_level: 0,
            max_tokens: 100,
            blocked_content_types: vec!["application/pdf".to_string()],
        })
    }

    #[test]
    fn allows_text_plain() {
        let result = filter().check("hello world", "text/plain");
        assert!(result.is_ok());
    }

    #[test]
    fn blocks_pdf_content_type() {
        let result = filter().check("data", "application/pdf");
        assert!(matches!(result, Err(AiError::EgressBlocked(_))));
    }

    #[test]
    fn blocks_when_over_budget() {
        // 401 chars → 100 estimated tokens, which equals max — so use 404 chars for 101
        let long_text = "a".repeat(404);
        let result = filter().check(&long_text, "text/plain");
        assert!(matches!(
            result,
            Err(AiError::TokenBudgetExceeded { .. })
        ));
    }

    #[test]
    fn allows_exactly_at_budget() {
        // 400 chars → exactly 100 tokens
        let text = "a".repeat(400);
        let result = filter().check(&text, "text/plain");
        assert!(result.is_ok());
    }
}
