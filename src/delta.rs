use crate::metadata::{DependencyInfo, ParsedMetadata};
use serde::Serialize;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySets {
    pub declared: Vec<DependencyInfo>,
    pub compiled: Vec<DependencyInfo>,
    pub delta: Vec<DeltaEntry>,
    pub orphaned: Vec<DependencyInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeltaEntry {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub via: String,
}

pub fn compute_sets(parsed: &ParsedMetadata) -> DependencySets {
    let declared_names = parsed
        .declared_deps
        .iter()
        .map(|dep| dep.name.as_str())
        .collect::<HashSet<_>>();
    let compiled_names = parsed
        .compiled_deps
        .iter()
        .map(|dep| dep.name.as_str())
        .collect::<HashSet<_>>();

    let delta = parsed
        .compiled_deps
        .iter()
        .filter(|dep| !declared_names.contains(dep.name.as_str()))
        .map(|dep| DeltaEntry {
            name: dep.name.clone(),
            version: dep.version.clone(),
            source: dep.source.clone(),
            via: via_dependency(parsed, &dep.name),
        })
        .collect();

    let orphaned = parsed
        .declared_deps
        .iter()
        .filter(|dep| !compiled_names.contains(dep.name.as_str()))
        .cloned()
        .collect();

    DependencySets {
        declared: parsed.declared_deps.clone(),
        compiled: parsed.compiled_deps.clone(),
        delta,
        orphaned,
    }
}

fn via_dependency(parsed: &ParsedMetadata, target: &str) -> String {
    for dep in &parsed.declared_deps {
        if dep.name == target {
            return dep.name.clone();
        }

        if reaches_target(parsed, &dep.name, target) {
            return dep.name.clone();
        }
    }

    "unknown".to_string()
}

fn reaches_target(parsed: &ParsedMetadata, start: &str, target: &str) -> bool {
    let mut queue = VecDeque::from([start.to_string()]);
    let mut visited = HashSet::new();

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current.clone()) {
            continue;
        }

        let Some(children) = parsed.package_graph.get(&current) else {
            continue;
        };

        if children.iter().any(|child| child == target) {
            return true;
        }

        queue.extend(children.iter().cloned());
    }

    false
}

pub fn format_human(sets: &DependencySets) -> String {
    let mut output = String::new();

    output.push_str("cargo-declared v0.1.0\n\n");
    output.push_str(&format!("declared:  {}\n", sets.declared.len()));
    output.push_str(&format!("compiled:  {}\n", sets.compiled.len()));
    output.push_str(&format!("delta:     {}\n", sets.delta.len()));

    if !sets.delta.is_empty() {
        output.push_str(&format!("\n+ transitive ({})\n", sets.delta.len()));
        for entry in &sets.delta {
            output.push_str(&format!(
                "  {} {} via: {}\n",
                entry.name,
                entry.version.as_deref().unwrap_or("unknown"),
                entry.via
            ));
        }
    }

    if !sets.orphaned.is_empty() {
        output.push_str(&format!("\n~ orphaned ({})\n", sets.orphaned.len()));
        for dep in &sets.orphaned {
            output.push_str(&format!("  {}\n", dep.name));
        }
    }

    output
}

pub fn format_json(sets: &DependencySets) -> Result<String, serde_json::Error> {
    let json = serde_json::json!({
        "declared": sets.declared,
        "compiled": sets.compiled,
        "delta": sets.delta,
        "orphaned": sets.orphaned,
        "summary": {
            "declared_count": sets.declared.len(),
            "compiled_count": sets.compiled.len(),
            "delta_count": sets.delta.len(),
            "orphaned_count": sets.orphaned.len()
        }
    });

    serde_json::to_string_pretty(&json)
}
