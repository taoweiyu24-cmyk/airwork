//! IMAP synchronisation service.
//!
//! The public interface is fully defined; the IMAP transport is stubbed pending
//! a production-ready async IMAP integration.

use wif_domain::{MailAccount, MailMessage};

use crate::error::MailError;

/// Synchronises email from an IMAP server for a single [`MailAccount`].
pub struct ImapSyncService {
    account: MailAccount,
}

impl ImapSyncService {
    /// Create a new sync service for `account`.
    pub fn new(account: MailAccount) -> Self {
        Self { account }
    }

    /// Fetch new messages from the IMAP server.
    ///
    /// When `since` is provided (Unix timestamp in seconds) only messages
    /// received after that point are returned.  Returns parsed [`MailMessage`]
    /// entities ready for storage.
    ///
    /// # Current status
    /// The actual IMAP transport is not yet implemented.  This method returns
    /// an empty `Vec` so callers can rely on the correct interface today.
    pub async fn fetch_new_messages(
        &self,
        since: Option<i64>,
    ) -> Result<Vec<MailMessage>, MailError> {
        // TODO: Implement actual IMAP connection.
        //
        // Outline:
        //   1. If self.account.use_oauth, exchange the stored refresh token for
        //      a fresh access token via oauth::refresh_token().
        //   2. Open a TLS stream to self.account.imap_host:self.account.imap_port.
        //   3. Authenticate (XOAUTH2 for OAuth accounts, plain for password).
        //   4. SELECT the INBOX folder.
        //   5. SEARCH SINCE <date> to get UIDs.
        //   6. FETCH each UID for ENVELOPE + BODY[TEXT].
        //   7. Map raw IMAP responses to MailMessage entities (new Ulid, set
        //      account_id, parse headers, extract body_text / body_html via
        //      html_extract::html_to_text).
        //   8. Return the Vec<MailMessage>.
        let _ = since; // suppress unused-variable warning until implemented
        let _ = &self.account; // suppress unused-field warning
        Ok(vec![])
    }

    /// Start IMAP IDLE push monitoring.
    ///
    /// `on_new_message` is called each time the server signals that new mail
    /// has arrived, with the newly-fetched messages as the argument.
    ///
    /// # Current status
    /// Not yet implemented — returns immediately with `Ok(())`.
    pub async fn start_idle<F>(&self, on_new_message: F) -> Result<(), MailError>
    where
        F: Fn(Vec<MailMessage>) + Send + 'static,
    {
        // TODO: Implement actual IMAP IDLE.
        //
        // Outline:
        //   1. Connect and authenticate (same as fetch_new_messages).
        //   2. Enter IDLE mode (RFC 2177).
        //   3. Spawn a background task that:
        //      a. Waits for an EXISTS or RECENT response.
        //      b. Calls fetch_new_messages(Some(last_seen_ts)) on receipt.
        //      c. Invokes on_new_message(messages).
        //      d. Re-enters IDLE (IMAP IDLE has a 29-minute server timeout).
        let _ = on_new_message;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ulid::Ulid;

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

    #[tokio::test]
    async fn fetch_new_messages_returns_empty_vec_when_stubbed() {
        let svc = ImapSyncService::new(make_account());
        let messages = svc.fetch_new_messages(None).await.unwrap();
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn fetch_new_messages_with_since_returns_empty_vec() {
        let svc = ImapSyncService::new(make_account());
        let messages = svc.fetch_new_messages(Some(1_700_000_000)).await.unwrap();
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn start_idle_completes_immediately_when_stubbed() {
        let svc = ImapSyncService::new(make_account());
        let result = svc.start_idle(|_msgs| {}).await;
        assert!(result.is_ok());
    }
}
