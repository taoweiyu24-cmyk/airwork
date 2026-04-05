//! `wif-core` — orchestration layer for WorkItemFlow.
//!
//! Provides the [`AppState`] composition root that holds all service instances,
//! the [`WifModule`] trait and [`ModuleHost`] for lifecycle management, and
//! the [`CoreError`] type for orchestration-level errors.

pub mod app_state;
pub mod error;
pub mod modules;

pub use app_state::AppState;
pub use error::CoreError;
pub use modules::{ModuleHost, WifModule};
