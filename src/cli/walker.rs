//! Shared directory walker for CLI commands.
//! Uses the `ignore` crate to respect `.gitignore`, `.hsmlignore`,
//! and `--ignore-pattern` flags while walking directories for `.hsml` files.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;

/// Directory names that are always skipped during traversal.
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
/// Built-in ignores can be re-included via `.hsmlignore` using `!pattern`
/// (e.g. `!build/` to re-include the `build` directory).
///
/// IO errors during traversal are collected but do not prevent other files
/// from being returned.
pub fn walk_hsml_files(dir: &Path, ignore_patterns: &[String]) -> Result<WalkResult, String> {
    let mut builder = WalkBuilder::new(dir);
    builder
        .add_custom_ignore_filename(".hsmlignore")
        .require_git(false)
        .follow_links(false);

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

    let reincluded = load_reinclude_patterns(dir);

    let mut files = Vec::new();
    let mut errors = Vec::new();

    for entry in builder.build() {
        match entry {
            Ok(entry) => {
                let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                let path = entry.path();
                if is_file
                    && path.extension().is_some_and(|ext| ext == "hsml")
                    && !is_builtin_ignored(path, &reincluded)
                {
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

/// Parse `.hsmlignore` in the root directory for re-include patterns (`!pattern`).
/// Returns the set of directory names that should NOT be treated as built-in ignores.
fn load_reinclude_patterns(dir: &Path) -> HashSet<String> {
    let Ok(content) = std::fs::read_to_string(dir.join(".hsmlignore")) else {
        return HashSet::new();
    };
    content
        .lines()
        .filter_map(|line| line.strip_prefix('!'))
        .map(|name| name.trim_end_matches('/').to_string())
        .collect()
}

/// Check if any path component matches a built-in ignored directory name,
/// unless that name has been re-included via `.hsmlignore`.
fn is_builtin_ignored(path: &Path, reincluded: &HashSet<String>) -> bool {
    path.ancestors().any(|ancestor| {
        ancestor.file_name().is_some_and(|name| {
            BUILTIN_IGNORES
                .iter()
                .any(|&ig| name == OsStr::new(ig) && !reincluded.contains(ig))
        })
    })
}
