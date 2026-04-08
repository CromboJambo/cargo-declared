use crate::metadata::ParsedMetadata;
use crate::output::validate_invariant as validate_parsed_invariant;

/// Audit the dependency graph and produce a structured report
pub async fn audit_graph(parsed: &ParsedMetadata) -> Result<AuditReport, AuditError> {
    let declared_count = parsed.declared_deps.len();
    let compiled_count = parsed.compiled_deps.len();
    
    // Validate the invariant
    if !validate_parsed_invariant(parsed) {
        return Err(AuditError::InvariantViolation {
            declared: declared_count,
            compiled: compiled_count,
        });
    }

    Ok(AuditReport {
        package: parsed.package_name.clone(),
        declared_count,
        compiled_count,
        delta_count: compiled_count - declared_count + count_orphaned(parsed),
        orphaned_count: count_orphaned(parsed),
        transitive_deps: parsed.compiled_deps.iter()
            .filter(|dep| !parsed.declared_deps.iter().any(|d| d.name == dep.name))
            .map(|dep| dep.name.clone())
            .collect(),
    })
}

fn count_orphaned(parsed: &ParsedMetadata) -> usize {
    parsed.declared_deps.iter()
        .filter(|declared| {
            !parsed.compiled_deps.iter().any(|compiled| compiled.name == declared.name)
        })
        .count()
}

#[derive(Debug)]
pub enum AuditError {
    InvariantViolation { declared: usize, compiled: usize },
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::InvariantViolation { declared, compiled } => {
                write!(f, "Invariant violated: declared={} compiled={}", declared, compiled)
            }
        }
    }
}

impl std::error::Error for AuditError {}

#[derive(Debug)]
pub struct AuditReport {
    pub package: String,
    pub declared_count: usize,
    pub compiled_count: usize,
    pub delta_count: usize,
    pub orphaned_count: usize,
    pub transitive_deps: Vec<String>,
}
