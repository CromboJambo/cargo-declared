pub mod delta;
pub mod error;
pub mod metadata;
pub mod output;

pub use crate::error::Error;
use crate::metadata::ParsedMetadata;
use crate::output::{display_human, display_json};

pub fn parse_metadata(path: Option<std::path::PathBuf>) -> Result<ParsedMetadata, Error> {
    crate::metadata::parse_metadata(path)
}

pub fn compute_and_display_human(path: Option<std::path::PathBuf>) -> Result<String, Error> {
    let parsed = parse_metadata(path)?;
    display_human(&parsed)
}

pub fn compute_and_display_json(path: Option<std::path::PathBuf>) -> Result<String, Error> {
    let parsed = parse_metadata(path)?;
    display_json(&parsed)
}

pub struct CargoDeclared {
    path: Option<std::path::PathBuf>,
}

impl CargoDeclared {
    pub fn new() -> Self {
        Self { path: None }
    }

    pub fn with_path(mut self, path: std::path::PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn run_human(self) -> Result<String, Error> {
        compute_and_display_human(self.path)
    }

    pub fn run_json(self) -> Result<String, Error> {
        compute_and_display_json(self.path)
    }
}

impl Default for CargoDeclared {
    fn default() -> Self {
        Self::new()
    }
}
