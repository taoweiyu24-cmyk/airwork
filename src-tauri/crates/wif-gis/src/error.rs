/// GIS module errors.
#[derive(Debug, thiserror::Error)]
pub enum GisError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    #[error("Coordinate transform error: {0}")]
    TransformError(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}
