pub mod delta;
pub mod error;
pub mod metadata;
pub mod output;

use crate::error::Error;
use crate::metadata::ParsedMetadata;
use crate::output::{display_human, display_invariant, display_json};

/// Parse cargo metadata from the current directory or specified path
pub fn parse_metadata(path: Option<std::path::PathBuf>) -> Result<ParsedMetadata, Error> {
    crate::metadata::parse_metadata(path)
}

/// Compute the dependency sets and display human-readable output
pub fn compute_and_display_human(path: Option<std::path::PathBuf>) -> Result<String, Error> {
    let parsed = parse_metadata(path)?;
    display_human(&parsed)
}

/// Compute the dependency sets and display JSON output
pub fn compute_and_display_json(path: Option<std::path::PathBuf>) -> Result<String, Error> {
    let parsed = parse_metadata(path)?;
    display_json(&parsed)
}

/// Validate the declared/compiled invariant
pub fn validate_invariant(path: Option<std::path::PathBuf>) -> Result<bool, Error> {
    let parsed = parse_metadata(path)?;
    display_invariant(display_invariant(true))
}

/// Main entry point for the public API
pub struct CargoDeclared {
    path: Option<std::path::PathBuf>,
}

impl CargoDeclared {
    /// Create a new CargoDeclared instance
    pub fn new() -> Self {
        Self { path: None }
    }

    /// Set a custom path for the Cargo.toml
    pub fn with_path(mut self, path: std::path::PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    /// Run the tool and get human-readable output
    pub fn run_human(self) -> Result<String, Error> {
        compute_and_display_human(self.path)
    }

    /// Run the tool and get JSON output
    pub fn run_json(self) -> Result<String, Error> {
        compute_and_display_json(self.path)
    }
}

impl Default for CargoDeclared {
    fn default() -> Self {
        Self::new()
    }
}
