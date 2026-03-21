use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Path does not exist: {path}")]
    PathNotFound { path: PathBuf },

    #[error("Failed to run cargo metadata: {source}")]
    CargoMetadata {
        #[from]
        source: cargo_metadata::Error,
    },

    #[error("Cargo metadata did not identify a root package")]
    NoRootPackage,

    #[error("Failed to serialize JSON output: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
