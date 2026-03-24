//! Shared directory walker for CLI commands.
//! Uses the `ignore` crate to respect `.gitignore`, `.hsmlignore`,
//! and `--ignore-pattern` flags while walking directories for `.hsml` files.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;

/// Directories that are always skipped during traversal.
const BUILTIN_IGNORES: &[&str] = &[
    "node_modules/",
    "target/",
    "dist/",
    "build/",
    ".git/",
    ".hg/",
    ".svn/",
];

/// Walk a directory and collect all `.hsml` file paths, respecting ignore rules.
///
/// Automatically respects:
/// - Built-in ignores (`node_modules`, `target`, `dist`, `build`, `.git`, `.hg`, `.svn`)
/// - `.gitignore` files (even outside git repositories)
/// - `.hsmlignore` files (same format as `.gitignore`)
/// - `--ignore-pattern` globs passed via CLI
///
/// Hidden files/directories are skipped by default.
pub fn walk_hsml_files(dir: &Path, ignore_patterns: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut builder = WalkBuilder::new(dir);
    builder
        .add_custom_ignore_filename(".hsmlignore")
        .require_git(false)
        .follow_links(false);

    let mut overrides = OverrideBuilder::new(dir);

    for pattern in BUILTIN_IGNORES {
        overrides
            .add(&format!("!{pattern}"))
            .map_err(|e| format!("Failed to build built-in ignore rules: {e}"))?;
    }

    for pattern in ignore_patterns {
        overrides
            .add(&format!("!{pattern}"))
            .map_err(|e| format!("Invalid ignore pattern '{pattern}': {e}"))?;
    }

    let overrides = overrides
        .build()
        .map_err(|e| format!("Failed to build ignore patterns: {e}"))?;
    builder.overrides(overrides);

    let mut files = Vec::new();
    let mut errors = Vec::new();

    for entry in builder.build() {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "hsml") {
                    files.push(path.to_path_buf());
                }
            }
            Err(e) => {
                errors.push(format!("{e}"));
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    Ok(files)
}
