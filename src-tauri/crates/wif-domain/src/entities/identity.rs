use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// The authenticated user identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub id: Ulid,
    pub name: String,
    pub email: String,
}

/// An alias email address linked to an [`Identity`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityAlias {
    pub identity_id: Ulid,
    pub alias_email: String,
}
