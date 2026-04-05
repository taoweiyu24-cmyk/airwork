/// Errors that can occur in the wif-core orchestration layer.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Initialization failed: {0}")]
    InitFailed(String),
    #[error("Module error: {0}")]
    ModuleError(String),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
