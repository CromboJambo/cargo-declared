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

pub fn validate_invariant(parsed: &ParsedMetadata) -> Result<bool, Error> {
    let sets = crate::delta::compute_sets(parsed);
    let invariant_holds = sets.declared.len() + sets.delta.len() == sets.compiled.len();
    Ok(invariant_holds)
}

pub fn display_invariant(invariant_holds: bool) -> String {
    if invariant_holds {
        "Invariant holds: declared + delta = compiled".to_string()
    } else {
        "Invariant violated: declared + delta != compiled".to_string()
    }
}
