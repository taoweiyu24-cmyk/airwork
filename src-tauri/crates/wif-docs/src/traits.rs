use std::path::Path;

use crate::{DocsError, DocumentMetadata};

/// Extracts content and metadata from a document file.
pub trait DocumentProcessor: Send + Sync {
    /// Extract text content from a document file.
    fn extract_text(&self, path: &Path) -> Result<String, DocsError>;

    /// Extract structured metadata from a document file.
    fn extract_metadata(&self, path: &Path) -> Result<DocumentMetadata, DocsError>;

    /// Return `true` if this processor handles the given file extension.
    ///
    /// The extension should be provided without a leading dot, e.g. `"xlsx"`.
    fn supports(&self, extension: &str) -> bool;
}

/// Exports tabular data to a file in a specific format.
pub trait DocumentExporter: Send + Sync {
    /// Write `rows` (each a `Vec<String>`) to `path`, preceded by `headers`.
    fn export(
        &self,
        path: &Path,
        headers: &[String],
        rows: &[Vec<String>],
    ) -> Result<(), DocsError>;

    /// Human-readable name for the export format, e.g. `"csv"`.
    fn format(&self) -> &str;
}
