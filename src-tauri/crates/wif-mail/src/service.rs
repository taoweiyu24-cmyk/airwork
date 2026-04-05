//! [`MailService`] orchestrates IMAP sync and SMTP send across multiple accounts.

use std::collections::HashMap;

use wif_domain::{MailAccount, MailMessage};

use crate::error::MailError;
use crate::send::{SendRequest, SmtpSendService};
use crate::sync::ImapSyncService;

/// High-level mail service that manages multiple [`MailAccount`]s.
///
/// Register accounts with [`register_account`], then use [`sync_account`] to
/// pull new messages and [`send_mail`] to dispatch outgoing email.
pub struct MailService {
    sync_services: HashMap<String, ImapSyncService>,
    send_services: HashMap<String, SmtpSendService>,
}

impl MailService {
    /// Create an empty service with no registered accounts.
    pub fn new() -> Self {
        Self {
            sync_services: HashMap::new(),
            send_services: HashMap::new(),
        }
    }

    /// Register a mail account, replacing any previously registered account
    /// with the same ID.
    pub fn register_account(&mut self, account: MailAccount) {
        let id = account.id.to_string();
        self.sync_services
            .insert(id.clone(), ImapSyncService::new(account.clone()));
        self.send_services
            .insert(id, SmtpSendService::new(account));
    }

    /// Fetch new messages for `account_id` via IMAP.
    ///
    /// Returns `Err(MailError::AccountNotFound)` when no account with the
    /// given ID has been registered.
    pub async fn sync_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<MailMessage>, MailError> {
        let sync = self
            .sync_services
            .get(account_id)
            .ok_or_else(|| MailError::AccountNotFound(account_id.to_string()))?;
        sync.fetch_new_messages(None).await
    }

    /// Sync the INBOX of a given account, fetching the most recent `limit`
    /// messages via IMAP.
    ///
    /// This is the primary entry point for the Tauri IPC command. It accepts
    /// a [`MailAccount`] directly (looked up by the caller) so the service
    /// does not need to hold a database reference.
    pub async fn sync_inbox(
        account: &MailAccount,
        limit: usize,
    ) -> Result<Vec<MailMessage>, MailError> {
        let account_clone = account.clone();
        // Run blocking IMAP I/O on a dedicated thread.
        tokio::task::spawn_blocking(move || {
            let svc = ImapSyncService::new(account_clone);
            svc.fetch_recent(limit)
        })
        .await
        .map_err(|e| MailError::ImapFailed(format!("邮件同步任务失败: {e}")))?
    }

    /// Send an email on behalf of `account_id`.
    ///
    /// Returns `Err(MailError::AccountNotFound)` when no account with the
    /// given ID has been registered.
    pub fn send_mail(
        &self,
        account_id: &str,
        request: &SendRequest,
    ) -> Result<(), MailError> {
        let send = self
            .send_services
            .get(account_id)
            .ok_or_else(|| MailError::AccountNotFound(account_id.to_string()))?;
        send.send(request)
    }

    /// Returns `true` if an account with `account_id` is registered.
    pub fn has_account(&self, account_id: &str) -> bool {
        self.sync_services.contains_key(account_id)
    }

    /// Remove an account and its associated services.
    pub fn unregister_account(&mut self, account_id: &str) {
        self.sync_services.remove(account_id);
        self.send_services.remove(account_id);
    }
}

impl Default for MailService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ulid::Ulid;

    fn make_account(id: Ulid) -> MailAccount {
        MailAccount {
            id,
            name: "Test Account".into(),
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
    fn register_and_check_account() {
        let mut svc = MailService::new();
        let id = Ulid::new();
        assert!(!svc.has_account(&id.to_string()));
        svc.register_account(make_account(id));
        assert!(svc.has_account(&id.to_string()));
    }

    #[test]
    fn unregister_removes_account() {
        let mut svc = MailService::new();
        let id = Ulid::new();
        svc.register_account(make_account(id));
        svc.unregister_account(&id.to_string());
        assert!(!svc.has_account(&id.to_string()));
    }

    #[tokio::test]
    async fn sync_unknown_account_returns_not_found() {
        let svc = MailService::new();
        let err = svc.sync_account("nonexistent-id").await.unwrap_err();
        assert!(matches!(err, MailError::AccountNotFound(_)));
        assert!(err.to_string().contains("nonexistent-id"));
    }

    #[tokio::test]
    async fn sync_known_account_returns_imap_error_without_server() {
        // With real IMAP implementation, syncing against a test account that
        // has no real server will produce an ImapFailed or OAuthFailed error.
        let mut svc = MailService::new();
        let id = Ulid::new();
        svc.register_account(make_account(id));
        let result = svc.sync_account(&id.to_string()).await;
        assert!(
            result.is_err(),
            "should fail when IMAP server is unreachable"
        );
    }

    #[test]
    fn send_to_unknown_account_returns_not_found() {
        let svc = MailService::new();
        let req = SendRequest {
            to: vec!["someone@example.com".into()],
            subject: "Hello".into(),
            body_text: "World".into(),
            body_html: None,
        };
        let err = svc.send_mail("nonexistent-id", &req).unwrap_err();
        assert!(matches!(err, MailError::AccountNotFound(_)));
    }
}
