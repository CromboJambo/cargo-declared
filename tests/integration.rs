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
shared-0.2 = { path = "shared-0.2.0", version = "0.2.0" }
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

    println!("Result: {}", result);

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
    let _result = compute_and_display_human(Some(cargo_toml_path.clone())).unwrap();
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
        !orphaned_names.iter().any(|name| *name == "shared-0.2"),
        "shared-0.2 should not be orphaned"
    );

    // Both should be declared
    let declared = json_value.get("declared").unwrap().as_array().unwrap();
    let declared_names: Vec<_> = declared.iter().filter_map(|dep| dep.get("name")).collect();

    assert!(
        declared_names.iter().any(|name| *name == "shared"),
        "shared should be declared"
    );
    assert!(
        declared_names.iter().any(|name| *name == "shared-0.2"),
        "shared-0.2 should be declared"
    );
}
