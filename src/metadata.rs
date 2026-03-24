use cargo_metadata::{DependencyKind as CargoDependencyKind, Metadata, MetadataCommand, Package};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    Normal,
    Development,
    Build,
}

/// Public API struct representing a dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub kind: DependencyKind,
    #[serde(skip_serializing)]
    pub package_name: String,
    #[serde(skip_serializing)]
    pub package_id: Option<String>,
    #[serde(skip_serializing)]
    pub optional: bool,
}

/// Internal struct for resolver operations
#[derive(Debug, Clone)]
pub struct ParsedMetadata {
    pub workspace_root: PathBuf,
    pub package_name: String,
    pub root_package_id: String,
    pub declared_deps: Vec<DependencyInfo>,
    pub compiled_deps: Vec<DependencyInfo>,
    pub package_graph: HashMap<String, Vec<String>>,
    pub package_names: HashMap<String, String>,
    pub direct_dep_names: HashMap<String, String>,
}

/// Internal struct for computing dependency sets
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySets {
    pub declared: Vec<DependencyInfo>,
    pub compiled: Vec<DependencyInfo>,
    pub delta: Vec<DeltaEntry>,
    pub orphaned: Vec<DependencyInfo>,
    pub optional: Vec<DependencyInfo>,
}

/// Entry in the delta (transitive dependencies) set
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeltaEntry {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub via: String,
}

pub fn parse_metadata(path: Option<PathBuf>) -> Result<ParsedMetadata> {
    let metadata = load_metadata(path.as_deref())?;
    let resolve = metadata.resolve.as_ref().ok_or(Error::NoRootPackage)?;
    let root_id = resolve.root.as_ref().ok_or(Error::NoRootPackage)?;
    let root_pkg = find_package(&metadata, root_id).ok_or(Error::NoRootPackage)?;
    let package_names = metadata
        .packages
        .iter()
        .map(|pkg| (pkg.id.to_string(), pkg.name.clone()))
        .collect::<HashMap<_, _>>();
    let root_dep_ids = resolve
        .nodes
        .iter()
        .find(|node| &node.id == root_id)
        .map(|node| {
            node.deps
                .iter()
                .map(|dep| (dep.name.clone(), dep.pkg.to_string()))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let declared_deps = root_pkg
        .dependencies
        .iter()
        .map(|dep| map_declared_dep(dep, root_dep_ids.get(dependency_display_name(dep)).cloned()))
        .collect::<Vec<_>>();
    let direct_dep_names = declared_deps
        .iter()
        .filter_map(|dep| {
            dep.package_id
                .clone()
                .map(|package_id| (package_id, dep.name.clone()))
        })
        .collect::<HashMap<_, _>>();

    Ok(ParsedMetadata {
        workspace_root: metadata.workspace_root.clone().into(),
        package_name: root_pkg.name.clone(),
        root_package_id: root_id.to_string(),
        declared_deps,
        compiled_deps: collect_compiled_deps(&metadata, root_id),
        package_graph: build_package_graph(&metadata),
        package_names,
        direct_dep_names,
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

fn dependency_display_name(dep: &cargo_metadata::Dependency) -> &str {
    dep.rename.as_deref().unwrap_or(&dep.name)
}

fn map_declared_dep(
    dep: &cargo_metadata::Dependency,
    package_id: Option<String>,
) -> DependencyInfo {
    DependencyInfo {
        name: dependency_display_name(dep).to_string(),
        version: Some(dep.req.to_string()),
        source: dep
            .path
            .as_ref()
            .map(|path| format!("path+{}", path))
            .or_else(|| dep.registry.clone()),
        kind: map_kind(dep.kind),
        package_name: dep.name.clone(),
        package_id,
        optional: map_optional(dep.kind),
    }
}

fn map_kind(kind: cargo_metadata::DependencyKind) -> DependencyKind {
    match kind {
        CargoDependencyKind::Development => DependencyKind::Development,
        CargoDependencyKind::Build => DependencyKind::Build,
        CargoDependencyKind::Normal | CargoDependencyKind::Unknown => DependencyKind::Normal,
    }
}

fn map_optional(kind: cargo_metadata::DependencyKind) -> bool {
    matches!(kind, cargo_metadata::DependencyKind::Unknown)
}

fn collect_compiled_deps(
    metadata: &Metadata,
    root_id: &cargo_metadata::PackageId,
) -> Vec<DependencyInfo> {
    let Some(resolve) = metadata.resolve.as_ref() else {
        return Vec::new();
    };

    let node_map: HashMap<_, _> = resolve.nodes.iter().map(|node| (&node.id, node)).collect();
    let mut queue = VecDeque::from([(root_id.clone(), DependencyKind::Normal)]);
    let mut kinds = HashMap::new();

    while let Some((current_id, current_kind)) = queue.pop_front() {
        let Some(node) = node_map.get(&current_id) else {
            continue;
        };

        for dep in &node.deps {
            let edge_kind = dep
                .dep_kinds
                .iter()
                .map(|info| map_kind(info.kind))
                .max_by_key(kind_rank)
                .unwrap_or(DependencyKind::Normal);
            let dep_kind = propagate_kind(&current_kind, &edge_kind);
            let should_enqueue = match kinds.get(&dep.pkg) {
                Some(existing) if kind_rank(existing) >= kind_rank(&dep_kind) => false,
                _ => {
                    kinds.insert(dep.pkg.clone(), dep_kind.clone());
                    true
                }
            };

            if should_enqueue {
                queue.push_back((dep.pkg.clone(), dep_kind));
            }
        }
    }

    resolve
        .nodes
        .iter()
        .filter(|node| &node.id != root_id)
        .filter_map(|node| find_package(metadata, &node.id).zip(kinds.get(&node.id)))
        .map(|(pkg, kind)| DependencyInfo {
            name: pkg.name.clone(),
            version: Some(pkg.version.to_string()),
            source: pkg.source.as_ref().map(ToString::to_string),
            kind: kind.clone(),
            package_name: pkg.name.clone(),
            package_id: Some(pkg.id.to_string()),
            optional: false,
        })
        .collect()
}

fn kind_rank(kind: &DependencyKind) -> u8 {
    match kind {
        DependencyKind::Development => 0,
        DependencyKind::Build => 1,
        DependencyKind::Normal => 2,
    }
}

fn propagate_kind(current: &DependencyKind, edge: &DependencyKind) -> DependencyKind {
    if matches!(current, DependencyKind::Normal) {
        edge.clone()
    } else {
        current.clone()
    }
}

fn build_package_graph(metadata: &Metadata) -> HashMap<String, Vec<String>> {
    let mut graph = HashMap::new();

    if let Some(resolve) = &metadata.resolve {
        for node in &resolve.nodes {
            let deps = node
                .deps
                .iter()
                .map(|dep| dep.pkg.to_string())
                .collect::<Vec<_>>();

            graph.insert(node.id.to_string(), deps);
        }
    }

    graph
}
