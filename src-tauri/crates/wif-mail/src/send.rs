//! SMTP send service backed by `lettre`.

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use wif_domain::MailAccount;

use crate::error::MailError;

/// Parameters for an outgoing email.
#[derive(Debug, Clone)]
pub struct SendRequest {
    /// List of recipient addresses.  At least one is required.
    pub to: Vec<String>,
    pub subject: String,
    pub body_text: String,
    /// Optional HTML alternative body.
    pub body_html: Option<String>,
}

/// Sends email via SMTP on behalf of a [`MailAccount`].
pub struct SmtpSendService {
    account: MailAccount,
}

impl SmtpSendService {
    pub fn new(account: MailAccount) -> Self {
        Self { account }
    }

    /// Send an email described by `request`.
    ///
    /// Currently sends only the plain-text body even when `body_html` is
    /// provided; multi-part MIME support can be added when required.
    pub fn send(&self, request: &SendRequest) -> Result<(), MailError> {
        if request.to.is_empty() {
            return Err(MailError::SmtpFailed(
                "At least one recipient is required".into(),
            ));
        }

        let from = self
            .account
            .email
            .parse()
            .map_err(|e| MailError::ParseError(format!("Invalid from address: {e}")))?;

        let to = request.to[0]
            .parse()
            .map_err(|e| MailError::ParseError(format!("Invalid to address: {e}")))?;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(&request.subject)
            .header(ContentType::TEXT_PLAIN)
            .body(request.body_text.clone())
            .map_err(|e| MailError::SmtpFailed(format!("Failed to build message: {e}")))?;

        // For OAuth accounts the "password" is the access token; for plain
        // accounts the caller should store the password in `access_token`.
        let password = self
            .account
            .access_token
            .clone()
            .unwrap_or_default();

        let creds = Credentials::new(self.account.email.clone(), password);

        let mailer = SmtpTransport::relay(&self.account.smtp_host)
            .map_err(|e| MailError::SmtpFailed(format!("Failed to connect to SMTP: {e}")))?
            .port(self.account.smtp_port)
            .credentials(creds)
            .build();

        mailer
            .send(&email)
            .map_err(|e| MailError::SmtpFailed(format!("Failed to send message: {e}")))?;

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
            name: "Sender".into(),
            email: "sender@example.com".into(),
            provider: "smtp".into(),
            imap_host: "imap.example.com".into(),
            imap_port: 993,
            smtp_host: "smtp.example.com".into(),
            smtp_port: 587,
            use_oauth: false,
            access_token: Some("secret".into()),
            refresh_token: None,
            is_active: true,
        }
    }

    #[test]
    fn send_fails_with_empty_recipient_list() {
        let svc = SmtpSendService::new(make_account());
        let req = SendRequest {
            to: vec![],
            subject: "Hi".into(),
            body_text: "Hello".into(),
            body_html: None,
        };
        let err = svc.send(&req).unwrap_err();
        assert!(err.to_string().contains("recipient"));
    }

    #[test]
    fn send_fails_with_invalid_to_address() {
        let svc = SmtpSendService::new(make_account());
        let req = SendRequest {
            to: vec!["not-an-email".into()],
            subject: "Hi".into(),
            body_text: "Hello".into(),
            body_html: None,
        };
        let err = svc.send(&req).unwrap_err();
        // Either ParseError or SmtpFailed is acceptable here.
        let msg = err.to_string();
        assert!(msg.contains("address") || msg.contains("parse") || msg.contains("build") || msg.contains("Invalid"), "unexpected error: {msg}");
    }
}
