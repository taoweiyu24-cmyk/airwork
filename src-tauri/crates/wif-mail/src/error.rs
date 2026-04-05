/// Errors produced by the `wif-mail` crate.
#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("IMAP connection failed: {0}")]
    ImapFailed(String),

    #[error("SMTP send failed: {0}")]
    SmtpFailed(String),

    #[error("OAuth2 token refresh failed: {0}")]
    OAuthFailed(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
