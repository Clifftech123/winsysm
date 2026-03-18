use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Disk scan failed: {0}")]
    DiskScan(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Cleanup error: {0}")]
    Cleanup(String),

    #[error("Security scan error: {0}")]
    Security(String),

    #[error("Permission denied: {0} — try running as Administrator")]
    PermissionDenied(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub type AppResult<T> = std::result::Result<T, AppError>;
