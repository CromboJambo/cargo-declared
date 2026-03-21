use crate::metadata::{DependencyInfo, Metadata};
use std::collections::{HashMap, HashSet};

/// Four sets of dependencies
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySets {
    pub declared: Vec<DependencyInfo>,
    pub compiled: Vec<DependencyInfo>,
    pub delta: Vec<DeltaEntry>,
    pub orphaned: Vec<DependencyInfo>,
}

/// Entry in the delta set with 'via' information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaEntry {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub via: String,
}

/// Compute the four dependency sets from parsed metadata
pub fn compute_sets(
    declared: &[DependencyInfo],
    compiled: &[DependencyInfo],
    metadata: &Metadata,
) -> DependencySets {
    let declared_set: HashSet<_> = declared.iter().collect();
    let compiled_set: HashSet<_> = compiled.iter().collect();

    // Delta: compiled but not declared
    let delta: Vec<DeltaEntry> = compiled
        .iter()
        .filter(|dep| !declared_set.contains(dep))
        .map(|dep| DeltaEntry {
            name: dep.name.clone(),
            version: dep.version.clone(),
            source: dep.source.clone(),
            via: via_dependency(dep, metadata),
        })
        .collect();

    // Orphaned: declared but not compiled
    let orphaned: Vec<DependencyInfo> = declared
        .iter()
        .filter(|dep| !compiled_set.contains(dep))
        .cloned()
        .collect();

    DependencySets {
        declared: declared.to_vec(),
        compiled: compiled.to_vec(),
        delta,
        orphaned,
    }
}

/// Find which declared dependency is the ancestor of a transitive dependency
fn via_dependency(dep: &DependencyInfo, metadata: &Metadata) -> String {
    // Build a map from package name to its dependencies
    let package_deps: HashMap<&str, Vec<&str>> = metadata
        .packages
        .iter()
        .map(|pkg| {
            let name = pkg.name.as_str();
            let deps: Vec<&str> = pkg
                .dependencies
                .iter()
                .filter(|d| !d.features.is_empty()) // Only non-feature dependencies
                .map(|d| d.name.as_str())
                .collect();
            (name, deps)
        })
        .collect();

    // Build a map from package name to its info for lookup
    let compiled_map: HashMap<&str, &DependencyInfo> =
        compiled.iter().map(|d| (d.name.as_str(), d)).collect();

    // Find a declared dependency that has this as a dependency
    for declared_dep in declared {
        if let Some(deps) = package_deps.get(&declared_dep.name.as_str()) {
            if deps.contains(&dep.name.as_str()) {
                return declared_dep.name.clone();
            }
        }
    }

    // Fallback: use the package name if nothing found
    "unknown".to_string()
}

/// Format the dependency sets for human-readable output
pub fn format_human(sets: &DependencySets) -> String {
    let mut output = String::new();

    output.push_str("cargo-declared v0.1.0\n\n");

    output.push_str(&format!("declared:  {}\n", sets.declared.len()));
    output.push_str(&format!("compiled:  {}\n", sets.compiled.len()));
    output.push_str(&format!("delta:     {}\n", sets.delta.len()));

    if !sets.delta.is_empty() {
        output.push_str("\n+ transitive ({})\n", sets.delta.len());
        for entry in &sets.delta {
            output.push_str(&format!(
                "  {} {:?} via: {}\n",
                entry.name, entry.version, entry.via
            ));
        }
    }

    if !sets.orphaned.is_empty() {
        output.push_str("\n~ orphaned ({})\n", sets.orphaned.len());
        for dep in &sets.orphaned {
            output.push_str(&format!("  {}\n", dep.name));
        }
    }

    output
}

/// Format the dependency sets for JSON output
pub fn format_json(sets: &DependencySets) -> Result<String, serde_json::Error> {
    let json = serde_json::json!({
        "declared": sets.declared.iter().map(|d| d.name.clone()).collect::<Vec<_>>(),
        "compiled": sets.compiled.iter().map(|d| d.name.clone()).collect::<Vec<_>>(),
        "delta": sets.delta.iter().map(|d| serde_json::json!({
            "name": d.name,
            "version": d.version,
            "source": d.source,
            "via": d.via
        })).collect::<Vec<_>>(),
        "orphaned": sets.orphaned.iter().map(|d| d.name.clone()).collect::<Vec<_>>(),
        "summary": {
            "declared_count": sets.declared.len(),
            "compiled_count": sets.compiled.len(),
            "delta_count": sets.delta.len(),
            "orphaned_count": sets.orphaned.len()
        }
    });

    Ok(serde_json::to_string_pretty(&json)?)
}
