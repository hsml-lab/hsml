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
