# Contributing

Thank you for your interest in contributing to HSML!

## Getting Started

1. Fork and clone the repository
2. Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed
3. Run `cargo build` to verify everything compiles

## Useful Cargo Commands

### Build

```sh
cargo build
```

### Format

```sh
cargo fmt --all
```

### Lint

```sh
cargo clippy --all-targets --all-features
```

### Test

```sh
cargo test
```

#### Snapshot tests

Some tests use [`insta`](https://insta.rs) snapshots instead of inline `assert_eq!`.
We reach for snapshots when the expected value is large or tedious to maintain by hand:

- Parsed ASTs (`tests/parse.rs`) — snapshotted as JSON with source locations redacted,
  so these tests stay focused on structure. Location correctness is asserted separately
  in the validator tests.
- Long compiled HTML output (the bigger cases in `tests/compiler.rs`).
- Multi-line rendered diagnostics (`src/diagnostic/format/tests/default.rs`).

Short, behavior-defining checks stay as plain `assert_eq!` — they read as documentation
and force you to think about the exact expected value.

When a snapshot test fails (or you add a new one), review and accept the change with
[`cargo-insta`](https://insta.rs/docs/cli/) (`cargo install cargo-insta`):

```sh
cargo insta test --review   # run tests, then interactively review pending snapshots
cargo insta accept          # accept all pending snapshots without review
```

Always eyeball the diff before accepting — a snapshot is only as trustworthy as the
review that produced it. Committed snapshots live in `**/snapshots/*.snap`.

### Run

HSML provides several subcommands:

#### Compile an HSML file to HTML

```sh
cargo run -- compile example.hsml
```

#### Parse an HSML file and output the AST

```sh
cargo run -- parse example.hsml
```

#### Format an HSML file

```sh
cargo run -- fmt example.hsml
```

#### Check an HSML file for errors

```sh
cargo run -- check example.hsml
```

#### Start the LSP server

```sh
cargo run -- lsp
```

### Code Coverage

Generates an HTML coverage report in the `coverage` directory. Requires [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov).

```sh
cargo llvm-cov --html --output-dir coverage
```
