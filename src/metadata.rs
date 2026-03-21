use cargo_metadata::{CargoOpt, DependencyKind, Metadata, MetadataCommand};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("No Cargo.toml found at {0}")]
    CargoTomlNotFound(PathBuf),

    #[error("Failed to run cargo metadata: {0}")]
    CargoMetadataError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type Result<T> = std::result::Result<T, MetadataError>;

/// Dependency information with its kind
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyInfo {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub kind: DependencyKind,
}

/// Parsed cargo metadata with dependency information
#[derive(Debug, Clone)]
pub struct ParsedMetadata {
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Package name from the manifest
    pub package_name: String,
    /// All declared dependencies (including dev and build)
    pub declared_deps: Vec<DependencyInfo>,
    /// Transitive compiled dependencies
    pub compiled_deps: Vec<DependencyInfo>,
    /// Dependency graph for resolving 'via' information
    pub dependencies: Vec<DependencyInfo>,
}

/// Parse cargo metadata from the current directory or specified path
pub fn parse_metadata(path: Option<PathBuf>) -> Result<ParsedMetadata> {
    let metadata = if let Some(p) = path {
        MetadataCommand::new().current_dir(p).exec()?
    } else {
        MetadataCommand::new().exec()?
    };

    let workspace_root = metadata.workspace_root.clone();
    let package_name = metadata
        .root
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_default();

    // Parse declared dependencies from the manifest
    let declared_deps = parse_declared_deps(&metadata)?;

    // Parse all dependencies from the metadata (including transitive)
    let dependencies = parse_dependencies(&metadata);

    // Filter to only the transitive compiled dependencies
    let compiled_deps = dependencies
        .into_iter()
        .filter(|dep| {
            // Only include dependencies that are not the root package
            !dep.name.eq(&package_name)
        })
        .collect();

    Ok(ParsedMetadata {
        workspace_root,
        package_name,
        declared_deps,
        compiled_deps,
        dependencies,
    })
}

/// Parse declared dependencies from the manifest
fn parse_declared_deps(metadata: &Metadata) -> Result<Vec<DependencyInfo>> {
    let manifest = metadata.manifest_path.as_path();

    let content =
        std::fs::read_to_string(manifest).map_err(|e| MetadataError::CargoTomlReadError {
            path: manifest.to_path_buf(),
            source: e,
        })?;

    let toml: toml::Value =
        toml::from_str(&content).map_err(|e| MetadataError::CargoTomlParseError {
            path: manifest.to_path_buf(),
            source: e,
        })?;

    let declared: Vec<DependencyInfo> = if let Some(table) = toml.get("dependencies") {
        parse_dependency_table(table, DependencyKind::Normal)
    } else {
        vec![]
    };

    let dev: Vec<DependencyInfo> = if let Some(table) = toml.get("dev-dependencies") {
        parse_dependency_table(table, DependencyKind::Development)
    } else {
        vec![]
    };

    let build: Vec<DependencyInfo> = if let Some(table) = toml.get("build-dependencies") {
        parse_dependency_table(table, DependencyKind::Build)
    } else {
        vec![]
    };

    // Combine all declared dependencies
    let mut all = declared;
    all.extend(dev);
    all.extend(build);

    Ok(all)
}

/// Parse a dependency table from TOML
fn parse_dependency_table(table: &toml::Value, kind: DependencyKind) -> Vec<DependencyInfo> {
    match table {
        toml::Value::Table(t) => t
            .iter()
            .filter_map(|(name, value)| {
                let name = name.to_string();
                let version = if let toml::Value::String(s) = value {
                    Some(s.value().clone())
                } else {
                    None
                };
                let source = if let toml::Value::Table(table) = value {
                    table.get("default-features").and_then(|v| {
                        if let toml::Value::Bool(b) = v {
                            if !b.value() {
                                return Some(
                                    "registry+https://github.com/rust-lang/crates.io-index"
                                        .to_string(),
                                );
                            }
                        }
                        None
                    })
                } else {
                    None
                };
                Some(DependencyInfo {
                    name,
                    version,
                    source,
                    kind,
                })
            })
            .collect(),
        _ => vec![],
    }
}

/// Parse all dependencies from cargo metadata
fn parse_dependencies(metadata: &Metadata) -> Vec<DependencyInfo> {
    let mut deps = Vec::new();

    for pkg in &metadata.packages {
        for dep in &pkg.dependencies {
            // Skip feature dependencies
            if dep.features.is_empty() {
                let kind = match dep.kind {
                    cargo_metadata::DependencyKind::Normal => DependencyKind::Normal,
                    cargo_metadata::DependencyKind::Development => DependencyKind::Development,
                    cargo_metadata::DependencyKind::Build => DependencyKind::Build,
                    cargo_metadata::DependencyKind::DevelopmentTool => DependencyKind::Development,
                    cargo_metadata::DependencyKind::Unknown => DependencyKind::Normal,
                };

                let source = if let Some(reg) = &dep.source {
                    if reg.starts_with("git+") {
                        Some(format!("git+{}", reg.strip_prefix("git+").unwrap_or(reg)))
                    } else {
                        Some(reg.clone())
                    }
                } else {
                    Some("registry+https://github.com/rust-lang/crates.io-index".to_string())
                };

                deps.push(DependencyInfo {
                    name: dep.name.clone(),
                    version: dep.version.clone(),
                    source,
                    kind,
                });
            }
        }
    }

    deps
}
