use crate::metadata::{dependency_key, DependencyInfo, ParsedMetadata};
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencySets {
    pub declared: Vec<DependencyInfo>,
    pub compiled: Vec<DependencyInfo>,
    pub delta: Vec<DeltaEntry>,
    pub orphaned: Vec<DependencyInfo>,
    pub summary: Summary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeltaEntry {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub via: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub declared_count: usize,
    pub compiled_count: usize,
    pub delta_count: usize,
    pub orphaned_count: usize,
}

pub fn compute_sets(parsed: &ParsedMetadata) -> DependencySets {
    let declared_ids = parsed
        .declared_dep_ids
        .iter()
        .filter_map(|id| id.as_ref())
        .collect::<HashSet<_>>();
    let compiled_ids = parsed
        .compiled_deps
        .iter()
        .filter_map(|dep| dep_package_id(parsed, dep))
        .collect::<HashSet<_>>();
    let predecessors = shortest_predecessors(parsed);

    let mut delta = parsed
        .compiled_deps
        .iter()
        .filter(|dep| dep_package_id(parsed, dep).is_some_and(|id| !declared_ids.contains(id)))
        .map(|dep| DeltaEntry {
            name: dep.name.clone(),
            version: dep.version.clone(),
            source: dep.source.clone(),
            via: via_dependency(parsed, &predecessors, dep),
        })
        .collect::<Vec<_>>();
    delta.sort_by(|a, b| {
        a.name.cmp(&b.name).then_with(|| {
            a.version
                .as_deref()
                .unwrap_or("")
                .cmp(b.version.as_deref().unwrap_or(""))
                .then_with(|| a.source.cmp(&b.source))
        })
    });

    let mut orphaned = parsed
        .declared_deps
        .iter()
        .zip(parsed.declared_dep_ids.iter())
        .filter(|(_, package_id)| {
            package_id
                .as_ref()
                .map(|id| !compiled_ids.contains(id))
                .unwrap_or(true)
        })
        .map(|(dep, _)| dep.clone())
        .collect::<Vec<_>>();
    orphaned.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));

    let summary = Summary {
        declared_count: parsed.declared_deps.len(),
        compiled_count: parsed.compiled_deps.len(),
        delta_count: delta.len(),
        orphaned_count: orphaned.len(),
    };

    DependencySets {
        declared: parsed.declared_deps.clone(),
        compiled: parsed.compiled_deps.clone(),
        delta,
        orphaned,
        summary,
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

fn via_dependency(
    parsed: &ParsedMetadata,
    predecessors: &HashMap<String, String>,
    dep: &DependencyInfo,
) -> String {
    let Some(package_id) = dep_package_id(parsed, dep) else {
        return "unknown".to_string();
    };
    let Some(predecessor_id) = predecessors.get(package_id) else {
        return "unknown".to_string();
    };

    parsed
        .package_names
        .get(predecessor_id)
        .cloned()
        .unwrap_or_else(|| "unknown".to_string())
}

fn dep_package_id<'a>(parsed: &'a ParsedMetadata, dep: &DependencyInfo) -> Option<&'a String> {
    parsed.compiled_dep_ids.get(&dependency_key(
        &dep.name,
        dep.version.as_deref(),
        dep.source.as_deref(),
    ))
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

    output.push_str(&format!("\n~ orphaned ({})\n", sets.orphaned.len()));
    if sets.orphaned.is_empty() {
        output.push_str("  none\n");
    } else {
        for dep in &sets.orphaned {
            output.push_str(&format!(
                "  {} {}\n",
                dep.name,
                dep.version.as_deref().unwrap_or("unknown")
            ));
        }
    }

    output
}

pub fn format_json(sets: &DependencySets) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&sets)
}
