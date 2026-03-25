```
 ██████╗ █████╗ ██████╗  ██████╗  ██████╗                       
██╔════╝██╔══██╗██╔══██╗██╔════╝ ██╔═══██╗                      
██║     ███████║██████╔╝██║  ███╗██║   ██║█████╗                
██║     ██╔══██║██╔══██╗██║   ██║██║   ██║╚════╝                
╚██████╗██║  ██║██║  ██║╚██████╔╝╚██████╔╝                      
 ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝  ╚═════╝                       
                                                                
██████╗ ███████╗ ██████╗██╗      █████╗ ██████╗ ███████╗██████╗ 
██╔══██╗██╔════╝██╔════╝██║     ██╔══██╗██╔══██╗██╔════╝██╔══██╗
██║  ██║█████╗  ██║     ██║     ███████║██████╔╝█████╗  ██║  ██║
██║  ██║██╔══╝  ██║     ██║     ██╔══██║██╔══██╗██╔══╝  ██║  ██║
██████╔╝███████╗╚██████╗███████╗██║  ██║██║  ██║███████╗██████╔╝
╚═════╝ ╚══════╝ ╚═════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═════╝ 
```                                                                
[![Crates.io](https://img.shields.io/crates/v/cargo-declared.svg)](https://crates.io/crates/cargo-declared)
[![Docs.rs](https://docs.rs/cargo-declared/badge.svg)](https://docs.rs/cargo-declared)
[![License](https://img.shields.io/crates/l/cargo-declared.svg)](LICENSE)

Audit the gap between declared and compiled dependencies.

## What it does

Answers two closely related questions:

- What compiled that you did not explicitly ask for?
- What did you declare that did not compile?

```text

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
