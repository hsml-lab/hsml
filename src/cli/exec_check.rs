use std::{
    env, fs,
    path::{Path, PathBuf},
};

use clap::ArgMatches;
use hsml::check_content;

use super::diagnostics::{FileDiagnostics, has_errors, render_diagnostics};

pub fn exec_check(matches: &ArgMatches) -> Result<(), String> {
    let format = matches
        .get_one::<String>("report_format")
        .map(|s| s.as_str());

    let path = match matches.get_one::<PathBuf>("path") {
        Some(p) => p.clone(),
        None => env::current_dir().map_err(|e| format!("Unable to get current directory: {e}"))?,
    };
    let path = &path;

    let mut results: Vec<FileDiagnostics> = Vec::new();
    let mut io_errors: Vec<String> = Vec::new();

    if path.is_dir() {
        if let Err(e) = collect_hsml_files_in_dir(path, &mut results) {
            io_errors.push(e);
        }
    } else if path.is_file() {
        if let Err(e) = collect_file(path, &mut results) {
            io_errors.push(e);
        }
    } else {
        return Err("Path must be a file or directory".to_string());
    }

    // Always render diagnostics before reporting IO errors
    render_diagnostics(&results, format);

    if !io_errors.is_empty() {
        Err(io_errors.join("\n"))
    } else if has_errors(&results) {
        Err(String::new())
    } else {
        Ok(())
    }
}

fn collect_file(file: &Path, results: &mut Vec<FileDiagnostics>) -> Result<(), String> {
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

    results.push(FileDiagnostics {
        diagnostics,
        source: content,
    });

    Ok(())
}

fn collect_hsml_files_in_dir(dir: &Path, results: &mut Vec<FileDiagnostics>) -> Result<(), String> {
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
