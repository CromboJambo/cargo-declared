# agent.md — cargo-declared

## What This Tool Is

`cargo-declared` surfaces the delta between what a Rust project *declares* in `Cargo.toml`
and what actually compiles into the binary. It demultiplies the dependency graph — making
the implicit explicit and the invisible auditable.

### The Architectural Parallel

The GNU Hurd project spent 35 years failing to ship a multiserver microkernel OS. The
architecture was sound: isolate kernel services into userspace servers communicating over
typed IPC. The design was never wrong. C was wrong. Every IPC boundary that should have
been a compile-time contract was instead a runtime vulnerability. The correct idea was
executed in a language that couldn't enforce its own invariants.

Redox OS is the existence proof that the Hurd theorists were right all along — they were
just 40 years early for a language that could implement it. Rust's ownership model *is*
the IPC contract enforcer. The compiler rejects the class of bugs that made the boundary
guarantees unenforceable.

`cargo-declared` operates on the same structural insight applied to dependency graphs. A
Cargo workspace is a trust boundary. Features flags are IPC channels. Transitive
dependencies are implicit contracts nobody signed. The declared graph and the compiled
graph diverge silently — exactly like Hurd servers drifting from their protocol
definitions with no enforcement layer catching it.

This tool is the enforcement layer. It makes the drift visible.

### The Manufacturing Analogy

In cost accounting, a BOM (bill of materials) multiplicity error means a component gets
counted N times through recursive flattening — producing cost variances that are
untraceable unless you demultiply the tree first. The compiled dependency graph is a BOM.
`cargo-declared` is the demultiplier. The delta it surfaces is the variance.

---

## Project Principles

- **Legibility over cleverness.** Data flow should be traceable by reading the code once.
- **Explicit over implicit.** The tool exists to surface hidden contracts; the
  implementation shouldn't create new ones.
- **Partition → transform → reduce → reindex → repeat.** Each pipeline stage should be
  independently testable and named.
- **`.unwrap()` and `.clone()` are intentional prototyping choices, not debt.** Mark
  sites with `// TODO: propagate` when hardening is the right next step.
- **Community amplification, not extraction.** The delta this tool surfaces should make
  maintainers and funding links more visible, not strip-mine them.

---

## Agent Setup

### Model Runner

`cargo-declared` is best served by a capable code model with strong tool-call reliability.
Recommended setup:

- **LM Studio** serving over its OpenAI-compatible endpoint (`http://localhost:1234/v1`)
- Models with reliable tool-call support: Qwen2.5-Coder-32B, Devstral, or any model
  where LM Studio ships its own jinja tool-call template (Q4_K_M quantizations tend to
  work; Q5 variants often don't — the failure is in the chat template, not the model)
- For remote inference from a workstation: point `api_url` at your Tailscale address

### Zed Agent Panel

Zed has a first-party LM Studio provider. In `settings.json`:

```json
{
  "language_models": {
    "lmstudio": {
      "api_url": "http://<tailscale-ip>:1234"
    }
  },
  "agent": {
    "default_model": {
      "provider": "lmstudio",
      "model": "qwen2.5-coder-32b-instruct"
    }
  }
}
```

Zed owns the tool dispatch loop (file edits, terminal, codebase search). Let it. Your job
is model selection and making sure the model emits valid tool-call JSON. If Zed shows
"No tools" for the selected model, switch quantization or model before debugging anything
else.

Extend the agent's tool surface via MCP servers in Zed's Agent Panel settings — useful
for adding `cargo` subcommand runners or custom audit scripts as tools.

### Rust Harness (Autonomous Loop)

For running agents outside Zed — batch analysis, automated delta reporting, CI
integration — use **Rig** (`rig-core` on crates.io). It speaks the OpenAI-compatible
protocol natively, so LM Studio is a zero-config drop-in:

```toml
[dependencies]
rig-core = "0.x"
tokio = { version = "1", features = ["full"] }
```

```rust
use rig::providers::openai::Client;

let client = Client::from_url("openai-key-unused", "http://localhost:1234/v1");
let agent = client
    .agent("qwen2.5-coder-32b-instruct")
    .preamble(include_str!("prompts/audit.md"))
    .build();
```

Rig is thin enough that you stay in control of the loop — no hidden state, no framework
magic. Fits the explicit data flow principle above.

---

## Agent Levels

These levels describe how much autonomy to grant an agent working on this codebase. You
should *feel* when you've outgrown a level — don't wait to be told.

### Level 1 — Read Only

The agent reads code, explains behavior, answers questions about the dependency graph
logic. No file writes. Safe to run on any branch.

**Exit signal:** You're asking it to make a change you already know how to describe
precisely. Stop describing, start letting it write.

### Level 2 — Guided Edits

The agent proposes and applies changes to a single module at a time. You review each
diff before moving to the next. Runs `cargo check` and `cargo test` after each edit
and surfaces failures before proceeding.

Appropriate for: adding a new output format, refactoring a pipeline stage, extending
the delta detection logic.

**Exit signal:** You're approving diffs without reading them because the pattern is
obvious and correct. The bottleneck is you, not the agent.

### Level 3 — Autonomous with Checkpoints

The agent owns a scoped task end-to-end: takes a goal, plans steps, executes, runs the
full test suite, and surfaces a summary diff for final review. It should stop and ask if
it hits an ambiguity that would require a design decision.

Appropriate for: implementing a fully-specced feature, performing a targeted refactor
across multiple files, generating documentation from code.

**Behavioral exits (agent should halt and surface state):**
- `cargo test` fails after two self-correction attempts
- A change would affect the public API surface
- A dependency would be added or removed
- The task scope has expanded beyond the original description

---

## Codebase Orientation

| Concern | Where to look |
|---|---|
| Dependency graph resolution | `src/resolve.rs` |
| Declared vs compiled delta | `src/delta.rs` |
| Output formatting | `src/output/` |
| CLI entry point | `src/main.rs` |
| Test fixtures | `tests/fixtures/` |

When in doubt: `cargo declared --help`, then read `src/main.rs` top to bottom. The data
flow is linear by design.

---

## Vocabulary

- **multiplicity error** — a dependency counted N times through recursive graph
  flattening; the root cause of untraceable cost variances in both BOMs and Cargo graphs
- **demultiplying** — the process of collapsing the compiled graph to its unique
  contributing edges and comparing against declared intent
- **delta** — the difference between the declared dependency set and the compiled
  dependency set; the primary output of this tool
- **declared graph** — what `Cargo.toml` says should be there
- **compiled graph** — what `cargo metadata` says actually got resolved and built
