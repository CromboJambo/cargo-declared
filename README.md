# cargo-declared

Audit the gap between declared and compiled dependencies.

## What it does

Answers one question: **what compiled that you didn't explicitly ask for?**
```
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
cargo declared              # delta only, human readable
cargo declared --json       # machine readable, pipe friendly
cargo declared --full       # all four sets
```

## Pipe into your audit log
```bash
cargo declared --json | mirror-log add --source cargo-declared
```
