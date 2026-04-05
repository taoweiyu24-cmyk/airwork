//! `wif-mail` — email integration for WorkItemFlow.
//!
//! Provides:
//! * [`MailService`] — high-level orchestrator for multi-account mail.
//! * [`ImapSyncService`] — fetch messages from an IMAP server.
//! * [`SmtpSendService`] — send messages via SMTP using `lettre`.
//! * [`OAuthConfig`] / [`OAuthProvider`] / [`refresh_token`] — OAuth2 helpers
//!   for Google and Microsoft providers.
//! * [`html_to_text`] — lightweight HTML → plain-text extractor.
//! * [`MailError`] — unified error type.

pub mod error;
pub mod html_extract;
pub mod oauth;
pub mod send;
pub mod service;
pub mod sync;

// Flat re-exports for ergonomic public API.
pub use error::MailError;
pub use html_extract::html_to_text;
pub use oauth::{refresh_token, OAuthConfig, OAuthProvider, TokenResponse};
pub use send::{SendRequest, SmtpSendService};
pub use service::MailService;
pub use sync::ImapSyncService;
