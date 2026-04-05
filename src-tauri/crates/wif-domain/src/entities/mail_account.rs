use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// A configured mail account used to ingest emails.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailAccount {
    pub id: Ulid,
    pub name: String,
    pub email: String,
    pub provider: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub use_oauth: bool,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub is_active: bool,
}
