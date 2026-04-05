use std::collections::HashMap;

use wif_domain::{AiProfile, AnalysisType, EgressPolicy, WorkItem};

use crate::{
    client::{ChatMessage, ChatResponse, OpenAiCompatibleClient},
    egress::EgressFilter,
    error::AiError,
    prompt_loader::{load_template, render},
};

/// Orchestrates AI analysis by combining the HTTP client, prompt loading, and
/// egress-policy enforcement.
pub struct AiService {
    client: Option<OpenAiCompatibleClient>,
    egress: EgressFilter,
}

impl AiService {
    /// Build a service.
    ///
    /// Pass `None` for `profile` when no AI provider has been configured yet;
    /// any call to [`analyze`](Self::analyze) will then return
    /// [`AiError::NoProvider`].
    pub fn new(profile: Option<&AiProfile>, policy: EgressPolicy) -> Self {
        Self {
            client: profile.map(OpenAiCompatibleClient::new),
            egress: EgressFilter::new(policy),
        }
    }

    /// Run an AI analysis on a work item.
    ///
    /// Steps:
    /// 1. Ensure a provider is configured.
    /// 2. Load and render the prompt template.
    /// 3. Enforce egress policy on the rendered content.
    /// 4. Send to the LLM and return the assistant's reply.
    pub async fn analyze(
        &self,
        work_item: &WorkItem,
        analysis_type: AnalysisType,
    ) -> Result<String, AiError> {
        let client = self.client.as_ref().ok_or(AiError::NoProvider)?;

        let template = load_template(&analysis_type);

        let mut vars = HashMap::new();
        vars.insert("title".to_string(), work_item.title.clone());
        vars.insert(
            "content".to_string(),
            work_item.content.clone().unwrap_or_default(),
        );
        vars.insert("source".to_string(), format!("{:?}", work_item.source));

        let user_msg = render(&template.user_prompt, &vars);

        // Enforce egress policy on the full outbound payload.
        let checked_user_msg = self.egress.check(&user_msg, "text/plain")?;

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: template.system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: checked_user_msg,
            },
        ];

        let ChatResponse { content, .. } = client.chat(messages).await?;

        Ok(content)
    }
}
