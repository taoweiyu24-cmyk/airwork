//! `wif-domain` — pure domain layer for WorkItemFlow.
//!
//! Contains entities, enums, value objects, and repository trait definitions.
//! No infrastructure dependencies (no database, network, or UI code).

pub mod entities;
pub mod enums;
pub mod repository;
pub mod value_objects;

// Flat re-exports for convenience.
pub use entities::{
    AiProfile, Attachment, Contact, GisFeature, GisLayer, Identity, IdentityAlias, MailAccount,
    MailMessage, Proposal, WorkEvent, WorkItem,
};

pub use enums::{AnalysisType, GeometryType, LayerType, Priority, Source, WorkItemStatus};

pub use value_objects::EgressPolicy;

pub use repository::{
    AiProfileRepository, AttachmentRepository, ContactRepository, DomainError,
    GisFeatureRepository, GisLayerRepository, MailAccountRepository, MailMessageRepository,
    ProposalRepository, WorkItemRepository,
};
