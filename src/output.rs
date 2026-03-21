use crate::delta::DependencySets;
use crate::error::Error;
use crate::metadata::ParsedMetadata;
use serde_json::Value;

/// Format output for human-readable display
pub fn display_human(parsed: &ParsedMetadata) -> Result<String, Error> {
    use crate::delta::compute_sets;
    use crate::delta::format_human;

    let sets = compute_sets(
        &parsed.declared_deps,
        &parsed.compiled_deps,
        &parsed.dependencies,
    );

    Ok(format_human(&sets))
}

/// Format output for JSON
pub fn display_json(parsed: &ParsedMetadata) -> Result<String, Error> {
    use crate::delta::compute_sets;
    use crate::delta::format_json;

    let sets = compute_sets(
        &parsed.declared_deps,
        &parsed.compiled_deps,
        &parsed.dependencies,
    );

    format_json(&sets)
}

/// Validate that the invariant holds: declared + delta = compiled
pub fn validate_invariant(parsed: &ParsedMetadata) -> Result<bool, Error> {
    use crate::delta::compute_sets;

    let sets = compute_sets(
        &parsed.declared_deps,
        &parsed.compiled_deps,
        &parsed.dependencies,
    );

    // Check: declared_count + delta_count == compiled_count
    let invariant_holds = sets.declared.len() + sets.delta.len() == sets.compiled.len();

    Ok(invariant_holds)
}

/// Display the invariant validation result
pub fn display_invariant(invariant_holds: bool) -> String {
    if invariant_holds {
        format!("Invariant holds: declared + delta = compiled ✓")
    } else {
        format!("Invariant violated: declared + delta ≠ compiled ✗")
    }
}
