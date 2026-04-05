//! IMAP synchronisation service.
//!
//! Provides [`ImapSyncService`] which connects to an IMAP server, fetches
//! recent messages, and parses them into [`MailMessage`] domain entities.

use imap_proto::types::Address;
use ulid::Ulid;
use wif_domain::{MailAccount, MailMessage};

use crate::error::MailError;
use crate::html_extract::html_to_text;

/// Synchronises email from an IMAP server for a single [`MailAccount`].
pub struct ImapSyncService {
    account: MailAccount,
}

impl ImapSyncService {
    /// Create a new sync service for `account`.
    pub fn new(account: MailAccount) -> Self {
        Self { account }
    }

    /// Establish a TLS connection and authenticate to the IMAP server.
    ///
    /// For OAuth accounts, the caller must ensure `access_token` has been
    /// refreshed before calling this method. For plain-password accounts the
    /// password is expected in the `access_token` field (same convention as
    /// [`SmtpSendService`]).
    fn connect(&self) -> Result<imap::Session<Box<dyn imap::ImapConnection>>, MailError> {
        let host = &self.account.imap_host;
        let port = self.account.imap_port;

        // Use ClientBuilder which handles TLS negotiation automatically.
        // Port 993 -> direct TLS, port 143 -> STARTTLS.
        let client = imap::ClientBuilder::new(host.as_str(), port)
            .connect()
            .map_err(|e| MailError::ImapFailed(format!("连接 {host}:{port} 失败: {e}")))?;

        // Authenticate — XOAUTH2 for OAuth accounts, LOGIN for password accounts.
        let session = if self.account.use_oauth {
            let token = self
                .account
                .access_token
                .as_deref()
                .ok_or_else(|| {
                    MailError::OAuthFailed("OAuth 账户缺少 access_token".into())
                })?;
            // Build the XOAUTH2 SASL initial response:
            // "user=<email>\x01auth=Bearer <token>\x01\x01"
            let auth_string = format!(
                "user={}\x01auth=Bearer {}\x01\x01",
                self.account.email, token
            );
            client
                .authenticate("XOAUTH2", &XOAuth2Authenticator(auth_string))
                .map_err(|(e, _client)| {
                    MailError::OAuthFailed(format!("XOAUTH2 认证失败: {e}"))
                })?
        } else {
            let password = self
                .account
                .access_token
                .as_deref()
                .unwrap_or("");
            client
                .login(&self.account.email, password)
                .map_err(|(e, _client)| {
                    MailError::ImapFailed(format!("IMAP 登录失败: {e}"))
                })?
        };

        Ok(session)
    }

    /// Fetch the most recent `limit` messages from INBOX.
    ///
    /// Connects, authenticates, selects INBOX, fetches envelope and body data
    /// for the latest messages, parses them into [`MailMessage`] entities, and
    /// properly logs out when done.
    pub fn fetch_recent(&self, limit: usize) -> Result<Vec<MailMessage>, MailError> {
        let mut session = self.connect()?;

        // SELECT the INBOX folder.
        let mailbox = session
            .select("INBOX")
            .map_err(|e| MailError::ImapFailed(format!("SELECT INBOX 失败: {e}")))?;

        let total = mailbox.exists;
        if total == 0 {
            session.logout().ok();
            return Ok(vec![]);
        }

        // Calculate the sequence range for the most recent `limit` messages.
        let start = if total as usize > limit {
            total - limit as u32 + 1
        } else {
            1
        };
        let range = format!("{start}:{total}");

        // Fetch envelope, body text, and internal date for each message.
        let messages = session
            .fetch(
                &range,
                "(ENVELOPE BODY[TEXT] BODY[HEADER.FIELDS (MESSAGE-ID)] INTERNALDATE)",
            )
            .map_err(|e| MailError::ImapFailed(format!("FETCH 失败: {e}")))?;

        let mut result = Vec::with_capacity(messages.len());

        for msg in messages.iter() {
            let parsed = self.parse_imap_fetch(msg);
            match parsed {
                Ok(mail_msg) => result.push(mail_msg),
                Err(e) => {
                    // Log and skip malformed messages instead of failing the
                    // entire sync.
                    tracing::warn!("跳过无法解析的邮件: {e}");
                }
            }
        }

        // Cleanly close the session.
        session.logout().ok();

        Ok(result)
    }

    /// Fetch new messages since a Unix timestamp.
    ///
    /// When `since` is provided (Unix timestamp in seconds) only messages
    /// received after that point are returned. Returns parsed [`MailMessage`]
    /// entities ready for storage.
    pub async fn fetch_new_messages(
        &self,
        since: Option<i64>,
    ) -> Result<Vec<MailMessage>, MailError> {
        // Clone the data needed for the blocking task.
        let account = self.account.clone();
        let since_ts = since;

        // Run the blocking IMAP I/O on a Tokio blocking thread so we do not
        // starve the async runtime.
        tokio::task::spawn_blocking(move || {
            let svc = ImapSyncService::new(account);
            let mut session = svc.connect()?;

            // SELECT the INBOX folder.
            let mailbox = session
                .select("INBOX")
                .map_err(|e| MailError::ImapFailed(format!("SELECT INBOX 失败: {e}")))?;

            let total = mailbox.exists;
            if total == 0 {
                session.logout().ok();
                return Ok(vec![]);
            }

            // Determine which messages to fetch.
            let range = if let Some(ts) = since_ts {
                // Use SEARCH SINCE to narrow down the result set.
                let date_str = timestamp_to_imap_date(ts);
                let uids = session
                    .search(format!("SINCE {date_str}"))
                    .map_err(|e| MailError::ImapFailed(format!("SEARCH 失败: {e}")))?;
                if uids.is_empty() {
                    session.logout().ok();
                    return Ok(vec![]);
                }
                uids.iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            } else {
                // Fetch last 50 messages by default.
                let start = if total > 50 { total - 49 } else { 1 };
                format!("{start}:{total}")
            };

            let messages = session
                .fetch(
                    &range,
                    "(ENVELOPE BODY[TEXT] BODY[HEADER.FIELDS (MESSAGE-ID)] INTERNALDATE)",
                )
                .map_err(|e| MailError::ImapFailed(format!("FETCH 失败: {e}")))?;

            let mut result = Vec::with_capacity(messages.len());
            for msg in messages.iter() {
                match svc.parse_imap_fetch(msg) {
                    Ok(mail_msg) => result.push(mail_msg),
                    Err(e) => {
                        tracing::warn!("跳过无法解析的邮件: {e}");
                    }
                }
            }

            session.logout().ok();
            Ok(result)
        })
        .await
        .map_err(|e| MailError::ImapFailed(format!("IMAP 同步任务失败: {e}")))?
    }

    /// Start IMAP IDLE push monitoring.
    ///
    /// `on_new_message` is called each time the server signals that new mail
    /// has arrived, with the newly-fetched messages as the argument.
    ///
    /// # Current status
    /// Not yet implemented -- logs a warning and returns immediately.
    pub async fn start_idle<F>(&self, on_new_message: F) -> Result<(), MailError>
    where
        F: Fn(Vec<MailMessage>) + Send + 'static,
    {
        let _ = on_new_message;
        tracing::warn!("IMAP IDLE 暂未实现，将在后续版本中支持");
        Ok(())
    }

    /// Parse a single IMAP FETCH response into a [`MailMessage`].
    fn parse_imap_fetch(&self, fetch: &imap::types::Fetch<'_>) -> Result<MailMessage, MailError> {
        let envelope = fetch
            .envelope()
            .ok_or_else(|| MailError::ParseError("邮件缺少 ENVELOPE".into()))?;

        // Subject
        let subject = envelope
            .subject
            .as_ref()
            .map(|s| decode_imap_utf8(s))
            .unwrap_or_default();

        // From address
        let from_address = envelope
            .from
            .as_ref()
            .and_then(|addrs| addrs.first())
            .map(format_imap_address)
            .unwrap_or_default();

        // To addresses
        let to_addresses = envelope
            .to
            .as_ref()
            .map(|addrs| addrs.iter().map(format_imap_address).collect())
            .unwrap_or_default();

        // Message-ID: prefer the header field, fall back to envelope message_id.
        let message_id = extract_message_id_header(fetch)
            .or_else(|| {
                envelope
                    .message_id
                    .as_ref()
                    .map(|id| decode_imap_utf8(id))
            })
            .unwrap_or_else(|| format!("<generated-{}>", Ulid::new()));

        // Body text — try to extract from BODY[TEXT], falling back to empty.
        let raw_body = fetch
            .text()
            .map(|b| String::from_utf8_lossy(b).into_owned());

        let (body_text, body_html) = match raw_body {
            Some(ref text) if looks_like_html(text) => {
                let plain = html_to_text(text);
                (Some(plain), Some(text.clone()))
            }
            Some(text) => (Some(text), None),
            None => (None, None),
        };

        // Received date — parse INTERNALDATE, fall back to current time.
        let received_at = fetch
            .internal_date()
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        Ok(MailMessage {
            id: Ulid::new(),
            account_id: self.account.id,
            message_id,
            subject,
            from_address,
            to_addresses,
            body_text,
            body_html,
            received_at,
            is_read: false,
            work_item_id: None,
        })
    }
}

// ---------------------------------------------------------------------------
// XOAUTH2 authenticator
// ---------------------------------------------------------------------------

/// Minimal XOAUTH2 SASL authenticator for use with the `imap` crate.
struct XOAuth2Authenticator(String);

impl imap::Authenticator for XOAuth2Authenticator {
    type Response = String;

    fn process(&self, _data: &[u8]) -> Self::Response {
        self.0.clone()
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Decode an IMAP `Cow<[u8]>` value to a UTF-8 string, lossy.
fn decode_imap_utf8(data: &[u8]) -> String {
    String::from_utf8_lossy(data).into_owned()
}

/// Format an IMAP address into a `name <mailbox@host>` string.
fn format_imap_address(addr: &Address<'_>) -> String {
    let mailbox = addr
        .mailbox
        .as_ref()
        .map(|m| decode_imap_utf8(m))
        .unwrap_or_default();
    let host = addr
        .host
        .as_ref()
        .map(|h| decode_imap_utf8(h))
        .unwrap_or_default();
    let email = format!("{mailbox}@{host}");

    match addr.name.as_ref() {
        Some(name) => {
            let decoded = decode_imap_utf8(name);
            if decoded.is_empty() {
                email
            } else {
                format!("{decoded} <{email}>")
            }
        }
        None => email,
    }
}

/// Extract the Message-ID from the BODY[HEADER.FIELDS (MESSAGE-ID)] section.
fn extract_message_id_header(fetch: &imap::types::Fetch<'_>) -> Option<String> {
    let header_bytes = fetch.header();
    if let Some(data) = header_bytes {
        let text = String::from_utf8_lossy(data);
        for line in text.lines() {
            let lower = line.to_lowercase();
            if lower.starts_with("message-id:") {
                return Some(line["message-id:".len()..].trim().to_string());
            }
        }
    }
    None
}

/// Heuristic to check if a string is likely HTML.
fn looks_like_html(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("<html")
        || lower.contains("<body")
        || lower.contains("<div")
        || lower.contains("<p>")
}

/// Convert a Unix timestamp to an IMAP SEARCH date string (e.g. "05-Apr-2026").
fn timestamp_to_imap_date(ts: i64) -> String {
    use chrono::{TimeZone, Utc};
    let dt = Utc.timestamp_opt(ts, 0).single().unwrap_or_else(Utc::now);
    dt.format("%d-%b-%Y").to_string()
}

// ---------------------------------------------------------------------------
// Connection configuration (for testing)
// ---------------------------------------------------------------------------

/// Parameters derived from a [`MailAccount`] for establishing an IMAP connection.
///
/// Exposed for unit testing without needing a live server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapConnectionConfig {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
    pub use_oauth: bool,
    pub email: String,
}

impl ImapConnectionConfig {
    /// Derive connection parameters from a [`MailAccount`].
    pub fn from_account(account: &MailAccount) -> Self {
        Self {
            host: account.imap_host.clone(),
            port: account.imap_port,
            use_tls: account.imap_port == 993,
            use_oauth: account.use_oauth,
            email: account.email.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_account() -> MailAccount {
        MailAccount {
            id: Ulid::new(),
            name: "Test".into(),
            email: "test@example.com".into(),
            provider: "gmail".into(),
            imap_host: "imap.gmail.com".into(),
            imap_port: 993,
            smtp_host: "smtp.gmail.com".into(),
            smtp_port: 587,
            use_oauth: false,
            access_token: None,
            refresh_token: None,
            is_active: true,
        }
    }

    #[test]
    fn connection_config_from_account_standard_tls() {
        let account = make_account();
        let config = ImapConnectionConfig::from_account(&account);

        assert_eq!(config.host, "imap.gmail.com");
        assert_eq!(config.port, 993);
        assert!(config.use_tls);
        assert!(!config.use_oauth);
        assert_eq!(config.email, "test@example.com");
    }

    #[test]
    fn connection_config_from_account_oauth() {
        let mut account = make_account();
        account.use_oauth = true;
        account.access_token = Some("ya29.token".into());
        account.imap_port = 993;

        let config = ImapConnectionConfig::from_account(&account);

        assert!(config.use_oauth);
        assert!(config.use_tls);
    }

    #[test]
    fn connection_config_non_standard_port_no_tls() {
        let mut account = make_account();
        account.imap_port = 143;

        let config = ImapConnectionConfig::from_account(&account);

        assert_eq!(config.port, 143);
        assert!(!config.use_tls, "port 143 should not be flagged as TLS");
    }

    #[test]
    fn timestamp_to_imap_date_produces_valid_format() {
        // 2024-01-15 00:00:00 UTC
        let ts = 1705276800;
        let date_str = timestamp_to_imap_date(ts);
        // Should be DD-Mon-YYYY format
        assert!(
            date_str.contains('-'),
            "date should contain dashes: {date_str}"
        );
        assert!(date_str.len() >= 9, "date string too short: {date_str}");
        // Verify it contains "Jan" and "2024"
        assert!(date_str.contains("Jan"), "expected Jan in {date_str}");
        assert!(date_str.contains("2024"), "expected 2024 in {date_str}");
    }

    #[test]
    fn looks_like_html_detects_html_content() {
        assert!(looks_like_html("<html><body>Hello</body></html>"));
        assert!(looks_like_html("<div>Some content</div>"));
        assert!(looks_like_html("text before <p>paragraph</p>"));
        assert!(!looks_like_html("Just plain text, no tags here."));
        assert!(!looks_like_html("Email with a < comparison > symbol"));
    }

    #[test]
    fn format_address_with_name() {
        use std::borrow::Cow;
        let addr = Address {
            name: Some(Cow::Borrowed(b"Alice")),
            adl: None,
            mailbox: Some(Cow::Borrowed(b"alice")),
            host: Some(Cow::Borrowed(b"example.com")),
        };
        let formatted = format_imap_address(&addr);
        assert_eq!(formatted, "Alice <alice@example.com>");
    }

    #[test]
    fn format_address_without_name() {
        use std::borrow::Cow;
        let addr = Address {
            name: None,
            adl: None,
            mailbox: Some(Cow::Borrowed(b"bob")),
            host: Some(Cow::Borrowed(b"test.org")),
        };
        let formatted = format_imap_address(&addr);
        assert_eq!(formatted, "bob@test.org");
    }

    #[tokio::test]
    async fn start_idle_completes_immediately() {
        let svc = ImapSyncService::new(make_account());
        let result = svc.start_idle(|_msgs| {}).await;
        assert!(result.is_ok());
    }

    #[test]
    fn decode_imap_utf8_handles_ascii() {
        let data = b"Hello World";
        assert_eq!(decode_imap_utf8(data), "Hello World");
    }

    #[test]
    fn decode_imap_utf8_handles_utf8() {
        let data = "你好世界".as_bytes();
        assert_eq!(decode_imap_utf8(data), "你好世界");
    }
}
