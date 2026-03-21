use cargo_metadata::{DependencyKind as CargoDependencyKind, Metadata, MetadataCommand, Package};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    Normal,
    Development,
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub kind: DependencyKind,
}

#[derive(Debug, Clone)]
pub struct ParsedMetadata {
    pub workspace_root: PathBuf,
    pub package_name: String,
    pub declared_deps: Vec<DependencyInfo>,
    pub compiled_deps: Vec<DependencyInfo>,
    pub package_graph: HashMap<String, Vec<String>>,
}

pub fn parse_metadata(path: Option<PathBuf>) -> Result<ParsedMetadata> {
    let metadata = load_metadata(path.as_deref())?;
    let resolve = metadata.resolve.as_ref().ok_or(Error::NoRootPackage)?;
    let root_id = resolve.root.as_ref().ok_or(Error::NoRootPackage)?;
    let root_pkg = find_package(&metadata, root_id).ok_or(Error::NoRootPackage)?;

    Ok(ParsedMetadata {
        workspace_root: metadata.workspace_root.clone().into(),
        package_name: root_pkg.name.clone(),
        declared_deps: root_pkg.dependencies.iter().map(map_declared_dep).collect(),
        compiled_deps: collect_compiled_deps(&metadata, root_id),
        package_graph: build_package_graph(&metadata),
    })
}

fn load_metadata(path: Option<&Path>) -> Result<Metadata> {
    let mut command = MetadataCommand::new();

    if let Some(path) = path {
        if !path.exists() {
            return Err(Error::PathNotFound {
                path: path.to_path_buf(),
            });
        }

        if path.file_name().is_some_and(|name| name == "Cargo.toml") {
            command.manifest_path(path);
        } else {
            command.current_dir(path);
        }
    }

    Ok(command.exec()?)
}

fn find_package<'a>(
    metadata: &'a Metadata,
    package_id: &cargo_metadata::PackageId,
) -> Option<&'a Package> {
    metadata.packages.iter().find(|pkg| &pkg.id == package_id)
}

fn map_declared_dep(dep: &cargo_metadata::Dependency) -> DependencyInfo {
    DependencyInfo {
        name: dep.rename.clone().unwrap_or_else(|| dep.name.clone()),
        version: Some(dep.req.to_string()),
        source: dep
            .path
            .as_ref()
            .map(|path| format!("path+{}", path))
            .or_else(|| dep.registry.clone()),
        kind: map_kind(dep.kind),
    }
}

fn map_kind(kind: cargo_metadata::DependencyKind) -> DependencyKind {
    match kind {
        CargoDependencyKind::Development => DependencyKind::Development,
        CargoDependencyKind::Build => DependencyKind::Build,
        CargoDependencyKind::Normal | CargoDependencyKind::Unknown => DependencyKind::Normal,
    }
}

fn collect_compiled_deps(
    metadata: &Metadata,
    root_id: &cargo_metadata::PackageId,
) -> Vec<DependencyInfo> {
    let Some(resolve) = metadata.resolve.as_ref() else {
        return Vec::new();
    };

    resolve
        .nodes
        .iter()
        .filter(|node| &node.id != root_id)
        .filter_map(|node| find_package(metadata, &node.id))
        .map(|pkg| DependencyInfo {
            name: pkg.name.clone(),
            version: Some(pkg.version.to_string()),
            source: pkg.source.as_ref().map(ToString::to_string),
            kind: DependencyKind::Normal,
        })
        .collect()
}

fn build_package_graph(metadata: &Metadata) -> HashMap<String, Vec<String>> {
    let id_to_name: HashMap<_, _> = metadata
        .packages
        .iter()
        .map(|pkg| (pkg.id.clone(), pkg.name.clone()))
        .collect();

    let mut graph = HashMap::new();

    if let Some(resolve) = &metadata.resolve {
        for node in &resolve.nodes {
            let Some(name) = id_to_name.get(&node.id) else {
                continue;
            };

            let deps = node
                .deps
                .iter()
                .map(|dep| dep.name.clone())
                .collect::<Vec<_>>();

            graph.insert(name.clone(), deps);
        }
    }

    graph
}
