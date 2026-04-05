//! wif-docs — Document processing for PDF/DOCX/XLSX/PPTX.
//!
//! Provides trait interfaces ([`DocumentProcessor`], [`DocumentExporter`]) and
//! concrete implementations for XLSX, CSV, and plain-text files.
//!
//! PDF and DOCX processors are intentionally deferred to a later phase due to
//! current limitations in the Rust ecosystem.  The trait interface is stable so
//! those processors can be added without breaking existing callers.

pub mod error;
pub mod exporters;
pub mod metadata;
pub mod processors;
pub mod traits;

// Re-export the public API at the crate root for ergonomic use.
pub use error::DocsError;
pub use exporters::{get_exporter, CsvExporter};
pub use metadata::DocumentMetadata;
pub use processors::{get_processor, CsvProcessor, TextProcessor, XlsxProcessor};
pub use traits::{DocumentExporter, DocumentProcessor};
