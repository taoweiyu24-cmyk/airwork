use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// An email message ingested from a mail account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailMessage {
    pub id: Ulid,
    pub account_id: Ulid,
    /// The message-id header value from the email.
    pub message_id: String,
    pub subject: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    /// Unix timestamp (seconds).
    pub received_at: i64,
    pub is_read: bool,
    pub work_item_id: Option<Ulid>,
}
