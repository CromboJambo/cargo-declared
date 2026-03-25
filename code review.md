## Assistant

Welcome to Corust Agent.

Co-build with an experienced Rust partner.

Your cwd is `/home/crombo/cargo-declared`.

Use `/init` to generate `AGENTS.md`.

Use `/usage` to check token quota.

Need help? Just ask.

## User

[@cargo-declared](file:///home/crombo/cargo-declared)
 give this a slop grade. you can be harsh it is heavly gen AI

## Assistant

Let me start by reading the project files to understand the codebase.

**Tool Call: git status --porcelain**
Status: Completed

Terminal:
```
{
  "conflicts": [],
  "staged": [
    {
      "file": "README.md",
      "status": "modified"
    },
    {
      "file": "src/delta.rs",
      "status": "modified"
    },
    {
      "file": "src/lib.rs",
      "status": "modified"
    },
    {
      "file": "src/metadata.rs",
      "status": "modified"
    },
    {
      "file": "src/output.rs",
      "status": "modified"
    }
  ],
  "unstaged": [],
  "untracked": []
}
```


**Tool Call: Read /home/crombo/cargo-declared**
Status: Completed

Read 1 lines

```txt
Toolset error: ToolCallError: ToolCallError: ToolCallError: Path is not a file: /home/crombo/cargo-declared
```

**Tool Call: Read /home/crombo/cargo-declared/src/lib.rs**
Status: Completed

Read 56 lines

```rs
pub mod delta;
pub mod error;
pub mod metadata;
pub mod output;

pub use crate::error::Error;
use crate::metadata::ParsedMetadata;
use crate::output::{display_human, display_json, validate_invariant as validate_parsed_invariant};

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

pub fn validate_invariant(path: Option<std::path::PathBuf>) -> Result<bool, Error> {
    let parsed = parse_metadata(path)?;
    Ok(validate_parsed_invariant(&parsed))
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

```

**Tool Call: Read /home/crombo/cargo-declared/Cargo.toml**
Status: Completed

Read 21 lines

```toml
[package]
name = "cargo-declared"
version = "0.1.2"
edition = "2021"
description = "Audit the gap between declared and compiled dependencies"
license = "MIT OR Apache-2.0"
repository = "https://github.com/CromboJambo/cargo-declared"
keywords = ["cargo", "dependencies", "audit", "declared", "tooling"]
categories = ["development-tools::cargo-plugins"]
readme = "README.md"

[dependencies]
cargo_metadata = "0.18"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
cargo_metadata = "0.18"
tempfile = "3"

```

**Tool Call: Read /home/crombo/cargo-declared/src/delta.rs**
Status: Completed

Read 188 lines

```rs
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

```

**Tool Call: Read /home/crombo/cargo-declared/src/metadata.rs**
Status: Completed

Read 246 lines

```rs
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
    pub declared_dep_ids: Vec<Option<String>>,
    pub compiled_dep_ids: HashMap<String, String>,
}

pub fn parse_metadata(path: Option<PathBuf>) -> Result<ParsedMetadata> {
    let metadata = load_metadata(path.as_deref())?;
    let resolve = metadata.resolve.as_ref().ok_or(Error::NoRootPackage)?;
    let root_id = resolve.root.as_ref().ok_or(Error::NoRootPackage)?;
    let root_pkg = find_package(&metadata, root_id).ok_or(Error::NoRootPackage)?;

    let package_names: HashMap<_, _> = metadata
        .packages
        .iter()
        .map(|pkg| (pkg.id.to_string(), pkg.name.clone()))
        .collect();

    let root_dep_ids: HashMap<_, _> = resolve
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

    let declared_deps: Vec<_> = root_pkg
        .dependencies
        .iter()
        .map(|dep| map_declared_dep(dep))
        .collect();

    let declared_dep_ids = root_pkg
        .dependencies
        .iter()
        .map(|dep| root_dep_ids.get(dependency_display_name(dep)).cloned())
        .collect();
    let compiled_dep_ids = metadata
        .packages
        .iter()
        .map(|pkg| {
            let version = pkg.version.to_string();
            let source = pkg.source.as_ref().map(ToString::to_string);
            (
                dependency_key(&pkg.name, Some(&version), source.as_deref()),
                pkg.id.to_string(),
            )
        })
        .collect();

    Ok(ParsedMetadata {
        workspace_root: metadata.workspace_root.clone().into(),
        package_name: root_pkg.name.clone(),
        root_package_id: root_id.to_string(),
        declared_deps,
        compiled_deps: collect_compiled_deps(&metadata, root_id),
        package_graph: build_package_graph(&metadata),
        package_names,
        declared_dep_ids,
        compiled_dep_ids,
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

fn map_declared_dep(dep: &cargo_metadata::Dependency) -> DependencyInfo {
    DependencyInfo {
        name: dependency_display_name(dep).to_string(),
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
        })
        .collect()
}

pub(crate) fn dependency_key(name: &str, version: Option<&str>, source: Option<&str>) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{}",
        name,
        version.unwrap_or("unknown"),
        source.unwrap_or("")
    )
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

```

**Tool Call: Read /home/crombo/cargo-declared/src/output.rs**
Status: Completed

Read 17 lines

```rs
use crate::error::Error;
use crate::metadata::ParsedMetadata;

pub fn display_human(parsed: &ParsedMetadata) -> Result<String, Error> {
    let sets = crate::delta::compute_sets(parsed);
    Ok(crate::delta::format_human(&sets))
}

pub fn display_json(parsed: &ParsedMetadata) -> Result<String, Error> {
    let sets = crate::delta::compute_sets(parsed);
    Ok(crate::delta::format_json(&sets)?)
}

pub fn validate_invariant(parsed: &ParsedMetadata) -> bool {
    let sets = crate::delta::compute_sets(parsed);
    sets.compiled.len() == sets.declared.len() - sets.orphaned.len() + sets.delta.len()
}

```

**Tool Call: Read /home/crombo/cargo-declared/src/error.rs**
Status: Completed

Read 24 lines

```rs
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Path does not exist: {path}")]
    PathNotFound { path: PathBuf },

    #[error("Failed to run cargo metadata: {source}")]
    CargoMetadata {
        #[from]
        source: cargo_metadata::Error,
    },

    #[error("Cargo metadata did not identify a root package")]
    NoRootPackage,

    #[error("Failed to serialize JSON output: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

```

**Tool Call: find /home/crombo/cargo-declared -name "*.rs" | sort**
Status: Completed

Terminal:
```
/home/crombo/cargo-declared/fixtures/direct/src/lib.rs
/home/crombo/cargo-declared/fixtures/multi-version/shared-0.1.0/src/lib.rs
/home/crombo/cargo-declared/fixtures/multi-version/shared-0.2.0/src/lib.rs
/home/crombo/cargo-declared/fixtures/src/lib.rs
/home/crombo/cargo-declared/fixtures/transitive/src/lib.rs
/home/crombo/cargo-declared/fixtures/unused/src/lib.rs
/home/crombo/cargo-declared/src/delta.rs
/home/crombo/cargo-declared/src/error.rs
/home/crombo/cargo-declared/src/lib.rs
/home/crombo/cargo-declared/src/main.rs
/home/crombo/cargo-declared/src/metadata.rs
/home/crombo/cargo-declared/src/output.rs
/home/crombo/cargo-declared/target/debug/build/serde_core-7d0f0a539f33282f/out/private.rs
/home/crombo/cargo-declared/target/debug/build/serde-e59866aa333d97d3/out/private.rs
/home/crombo/cargo-declared/target/debug/build/thiserror-daae1c51f482a6df/out/private.rs
/home/crombo/cargo-declared/target/release/build/serde-be35a5fc8ccfbf10/out/private.rs
/home/crombo/cargo-declared/target/release/build/serde_core-03be5aa6f83e4fbc/out/private.rs
/home/crombo/cargo-declared/target/release/build/thiserror-5912b9328e617fc1/out/private.rs
/home/crombo/cargo-declared/tests/integration.rs

```


**Tool Call: Read /home/crombo/cargo-declared/README.md**
Status: Completed

Read 116 lines

```md
# cargo-declared

[![Crates.io](https://img.shields.io/crates/v/cargo-declared.svg)](https://crates.io/crates/cargo-declared)
[![Docs.rs](https://docs.rs/cargo-declared/badge.svg)](https://docs.rs/cargo-declared)
[![License](https://img.shields.io/crates/l/cargo-declared.svg)](LICENSE)

Audit the gap between declared and compiled dependencies.

## What it does

Answers two closely related questions:

- What compiled that you did not explicitly ask for?
- What did you declare that did not compile?

```text
cargo-declared v0.1.2

declared:  5
compiled:  47
delta:     42

+ transitive (42)
  syn        1.0.109  via: clap
  quote      1.0.44   via: clap
  ...

~ orphaned (0)
  none
```

`delta` is the resolved transitive set. `orphaned` is the declared set that did not resolve into the compiled graph, which is most often optional or inactive dependencies.

## What it is not

- Not a vulnerability scanner (`cargo audit` does that)
- Not a license checker (`cargo deny` does that)
- Not a bloat analyzer (`cargo bloat` does that)
- Not an unused dependency finder (`cargo machete` does that)

Every feature request gets measured against this list.

## Install

```bash
cargo install cargo-declared
```

## Usage

```bash
cargo declared        # human readable
cargo declared --json # machine readable, pipe friendly
cargo declared --path /path/to/Cargo.toml
cargo declared --path /path/to/workspace-member
```

## JSON output

`--json` prints a single object with these top-level keys:

- `declared`
- `compiled`
- `delta`
- `orphaned`
- `summary`

Example:

```json
{
  "declared": [
    {
      "name": "clap",
      "version": "^4",
      "source": "registry+https://github.com/rust-lang/crates.io-index",
      "kind": "normal"
    }
  ],
  "compiled": [
    {
      "name": "clap",
      "version": "4.6.0",
      "source": "registry+https://github.com/rust-lang/crates.io-index",
      "kind": "normal"
    },
    {
      "name": "clap_builder",
      "version": "4.6.0",
      "source": "registry+https://github.com/rust-lang/crates.io-index",
      "kind": "normal"
    }
  ],
  "delta": [
    {
      "name": "clap_builder",
      "version": "4.6.0",
      "source": "registry+https://github.com/rust-lang/crates.io-index",
      "via": "clap"
    }
  ],
  "orphaned": [],
  "summary": {
    "declared_count": 1,
    "compiled_count": 2,
    "delta_count": 1,
    "orphaned_count": 0
  }
}
```

## Pipe into your audit log

```bash
cargo declared --json | mirror-log add --source cargo-declared
```

```

## Assistant

...

**Tool Call: Read /home/crombo/cargo-declared/src/main.rs**
Status: Completed

Read 51 lines

```rs
use cargo_declared::{CargoDeclared, Error};
use clap::Parser;
use std::path::PathBuf;

/// Audit the gap between declared and compiled dependencies
#[derive(Parser, Debug)]
#[command(name = "cargo-declared")]
#[command(about = "Audit the gap between declared and compiled dependencies", long_about = None)]
struct Cli {
    /// Path to the Cargo.toml file
    #[arg(short, long, value_name = "PATH")]
    path: Option<PathBuf>,

    /// Output in JSON format
    #[arg(short, long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.json {
        true => run_json(cli.path),
        false => run_human(cli.path),
    };

    match result {
        Ok(output) => {
            println!("{}", output);
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_human(path: Option<PathBuf>) -> Result<String, Error> {
    let tool = path.map_or_else(CargoDeclared::new, |path| {
        CargoDeclared::new().with_path(path)
    });
    tool.run_human()
}

fn run_json(path: Option<PathBuf>) -> Result<String, Error> {
    let tool = path.map_or_else(CargoDeclared::new, |path| {
        CargoDeclared::new().with_path(path)
    });
    tool.run_json()
}

```

**Tool Call: Read /home/crombo/cargo-declared/tests/integration.rs**
Status: Completed

Read 554 lines

```rs
use cargo_declared::{
    compute_and_display_human, compute_and_display_json, parse_metadata, validate_invariant,
};
use std::fs;
use tempfile::TempDir;

fn write_manifest(dir: &TempDir) -> std::path::PathBuf {
    let root = dir.path();
    fs::create_dir_all(root.join("direct")).unwrap();
    fs::create_dir_all(root.join("direct/src")).unwrap();
    fs::create_dir_all(root.join("transitive")).unwrap();
    fs::create_dir_all(root.join("transitive/src")).unwrap();
    fs::create_dir_all(root.join("transitive2")).unwrap();
    fs::create_dir_all(root.join("transitive2/src")).unwrap();
    fs::create_dir_all(root.join("unused")).unwrap();
    fs::create_dir_all(root.join("unused/src")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[dependencies]
direct = { path = "direct" }
unused = { path = "unused", optional = true }
"#,
    )
    .unwrap();

    fs::write(root.join("src/lib.rs"), "pub fn root() {}\n").unwrap();

    fs::write(
        root.join("direct/Cargo.toml"),
        r#"
[package]
name = "direct"
version = "0.1.0"
edition = "2021"

[dependencies]
transitive = { path = "../transitive" }
"#,
    )
    .unwrap();

    fs::write(root.join("direct/src/lib.rs"), "pub fn direct() {}\n").unwrap();

    fs::write(
        root.join("transitive/Cargo.toml"),
        r#"
[package]
name = "transitive"
version = "0.1.0"
edition = "2021"

[dependencies]
transitive2 = { path = "../transitive2" }
"#,
    )
    .unwrap();

    fs::write(
        root.join("transitive/src/lib.rs"),
        "pub fn transitive() {}\n",
    )
    .unwrap();

    fs::write(
        root.join("transitive2/Cargo.toml"),
        r#"
[package]
name = "transitive2"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        root.join("transitive2/src/lib.rs"),
        "pub fn transitive2() {}\n",
    )
    .unwrap();

    fs::write(
        root.join("unused/Cargo.toml"),
        r#"
[package]
name = "unused"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(root.join("unused/src/lib.rs"), "pub fn unused() {}\n").unwrap();

    root.join("Cargo.toml")
}

fn write_renamed_manifest(dir: &TempDir) -> std::path::PathBuf {
    let root = dir.path();
    fs::create_dir_all(root.join("dep/src")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "rename-root"
version = "0.1.0"
edition = "2021"

[dependencies]
serde_alias = { package = "dep-pkg", path = "dep" }
"#,
    )
    .unwrap();

    fs::write(root.join("src/lib.rs"), "pub fn root() {}\n").unwrap();

    fs::write(
        root.join("dep/Cargo.toml"),
        r#"
[package]
name = "dep-pkg"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(root.join("dep/src/lib.rs"), "pub fn dep() {}\n").unwrap();

    root.join("Cargo.toml")
}

fn write_build_manifest(dir: &TempDir) -> std::path::PathBuf {
    let root = dir.path();
    fs::create_dir_all(root.join("normal/src")).unwrap();
    fs::create_dir_all(root.join("builddep/src")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "build-root"
version = "0.1.0"
edition = "2021"

[dependencies]
normal = { path = "normal" }

[build-dependencies]
builddep = { path = "builddep" }
"#,
    )
    .unwrap();

    fs::write(root.join("src/lib.rs"), "pub fn root() {}\n").unwrap();

    fs::write(
        root.join("normal/Cargo.toml"),
        r#"
[package]
name = "normal"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(root.join("normal/src/lib.rs"), "pub fn normal() {}\n").unwrap();

    fs::write(
        root.join("builddep/Cargo.toml"),
        r#"
[package]
name = "builddep"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(root.join("builddep/src/lib.rs"), "pub fn builddep() {}\n").unwrap();

    root.join("Cargo.toml")
}

fn write_multi_version_manifest(dir: &TempDir) -> std::path::PathBuf {
    let root = dir.path();
    fs::create_dir_all(root.join("shared-0.1.0/src")).unwrap();
    fs::create_dir_all(root.join("shared-0.2.0/src")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "multi-version-test"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "shared-0.1.0", version = "0.1.0" }
shared_v2 = { package = "shared", path = "shared-0.2.0", version = "0.2.0" }
"#,
    )
    .unwrap();

    fs::write(root.join("src/lib.rs"), "pub fn root() {}\n").unwrap();

    fs::write(
        root.join("shared-0.1.0/Cargo.toml"),
        r#"
[package]
name = "shared"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        root.join("shared-0.1.0/src/lib.rs"),
        "pub fn shared_0_1_0() {}\n",
    )
    .unwrap();

    fs::write(
        root.join("shared-0.2.0/Cargo.toml"),
        r#"
[package]
name = "shared"
version = "0.2.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        root.join("shared-0.2.0/src/lib.rs"),
        "pub fn shared_0_2_0() {}\n",
    )
    .unwrap();

    root.join("Cargo.toml")
}

#[test]
fn test_delta_computation() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_manifest(&temp_dir);
    let result = compute_and_display_human(Some(cargo_toml_path)).unwrap();

    assert!(result.contains("declared:"));
    assert!(result.contains("compiled:"));
    assert!(result.contains("delta:"));
    assert!(result.contains("transitive"));
}

#[test]
fn test_json_output_validity() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_manifest(&temp_dir);
    let result = compute_and_display_json(Some(cargo_toml_path)).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert!(json_value.is_object());
    assert!(json_value.get("declared").is_some());
    assert!(json_value.get("compiled").is_some());
    assert!(json_value.get("delta").is_some());
    assert!(json_value.get("orphaned").is_some());
    assert!(json_value.get("summary").is_some());

    let summary = json_value.get("summary").unwrap();
    assert!(summary.get("declared_count").is_some());
    assert!(summary.get("compiled_count").is_some());
    assert!(summary.get("delta_count").is_some());
    assert!(summary.get("orphaned_count").is_some());
}

#[test]
fn test_invariant_holds() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_manifest(&temp_dir);
    assert!(validate_invariant(Some(cargo_toml_path)).unwrap());
}

#[test]
fn test_parse_metadata_accepts_manifest_path() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_manifest(&temp_dir);
    let parsed = parse_metadata(Some(cargo_toml_path)).unwrap();

    assert_eq!(parsed.package_name, "test-package");
    assert!(parsed.declared_deps.iter().any(|dep| dep.name == "direct"));
}

#[test]
fn test_transitive_dependency_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_manifest(&temp_dir);
    let result = compute_and_display_human(Some(cargo_toml_path)).unwrap();

    // Verify that both transitive dependencies are detected
    assert!(result.contains("transitive"));
    assert!(result.contains("transitive2"));

    // Verify that transitive dependencies are listed in delta section
    assert!(result.contains("transitive"));
    assert!(result.contains("transitive2"));

    // Verify that transitive dependencies have proper via information
    assert!(result.contains("via: direct"));
    assert!(result.contains("via: transitive"));
}

#[test]
fn test_renamed_dependency_is_not_delta_or_orphaned() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_renamed_manifest(&temp_dir);
    let result = compute_and_display_human(Some(cargo_toml_path)).unwrap();

    assert!(result.contains("declared:  1"));
    assert!(result.contains("compiled:  1"));
    assert!(result.contains("delta:     0"));
    assert!(!result.contains("dep-pkg 0.1.0 via:"));
    assert!(!result.contains("serde_alias"));
}

#[test]
fn test_compiled_dependency_kinds_are_preserved() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_build_manifest(&temp_dir);
    let result = compute_and_display_json(Some(cargo_toml_path)).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&result).unwrap();
    let compiled = json_value.get("compiled").unwrap().as_array().unwrap();

    let normal = compiled
        .iter()
        .find(|dep| dep.get("name").unwrap() == "normal");
    let builddep = compiled
        .iter()
        .find(|dep| dep.get("name").unwrap() == "builddep");

    assert_eq!(normal.unwrap().get("kind").unwrap(), "normal");
    assert_eq!(builddep.unwrap().get("kind").unwrap(), "build");
}

#[test]
fn test_multi_version_package_name_collision() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_multi_version_manifest(&temp_dir);
    let json_result = compute_and_display_json(Some(cargo_toml_path)).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_result).unwrap();
    let compiled = json_value.get("compiled").unwrap().as_array().unwrap();

    // Both versions should be compiled
    let shared_0_1_0 = compiled
        .iter()
        .find(|dep| dep.get("name").unwrap() == "shared" && dep.get("version").unwrap() == "0.1.0");
    let shared_0_2 = compiled
        .iter()
        .find(|dep| dep.get("name").unwrap() == "shared" && dep.get("version").unwrap() == "0.2.0");

    assert!(shared_0_1_0.is_some(), "shared 0.1.0 should be compiled");
    assert!(shared_0_2.is_some(), "shared 0.2.0 should be compiled");

    let orphaned = json_value.get("orphaned").unwrap().as_array().unwrap();
    let orphaned_names: Vec<_> = orphaned.iter().filter_map(|dep| dep.get("name")).collect();

    assert!(
        !orphaned_names.iter().any(|name| *name == "shared"),
        "shared should not be orphaned"
    );
    assert!(
        !orphaned_names.iter().any(|name| *name == "shared_v2"),
        "shared_v2 should not be orphaned"
    );

    // Both should be declared
    let declared = json_value.get("declared").unwrap().as_array().unwrap();
    let declared_names: Vec<_> = declared.iter().filter_map(|dep| dep.get("name")).collect();

    assert!(
        declared_names.iter().any(|name| *name == "shared"),
        "shared should be declared"
    );
    assert!(
        declared_names.iter().any(|name| *name == "shared_v2"),
        "shared_v2 should be declared"
    );
}

fn write_multi_version_transitive_manifest(dir: &TempDir) -> std::path::PathBuf {
    let root = dir.path();
    fs::create_dir_all(root.join("shared-0.1.0/src")).unwrap();
    fs::create_dir_all(root.join("shared-0.2.0/src")).unwrap();
    fs::create_dir_all(root.join("transitive-a/src")).unwrap();
    fs::create_dir_all(root.join("transitive-b/src")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "multi-version-transitive-test"
version = "0.1.0"
edition = "2021"

[dependencies]
transitive-a = { path = "transitive-a" }
transitive-b = { path = "transitive-b" }
"#,
    )
    .unwrap();

    fs::write(root.join("src/lib.rs"), "pub fn root() {}\n").unwrap();

    fs::write(
        root.join("shared-0.1.0/Cargo.toml"),
        r#"
[package]
name = "shared"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        root.join("shared-0.1.0/src/lib.rs"),
        "pub fn shared_0_1_0() {}\n",
    )
    .unwrap();

    fs::write(
        root.join("shared-0.2.0/Cargo.toml"),
        r#"
[package]
name = "shared"
version = "0.2.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        root.join("shared-0.2.0/src/lib.rs"),
        "pub fn shared_0_2_0() {}\n",
    )
    .unwrap();

    fs::write(
        root.join("transitive-a/Cargo.toml"),
        r#"
[package]
name = "transitive-a"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "../shared-0.1.0", version = "0.1.0" }
"#,
    )
    .unwrap();

    fs::write(
        root.join("transitive-a/src/lib.rs"),
        "pub fn transitive_a() {}\n",
    )
    .unwrap();

    fs::write(
        root.join("transitive-b/Cargo.toml"),
        r#"
[package]
name = "transitive-b"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "../shared-0.2.0", version = "0.2.0" }
"#,
    )
    .unwrap();

    fs::write(
        root.join("transitive-b/src/lib.rs"),
        "pub fn transitive_b() {}\n",
    )
    .unwrap();

    root.join("Cargo.toml")
}

#[test]
fn test_multi_version_transitive_chain() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_multi_version_transitive_manifest(&temp_dir);
    let json_result = compute_and_display_json(Some(cargo_toml_path)).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_result).unwrap();
    let compiled = json_value.get("compiled").unwrap().as_array().unwrap();

    // Both shared versions should be compiled (even though they have different transitive dependencies)
    let shared_0_1_0 = compiled
        .iter()
        .find(|dep| dep.get("name").unwrap() == "shared" && dep.get("version").unwrap() == "0.1.0");
    let shared_0_2 = compiled
        .iter()
        .find(|dep| dep.get("name").unwrap() == "shared" && dep.get("version").unwrap() == "0.2.0");

    assert!(shared_0_1_0.is_some(), "shared 0.1.0 should be compiled");
    assert!(shared_0_2.is_some(), "shared 0.2.0 should be compiled");

    let orphaned = json_value.get("orphaned").unwrap().as_array().unwrap();
    let orphaned_names: Vec<_> = orphaned.iter().filter_map(|dep| dep.get("name")).collect();

    // Neither shared version should be orphaned
    assert!(
        !orphaned_names.iter().any(|name| *name == "shared"),
        "shared should not be orphaned"
    );
    assert!(
        !orphaned_names.iter().any(|name| *name == "shared_v2"),
        "shared_v2 should not be orphaned"
    );

    let delta = json_value.get("delta").unwrap().as_array().unwrap();

    assert!(
        delta.iter().any(|dep| {
            dep.get("name").unwrap() == "shared"
                && dep.get("version").unwrap() == "0.1.0"
                && dep.get("via").unwrap() == "transitive-a"
        }),
        "shared 0.1.0 should be delta via transitive-a"
    );
    assert!(
        delta.iter().any(|dep| {
            dep.get("name").unwrap() == "shared"
                && dep.get("version").unwrap() == "0.2.0"
                && dep.get("via").unwrap() == "transitive-b"
        }),
        "shared 0.2.0 should be delta via transitive-b"
    );
}

```

**Tool Call: (native) cargo test (in /home/crombo/cargo-declared)**
Status: Completed

Terminal:
```

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 9 tests
test test_multi_version_transitive_chain ... ok
test test_delta_computation ... ok
test test_renamed_dependency_is_not_delta_or_orphaned ... ok
test test_multi_version_package_name_collision ... ok
test test_parse_metadata_accepts_manifest_path ... ok
test test_invariant_holds ... ok
test test_compiled_dependency_kinds_are_preserved ... ok
test test_transitive_dependency_tracking ... ok
test test_json_output_validity ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running unittests src/lib.rs (target/debug/deps/cargo_declared-124893c87b97012a)
     Running unittests src/main.rs (target/debug/deps/cargo_declared-fdfd29b6c6e1df8e)
     Running tests/integration.rs (target/debug/deps/integration-79cb787b6146fecc)
   Doc-tests cargo_declared

```


**Tool Call: (native) cargo clippy (in /home/crombo/cargo-declared)**
Status: Completed

Terminal:
```
    Checking cargo-declared v0.1.2 (/home/crombo/cargo-declared)
warning: redundant closure
  --> src/metadata.rs:66:14
   |
66 |         .map(|dep| map_declared_dep(dep))
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: replace the closure with the function itself: `map_declared_dep`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#redundant_closure
   = note: `#[warn(clippy::redundant_closure)]` on by default

warning: `cargo-declared` (lib) generated 1 warning (run `cargo clippy --fix --lib -p cargo-declared` to apply 1 suggestion)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s

```


## Assistant

Here's the slop grade.

---

## cargo-declared — Slop Grade

**Overall: B− / "Competent Slop"**

The project works. Tests pass, clippy has exactly one warning (a trivial redundant closure). The core idea is legitimate and scoped. But the gen-AI fingerprints are everywhere, and several design choices are actively bad.

---

### What's Fine

- **Tests pass, 9/9.** The test fixtures are the strongest part — they cover renamed deps, multi-version collisions, build deps, transitive chains. Someone (or something) thought about edge cases.
- **Error handling is clean.** `thiserror` used properly, no `.unwrap()` in library code, `PathNotFound` is a real error variant instead of a string.
- **`dependency_key` using `\x1F` (Unit Separator) as a delimiter** is a genuinely good choice for a composite key. Not a slop move.
- **BFS for shortest-path predecessor** in `shortest_predecessors` is correct and appropriate.

---

### The Slop

**1. `output.rs` is a useless indirection layer (the biggest red flag)**

```rust
// output.rs
pub fn display_human(parsed: &ParsedMetadata) -> Result<String, Error> {
    let sets = crate::delta::compute_sets(parsed);
    Ok(crate::delta::format_human(&sets))
}
```

This file is pure AI padding. It exists because the model wanted a module called `output`, not because there's any logic here. `display_human` is a one-liner that calls two other functions. `validate_invariant` is a one-liner that lives in the wrong module entirely. None of this earns its own file.

**2. `lib.rs` is a second useless indirection layer on top of the first**

```rust
pub fn compute_and_display_human(path: Option<std::path::PathBuf>) -> Result<String, Error> {
    let parsed = parse_metadata(path)?;
    display_human(&parsed)
}
```

`lib.rs` wraps `output.rs` which wraps `delta.rs`. Three layers of one-liners. The `CargoDeclared` builder struct exists purely to give the AI something "OOP-flavored" to generate. `CargoDeclared::new().with_path(p).run_human()` does the same thing as `compute_and_display_human(Some(p))`. Pick one.

**3. `main.rs` has `run_human` and `run_json` as free functions for no reason**

```rust
fn run_human(path: Option<PathBuf>) -> Result<String, Error> {
    let tool = path.map_or_else(CargoDeclared::new, |path| {
        CargoDeclared::new().with_path(path)
    });
    tool.run_human()
}
```

This is a 5-line function that calls a 2-line function that calls a 1-line function. The entire `main.rs` could be `CargoDeclared::new().with_path_opt(cli.path).run()` if the design wasn't fractured across three layers.

**4. `validate_invariant` is a lie**

```rust
pub fn validate_invariant(parsed: &ParsedMetadata) -> bool {
    let sets = crate::delta::compute_sets(parsed);
    sets.compiled.len() == sets.declared.len() - sets.orphaned.len() + sets.delta.len()
}
```

This is exposed as a public API function and tested with `assert!(validate_invariant(...))`. What does it mean when it returns `false`? Nothing is done about it. It's not checked anywhere in the actual tool. It's a "we should validate something" gesture that the AI added to feel thorough, with no actual use.

**5. The test assertions are shallow and redundant**

```rust
// test_transitive_dependency_tracking
assert!(result.contains("transitive"));
assert!(result.contains("transitive2"));

// Verify that transitive dependencies are listed in delta section
assert!(result.contains("transitive"));  // <-- exact duplicate
assert!(result.contains("transitive2")); // <-- exact duplicate
```

Literal duplicate `assert!` calls with a comment above them claiming they test something different. Classic AI filler. The tests check string containment on human-readable output rather than structured data — fragile and hard to maintain.

**6. `[dev-dependencies]` duplicates `[dependencies]`**

```toml
[dependencies]
cargo_metadata = "0.18"

[dev-dependencies]
cargo_metadata = "0.18"
```

`cargo_metadata` is already a normal dependency. Listing it again in `dev-dependencies` does nothing. The AI added it because the integration tests use it and it didn't think to check if it was already available.

**7. The `CargoDeclared` builder has no reason to exist as a struct**

There's no state being accumulated. There's no deferred execution benefit. It's a builder with one optional field. The AI generated it because "builder pattern" sounds like good API design.

---

### Summary Table

| Area | Grade | Notes |
|---|---|---|
| Core logic (BFS, delta computation) | B+ | Correct, readable |
| Error handling | A− | Clean and complete |
| Module structure | D | Three layers of one-liners |
| Public API design | D+ | Builder that shouldn't exist, useless indirection |
| Tests — coverage | B | Good edge cases covered |
| Tests — quality | C− | Duplicate asserts, string-sniffing, shallow |
| `validate_invariant` | F | Exposed, tested, never used for anything |
| `dev-dependencies` | D | Duplicate of normal dep |

The code isn't broken. It's just the shape of code that was generated to *look* like a well-structured project rather than one that was *designed* as one.

