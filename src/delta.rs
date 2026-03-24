use crate::metadata::{DependencyInfo, ParsedMetadata};
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
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
        .map(dependency_identity)
        .collect::<HashSet<_>>();
    let compiled_names = parsed
        .compiled_deps
        .iter()
        .map(dependency_identity)
        .collect::<HashSet<_>>();
    let predecessors = shortest_predecessors(parsed);

    let delta = parsed
        .compiled_deps
        .iter()
        .filter(|dep| !declared_names.contains(&dependency_identity(dep)))
        .map(|dep| DeltaEntry {
            name: dep.name.clone(),
            version: dep.version.clone(),
            source: dep.source.clone(),
            via: dep
                .package_id
                .as_deref()
                .map(|package_id| via_dependency(parsed, package_id, &predecessors))
                .unwrap_or_else(|| "unknown".to_string()),
        })
        .sorted_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)))
        .collect();

    DependencySets {
        declared: parsed.declared_deps.clone(),
        compiled: parsed.compiled_deps.clone(),
        delta,
    }
}

fn dependency_identity(dep: &DependencyInfo) -> String {
    dep.package_id
        .clone()
        .unwrap_or_else(|| format!("unresolved:{}:{}", dep.kind_key(), dep.package_name))
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

fn via_dependency(
    parsed: &ParsedMetadata,
    target: &str,
    predecessors: &HashMap<String, String>,
) -> String {
    predecessors
        .get(target)
        .filter(|parent| *parent != &parsed.root_package_id)
        .and_then(|parent| {
            parsed
                .direct_dep_names
                .get(parent)
                .cloned()
                .or_else(|| parsed.package_names.get(parent).cloned())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

trait DependencyInfoExt {
    fn kind_key(&self) -> &'static str;
}

impl DependencyInfoExt for DependencyInfo {
    fn kind_key(&self) -> &'static str {
        match self.kind {
            crate::metadata::DependencyKind::Normal => "normal",
            crate::metadata::DependencyKind::Development => "development",
            crate::metadata::DependencyKind::Build => "build",
        }
    }
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
