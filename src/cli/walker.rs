//! Shared directory walker for CLI commands.
//! Uses the `ignore` crate to respect `.gitignore`, `.hsmlignore`,
//! and `--ignore-pattern` flags while walking directories for `.hsml` files.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;

/// Directories that are always skipped during traversal.
/// These can be re-included via `.hsmlignore` (e.g. `!build/`).
const BUILTIN_IGNORES: &[&str] = &["node_modules", "target", "dist", "build", ".hg", ".svn"];

/// Collected `.hsml` file paths with any IO errors encountered during traversal.
pub struct WalkResult {
    pub files: Vec<PathBuf>,
    pub errors: Vec<String>,
}

/// Walk a directory and collect all `.hsml` file paths, respecting ignore rules.
///
/// Automatically respects:
/// - Built-in ignores (`node_modules`, `target`, `dist`, `build`, `.hg`, `.svn`)
/// - `.gitignore` files (even outside git repositories)
/// - `.hsmlignore` files (same format as `.gitignore`)
/// - `--ignore-pattern` globs passed via CLI
///
/// Hidden files/directories are skipped by default.
///
/// Precedence (highest to lowest):
/// 1. `--ignore-pattern` flags (overrides, cannot be re-included)
/// 2. `.hsmlignore` files (can re-include built-in ignores with `!pattern`)
/// 3. `.gitignore` files
/// 4. Built-in ignores
///
/// IO errors during traversal are collected but do not prevent other files
/// from being returned.
pub fn walk_hsml_files(dir: &Path, ignore_patterns: &[String]) -> Result<WalkResult, String> {
    let mut builder = WalkBuilder::new(dir);
    builder
        .add_custom_ignore_filename(".hsmlignore")
        .require_git(false)
        .follow_links(false);

    // Built-in ignores are loaded from a temporary file so they sit at a lower
    // priority than `.hsmlignore`. This lets users re-include them with `!pattern`.
    let builtin_content = BUILTIN_IGNORES.join("\n");
    let builtin_path = std::env::temp_dir().join("hsml-builtin-ignore");
    std::fs::write(&builtin_path, &builtin_content)
        .map_err(|e| format!("Failed to write built-in ignore rules: {e}"))?;
    if let Some(err) = builder.add_ignore(&builtin_path) {
        return Err(format!("Failed to load built-in ignore rules: {err}"));
    }

    // --ignore-pattern flags are overrides (highest priority, cannot be re-included)
    if !ignore_patterns.is_empty() {
        let mut overrides = OverrideBuilder::new(dir);
        for pattern in ignore_patterns {
            overrides
                .add(&format!("!{pattern}"))
                .map_err(|e| format!("Invalid ignore pattern '{pattern}': {e}"))?;
        }
        let overrides = overrides
            .build()
            .map_err(|e| format!("Failed to build ignore patterns: {e}"))?;
        builder.overrides(overrides);
    }

    let mut files = Vec::new();
    let mut errors = Vec::new();

    for entry in builder.build() {
        match entry {
            Ok(entry) => {
                let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                let path = entry.path();
                if is_file && path.extension().is_some_and(|ext| ext == "hsml") {
                    files.push(path.to_path_buf());
                }
            }
            Err(e) => {
                errors.push(format!("{e}"));
            }
        }
    }

    Ok(WalkResult { files, errors })
}
