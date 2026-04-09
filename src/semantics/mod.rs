/// Semantic analysis module for cargo-declared
///
/// This module adds tagging, confidence scoring, and structural insights
/// to dependency analysis, turning it from a "checker" into a "lens".

use crate::metadata::{DependencyInfo, ParsedMetadata};
use serde::Serialize;

// ============== Types ==============

#[derive(Debug, Clone, Serialize)]
pub struct SemanticDependency {
    pub crate_: String,
    pub declared: bool,
    pub used: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SemanticOutput {
    pub package: String,
    pub dependencies: Vec<SemanticDependency>,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub total_declared: usize,
    pub total_used: usize,
    pub unused_count: usize,
    pub dead_weight_score: f64,
    pub core_coverage: f64,
}

// ============== Tag Constants ==============

pub mod tags {
    pub const DEAD_WEIGHT: &str = "dead_weight";
    pub const CORE: &str = "core";
    pub const OPTIONAL: &str = "optional";
    pub const DEV_ONLY: &str = "dev_only";
    pub const BUILD_ONLY: &str = "build_only";
    pub const EXPERIMENTAL: &str = "experimental";
}

// ============== Analysis Functions ==============

/// Infer semantic tags for a dependency based on its properties
pub fn infer_tags(dep: &DependencyInfo, parsed: &ParsedMetadata) -> Vec<String> {
    let mut tags = vec![];

    // Check if declared but not used (dead weight)
    let is_used = parsed.compiled_deps.iter().any(|c| c.name == dep.name);
    if !is_used {
        tags.push(tags::DEAD_WEIGHT.to_string());
    }

    // Check dependency kind
    match dep.kind {
        crate::metadata::DependencyKind::Development => {
            tags.push(tags::DEV_ONLY.to_string());
        }
        crate::metadata::DependencyKind::Build => {
            tags.push(tags::BUILD_ONLY.to_string());
        }
        crate::metadata::DependencyKind::Normal => {
            if is_used {
                tags.push(tags::CORE.to_string());
            } else {
                tags.push(tags::OPTIONAL.to_string());
            }
        }
    }

    // Check for path-based dependencies (local workspace members)
    if dep.source.as_ref().is_some_and(|s| s.starts_with("path+")) {
        tags.push(tags::EXPERIMENTAL.to_string());
    }

    tags
}

/// Find source file locations where a dependency is used (placeholder for AST integration)
fn find_locations(_dep: &DependencyInfo, _parsed: &ParsedMetadata) -> Vec<String> {
    // This will be enhanced with actual source analysis when we integrate nu_rust_ast
    vec![]
}

/// Analyze dependencies and assign semantic tags
pub fn analyze_semantics(parsed: &ParsedMetadata) -> Vec<SemanticDependency> {
    let mut deps = vec![];

    // First pass: declared dependencies
    for decl in &parsed.declared_deps {
        let used = parsed.compiled_deps.iter().any(|c| c.name == decl.name);

        let tags = infer_tags(decl, parsed);
        let confidence = if used { Some(0.95) } else { Some(0.85) };

        deps.push(SemanticDependency {
            crate_: decl.name.clone(),
            declared: true,
            used,
            locations: vec![],
            confidence,
            tags,
        });
    }

    // Add compiled-only deps (transitive dependencies)
    for compiled in &parsed.compiled_deps {
        if !parsed.declared_deps.iter().any(|d| d.name == compiled.name) {
            deps.push(SemanticDependency {
                crate_: compiled.name.clone(),
                declared: false,
                used: true,
                locations: vec![],
                confidence: Some(1.0),
                tags: vec![tags::CORE.to_string()],
            });
        }
    }

    deps
}

/// Generate summary statistics from semantic analysis
pub fn generate_summary(deps: &[SemanticDependency]) -> Summary {
    let total_declared = deps.iter().filter(|d| d.declared).count();
    let total_used = deps.iter().filter(|d| d.used).count();
    let unused_count = deps.iter().filter(|d| d.declared && !d.used).count();

    // Dead weight score: ratio of declared-but-unused to total declared
    let dead_weight_score = if total_declared > 0 {
        unused_count as f64 / total_declared as f64
    } else {
        0.0
    };

    Summary {
        total_declared,
        total_used,
        unused_count,
        dead_weight_score,
        core_coverage: 1.0,
    }
}
