/// Errors produced by the wif-docs crate.
#[derive(Debug, thiserror::Error)]
pub enum DocsError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),
}
