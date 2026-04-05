//! Repository trait definitions for WorkItemFlow.
//!
//! Each trait is a pure interface; concrete implementations live in `wif-data`.

pub mod ai_profile_repo;
pub mod attachment_repo;
pub mod contact_repo;
pub mod gis_repo;
pub mod mail_repo;
pub mod work_item_repo;

pub use ai_profile_repo::AiProfileRepository;
pub use attachment_repo::AttachmentRepository;
pub use contact_repo::ContactRepository;
pub use gis_repo::{GisFeatureRepository, GisLayerRepository};
pub use mail_repo::{MailAccountRepository, MailMessageRepository};
pub use work_item_repo::{ProposalRepository, WorkItemRepository};

/// Domain-level error type shared by all repository operations.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("database error: {0}")]
    DatabaseError(String),
}

/// Convenience alias used throughout the repository module.
pub type Result<T> = std::result::Result<T, DomainError>;
