//! Domain entities for WorkItemFlow.

pub mod ai_profile;
pub mod attachment;
pub mod contact;
pub mod gis_feature;
pub mod gis_layer;
pub mod identity;
pub mod mail_account;
pub mod mail_message;
pub mod proposal;
pub mod work_event;
pub mod work_item;

pub use ai_profile::AiProfile;
pub use attachment::Attachment;
pub use contact::Contact;
pub use gis_feature::GisFeature;
pub use gis_layer::GisLayer;
pub use identity::{Identity, IdentityAlias};
pub use mail_account::MailAccount;
pub use mail_message::MailMessage;
pub use proposal::Proposal;
pub use work_event::WorkEvent;
pub use work_item::WorkItem;
