//! OAuth2 provider configurations and token refresh for Google and Microsoft.

use serde::Deserialize;

use crate::error::MailError;

/// Supported OAuth2 mail providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
    Microsoft,
}

/// OAuth2 configuration for a provider.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    /// Google OAuth2 configuration for Gmail IMAP/SMTP access.
    pub fn google() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token_url: "https://oauth2.googleapis.com/token".into(),
            redirect_uri: "urn:ietf:wg:oauth:2.0:oob".into(),
            scopes: vec![
                "https://mail.google.com/".into(),
                "https://www.googleapis.com/auth/gmail.send".into(),
            ],
        }
    }

    /// Microsoft OAuth2 configuration for Outlook / Exchange Online IMAP/SMTP.
    pub fn microsoft() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".into(),
            token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".into(),
            redirect_uri: "https://login.microsoftonline.com/common/oauth2/nativeclient".into(),
            scopes: vec![
                "https://outlook.office.com/IMAP.AccessAsUser.All".into(),
                "https://outlook.office.com/SMTP.Send".into(),
                "offline_access".into(),
            ],
        }
    }
}

/// Response payload returned from a token endpoint after a successful refresh.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    /// Lifetime in seconds.
    pub expires_in: u64,
    /// The provider may or may not return a new refresh token.
    pub refresh_token: Option<String>,
}

/// Exchange a refresh token for a new access token using the given provider config.
pub async fn refresh_token(
    config: &OAuthConfig,
    refresh_token: &str,
) -> Result<TokenResponse, MailError> {
    let client = reqwest::Client::new();

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", &config.client_id),
        ("client_secret", &config.client_secret),
        ("redirect_uri", &config.redirect_uri),
    ];

    let response = client
        .post(&config.token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| MailError::OAuthFailed(format!("HTTP request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(MailError::OAuthFailed(format!(
            "Token endpoint returned {status}: {body}"
        )));
    }

    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| MailError::OAuthFailed(format!("Failed to parse token response: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn google_config_has_correct_token_url() {
        let cfg = OAuthConfig::google();
        assert_eq!(cfg.token_url, "https://oauth2.googleapis.com/token");
        assert!(!cfg.scopes.is_empty());
    }

    #[test]
    fn microsoft_config_has_correct_token_url() {
        let cfg = OAuthConfig::microsoft();
        assert!(cfg
            .token_url
            .contains("login.microsoftonline.com"));
        assert!(cfg.scopes.iter().any(|s| s == "offline_access"));
    }
}
