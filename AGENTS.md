# Repository Guidelines

## Project Overview

`cargo-declared` is a Cargo plugin that audits the gap between the dependencies declared in `Cargo.toml` and those actually compiled into the binary. It reports declared, compiled, delta (transitive-only), and orphaned dependencies in both human-readable and JSON formats.

---

## Project Structure

```
cargo-declared/
├── src/
│   ├── main.rs       # CLI entry point (clap-based argument parsing)
│   ├── lib.rs        # Public API surface; re-exports CargoDeclared and Error
│   ├── metadata.rs   # Parses Cargo metadata into ParsedMetadata
│   ├── delta.rs      # Computes the diff between declared and compiled deps
│   ├── output.rs     # Formats results as human-readable text or JSON
│   └── error.rs      # Unified Error type (thiserror)
├── tests/
│   └── integration.rs  # All integration tests using tempfile workspaces
├── .github/workflows/
│   └── rust.yml        # CI: build + test on push/PR to main
└── Cargo.toml
```

---

## Build, Test, and Development Commands

| Command | Description |
|---|---|
| `cargo build` | Compile the project in debug mode |
| `cargo build --release` | Compile with optimizations |
| `cargo test` | Run all unit and integration tests |
| `cargo check` | Fast type-check without producing artifacts |
| `cargo clippy` | Lint the codebase for common mistakes |
| `cargo fmt` | Auto-format all source files |
| `cargo fmt --check` | Verify formatting without making changes (used in CI) |

To test the CLI locally against the current workspace:

```sh
cargo run -- --path /path/to/project/Cargo.toml
cargo run -- --path /path/to/project/Cargo.toml --json
```

---

## Coding Style & Naming Conventions

- **Formatter**: `rustfmt` with default settings. All code must be formatted before committing.
- **Linter**: `clippy` — address all warnings before opening a PR.
- **Naming**: Follow standard Rust conventions — `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- **Errors**: Use `thiserror` for structured error types. Do not use `.unwrap()` in library code (`src/`); reserve it for tests.
- **Modules**: Keep each module focused on a single concern (parsing, diffing, formatting). Avoid cross-module side effects.

---

## Testing Guidelines

- **Framework**: Rust's built-in `#[test]` with `tempfile` for isolated workspace fixtures.
- **Location**: All integration tests live in `tests/integration.rs`. Unit tests may be added inline in `src/` modules using `#[cfg(test)]`.
- **Fixtures**: Build temporary Cargo workspaces in tests using `TempDir` — do not rely on the repository's own `Cargo.toml` as a test fixture.
- **Naming**: Test functions must be descriptive and prefixed with `test_`, e.g., `test_renamed_dependency_is_not_delta_or_orphaned`.
- **Coverage**: Every new feature or bug fix must be accompanied by a test that would have caught the regression.

---

## Commit & Pull Request Guidelines

- **Commit messages**: Use short, imperative-mood subject lines (e.g., `Add JSON output format`, `Fix multi-version transitive tracking`). No ticket prefix is required.
- **Scope**: Keep commits focused — one logical change per commit.
- **PRs**: Target the `main` branch. Include a clear description of what changed and why. Link any relevant issues.
- **CI**: All PRs must pass the GitHub Actions workflow (build + tests) before merging.
