use std::path::PathBuf;

/// Error types for cargo-declared
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No Cargo.toml found at {path}")]
    CargoTomlNotFound { path: PathBuf },

    #[error("Failed to read Cargo.toml at {path}: {source}")]
    CargoTomlReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse Cargo.toml at {path}: {source}")]
    CargoTomlParseError {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Failed to run cargo metadata: {source}")]
    CargoMetadataError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("No workspace members found in the current directory")]
    NoWorkspaceMembers,

    #[error("No workspace members found in the specified directory")]
    NoWorkspaceMembersInPath { path: PathBuf },

    #[error("Failed to deserialize JSON output: {source}")]
    JsonError {
        #[source]
        source: serde_json::Error,
    },
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
