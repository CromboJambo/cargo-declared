use crate::metadata::{DependencyInfo, ParsedMetadata};
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencySets {
    pub declared: Vec<DependencyInfo>,
    pub compiled: Vec<DependencyInfo>,
    pub delta: Vec<DeltaEntry>,
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
        .map(|dep| {
            format!(
                "{}:{}",
                dep.name,
                dep.version.as_deref().unwrap_or("unknown")
            )
        })
        .collect::<HashSet<_>>();
    let compiled_names = parsed
        .compiled_deps
        .iter()
        .map(|dep| {
            format!(
                "{}:{}",
                dep.name,
                dep.version.as_deref().unwrap_or("unknown")
            )
        })
        .collect::<HashSet<_>>();
    let predecessors = shortest_predecessors(parsed);

    let delta = parsed
        .compiled_deps
        .iter()
        .filter(|dep| {
            !declared_names.contains(&format!(
                "{}:{}",
                dep.name,
                dep.version.as_deref().unwrap_or("unknown")
            ))
        })
        .map(|dep| DeltaEntry {
            name: dep.name.clone(),
            version: dep.version.clone(),
            source: dep.source.clone(),
            via: via_dependency(parsed, dep),
        })
        .sorted_by(|a, b| {
            a.name.cmp(&b.name).then_with(|| {
                a.version
                    .as_deref()
                    .cmp(&b.version.as_deref().unwrap_or(""))
                    .then_with(|| a.source.cmp(&b.source))
            })
        })
        .collect();

    DependencySets {
        declared: parsed.declared_deps.clone(),
        compiled: parsed.compiled_deps.clone(),
        delta,
    }
}

fn shortest_predecessors(parsed: &ParsedMetadata) -> HashMap<String, String> {
    let mut queue = VecDeque::from([parsed.root_package_id.clone()]);
    let mut visited = HashSet::new();
    let mut predecessors = HashMap::new();

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current.clone()) {
            continue;
        }

        let Some(children) = parsed.package_graph.get(&current) else {
            continue;
        };

        for child in children {
            if !predecessors.contains_key(child) {
                predecessors.insert(child.clone(), current.clone());
            }
            queue.push_back(child.clone());
        }
    }

    predecessors
}

fn via_dependency(parsed: &ParsedMetadata, dep: &DependencyInfo) -> String {
    // Find which direct dependency brought in this transitive dependency
    // We need to match package_id to a direct dependency
    if let Some(package_id) = &dep.version {
        // Look up the package name from the package_id
        if let Some(package_name) = parsed.package_names.get(package_id) {
            // Check if this is a direct dependency
            if parsed.declared_deps.iter().any(|d| d.name == *package_name) {
                return package_name.clone();
            }
        }
    }

    // If we can't find it, check if it's a root dependency
    if parsed
        .declared_deps
        .iter()
        .any(|d| d.version == dep.version)
    {
        return "root".to_string();
    }

    "unknown".to_string()
}

pub fn format_human(sets: &DependencySets) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "cargo-declared v{}\n\n",
        env!("CARGO_PKG_VERSION")
    ));
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

    output
}

pub fn format_json(sets: &DependencySets) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&sets)
}
