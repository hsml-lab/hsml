use std::path::PathBuf;

use clap::ArgMatches;

/// Resolve the target path from CLI arguments, falling back to the current directory.
pub fn resolve_path(matches: &ArgMatches) -> Result<PathBuf, String> {
    match matches.get_one::<PathBuf>("path") {
        Some(p) => Ok(p.clone()),
        None => {
            std::env::current_dir().map_err(|e| format!("Unable to get current directory: {e}"))
        }
    }
}

/// Resolve the target path from CLI arguments (required, no fallback).
pub fn resolve_required_path(matches: &ArgMatches) -> PathBuf {
    matches.get_one::<PathBuf>("path").unwrap().clone()
}

/// Collect `--ignore-pattern` values from CLI arguments.
pub fn resolve_ignore_patterns(matches: &ArgMatches) -> Vec<String> {
    matches
        .get_many::<String>("ignore_pattern")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default()
}

/// Validate that a path has the `.hsml` extension.
pub fn validate_hsml_extension(path: &std::path::Path) -> Result<(), String> {
    path.extension()
        .filter(|&ext| ext == "hsml")
        .ok_or("File must have .hsml extension".to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests;
