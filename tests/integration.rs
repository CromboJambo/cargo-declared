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
"#,
    )
    .unwrap();

    fs::write(
        root.join("transitive/src/lib.rs"),
        "pub fn transitive() {}\n",
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
fn test_orphaned_detection() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_manifest(&temp_dir);
    let result = compute_and_display_human(Some(cargo_toml_path)).unwrap();

    assert!(result.contains("orphaned"));
    assert!(result.contains("unused"));
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
    assert!(!validate_invariant(Some(cargo_toml_path)).unwrap());
}

#[test]
fn test_parse_metadata_accepts_manifest_path() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = write_manifest(&temp_dir);
    let parsed = parse_metadata(Some(cargo_toml_path)).unwrap();

    assert_eq!(parsed.package_name, "test-package");
    assert!(parsed.declared_deps.iter().any(|dep| dep.name == "direct"));
}
