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
