//! `wif-data` — SQLite data access layer for WorkItemFlow.
//!
//! Provides [`Database`] (connection wrapper + migrations) and
//! `Sqlite*Repo` types that implement the repository traits from `wif-domain`.

pub mod connection;
pub mod migrations;
pub mod repositories;

pub use connection::Database;
pub use repositories::{
    SqliteAiProfileRepo, SqliteAttachmentRepo, SqliteContactRepo, SqliteGisFeatureRepo,
    SqliteGisLayerRepo, SqliteMailAccountRepo, SqliteMailMessageRepo, SqliteProposalRepo,
    SqliteWorkItemRepo,
};
