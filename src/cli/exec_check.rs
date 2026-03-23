use std::{env, fs, path::PathBuf};

use clap::ArgMatches;
use hsml::{
    check_content,
    diagnostic::{
        Severity,
        format::{DiagnosticFormatter, default::DefaultFormatter, json::JsonFormatter},
    },
};

pub fn exec_check(matches: &ArgMatches) -> Result<(), String> {
    let path = matches.get_one::<PathBuf>("path");
    let format = matches
        .get_one::<String>("report_format")
        .map(|s| s.as_str());

    let fallback_path =
        env::current_dir().map_err(|e| format!("Unable to get current directory: {e}"))?;
    let path = path.unwrap_or(&fallback_path);

    if path.is_dir() {
        check_hsml_files_in_dir(path, format)
    } else if path.is_file() {
        check_file(path, format)
    } else {
        Err("Path must be a file or directory".to_string())
    }
}

fn format_diagnostics(
    diagnostics: &[hsml::diagnostic::Diagnostic],
    source: &str,
    format: Option<&str>,
) -> String {
    match format {
        Some("json") => JsonFormatter.format(diagnostics, Some(source)),
        _ => DefaultFormatter.format(diagnostics, Some(source)),
    }
}

fn check_file(file: &PathBuf, format: Option<&str>) -> Result<(), String> {
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

    if !diagnostics.is_empty() {
        let has_errors = diagnostics.iter().any(|d| d.severity == Severity::Error);
        eprint!("{}", format_diagnostics(&diagnostics, &content, format));

        if has_errors {
            return Err(String::new());
        }
    }

    Ok(())
}

fn check_hsml_files_in_dir(dir: &PathBuf, format: Option<&str>) -> Result<(), String> {
    let mut has_diagnostic_errors = false;

    for entry in
        fs::read_dir(dir).map_err(|e| format!("Unable to read directory {}: {e}", dir.display()))?
    {
        let entry = entry
            .map_err(|e| format!("Unable to read directory entry in {}: {e}", dir.display()))?;
        let path = entry.path();

        if path.is_dir() {
            match check_hsml_files_in_dir(&path, format) {
                Ok(()) => {}
                Err(e) if e.is_empty() => has_diagnostic_errors = true,
                Err(e) => return Err(e),
            }
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "hsml") {
            match check_file(&path, format) {
                Ok(()) => {}
                Err(e) if e.is_empty() => has_diagnostic_errors = true,
                Err(e) => return Err(e),
            }
        }
    }

    if has_diagnostic_errors {
        Err(String::new())
    } else {
        Ok(())
    }
}
