use std::{env, fs, path::PathBuf};

use clap::ArgMatches;
use hsml::{
    check_content,
    diagnostic::{
        Diagnostic, Severity,
        format::{DiagnosticFormatter, default::DefaultFormatter, json::JsonFormatter},
    },
};

/// Collected diagnostics with the source content they came from.
struct FileResult {
    diagnostics: Vec<Diagnostic>,
    source: String,
}

pub fn exec_check(matches: &ArgMatches) -> Result<(), String> {
    let format = matches
        .get_one::<String>("report_format")
        .map(|s| s.as_str());

    let path = match matches.get_one::<PathBuf>("path") {
        Some(p) => p.clone(),
        None => env::current_dir().map_err(|e| format!("Unable to get current directory: {e}"))?,
    };
    let path = &path;

    let mut results: Vec<FileResult> = Vec::new();

    if path.is_dir() {
        collect_hsml_files_in_dir(path, &mut results)?;
    } else if path.is_file() {
        collect_file(path, &mut results)?;
    } else {
        return Err("Path must be a file or directory".to_string());
    }

    // Gather all diagnostics
    let all_diagnostics: Vec<&Diagnostic> =
        results.iter().flat_map(|r| r.diagnostics.iter()).collect();

    if all_diagnostics.is_empty() {
        return Ok(());
    }

    // Render output
    match format {
        Some("json") => {
            // JSON: single array of all diagnostics
            let owned: Vec<_> = all_diagnostics.into_iter().cloned().collect();
            let output = JsonFormatter.format(&owned, None);
            eprintln!("{output}");
        }
        _ => {
            // Default: render per-file with source context
            for result in &results {
                if !result.diagnostics.is_empty() {
                    eprint!(
                        "{}",
                        DefaultFormatter.format(&result.diagnostics, Some(&result.source))
                    );
                }
            }
        }
    }

    let has_errors = results
        .iter()
        .flat_map(|r| r.diagnostics.iter())
        .any(|d| d.severity == Severity::Error);

    if has_errors {
        Err(String::new())
    } else {
        Ok(())
    }
}

fn collect_file(file: &PathBuf, results: &mut Vec<FileResult>) -> Result<(), String> {
    if !file.exists() {
        return Err("File does not exist".to_string());
    }

    if !file.is_file() {
        return Err("Given file must be a file".to_string());
    }

    file.extension()
        .filter(|&ext| ext == "hsml")
        .ok_or("File must have .hsml extension".to_string())?;

    let content = fs::read_to_string(file)
        .map_err(|e| format!("Unable to read file {}: {e}", file.display()))?;

    let diagnostics: Vec<_> = check_content(&content)
        .into_iter()
        .map(|d| d.with_file_path(file.display().to_string()))
        .collect();

    results.push(FileResult {
        diagnostics,
        source: content,
    });

    Ok(())
}

fn collect_hsml_files_in_dir(dir: &PathBuf, results: &mut Vec<FileResult>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|e| format!("Unable to read directory {}: {e}", dir.display()))?
    {
        let entry = entry
            .map_err(|e| format!("Unable to read directory entry in {}: {e}", dir.display()))?;

        // Skip symlinks to prevent infinite recursion from circular links
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Unable to read file type in {}: {e}", dir.display()))?;
        if file_type.is_symlink() {
            continue;
        }

        let path = entry.path();

        if path.is_dir() {
            collect_hsml_files_in_dir(&path, results)?;
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "hsml") {
            collect_file(&path, results)?;
        }
    }

    Ok(())
}
