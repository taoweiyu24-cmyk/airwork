use serde::{Deserialize, Serialize};

/// Policy controlling what content may egress from the system (e.g. to an AI provider).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EgressPolicy {
    /// 0 = public, higher values = more sensitive.
    pub sensitivity_level: u8,
    /// Maximum number of tokens that may be sent in a single request.
    pub max_tokens: u32,
    /// MIME types (or application-defined labels) that are never allowed to egress.
    pub blocked_content_types: Vec<String>,
}

impl EgressPolicy {
    /// Returns `true` when the given content type is not in the blocked list.
    pub fn allows_content_type(&self, ct: &str) -> bool {
        !self.blocked_content_types.iter().any(|blocked| blocked == ct)
    }

    /// Returns `true` when the token count is within the allowed budget.
    pub fn is_within_budget(&self, tokens: u32) -> bool {
        tokens <= self.max_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> EgressPolicy {
        EgressPolicy {
            sensitivity_level: 1,
            max_tokens: 1000,
            blocked_content_types: vec!["application/pdf".to_string()],
        }
    }

    #[test]
    fn allows_unblocked_content_type() {
        assert!(policy().allows_content_type("text/plain"));
    }

    #[test]
    fn rejects_blocked_content_type() {
        assert!(!policy().allows_content_type("application/pdf"));
    }

    #[test]
    fn within_budget_when_equal() {
        assert!(policy().is_within_budget(1000));
    }

    #[test]
    fn exceeds_budget_when_over() {
        assert!(!policy().is_within_budget(1001));
    }
}
