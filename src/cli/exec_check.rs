use std::{fs, path::Path, time::Instant};

use clap::ArgMatches;
use hsml::check_content;

use super::common::{resolve_ignore_patterns, resolve_path, validate_hsml_extension};
use super::diagnostics::{
    FileDiagnostics, has_errors, print_summary, render_diagnostics, resolve_colors,
};
use super::walker::walk_hsml_files;

pub fn exec_check(matches: &ArgMatches) -> Result<(), String> {
    let format = matches
        .get_one::<String>("report_format")
        .map(|s| s.as_str());
    let debug = matches.get_flag("debug");
    let (no_color, _dim) = resolve_colors(matches);
    let ignore_patterns = resolve_ignore_patterns(matches);
    let path = resolve_path(matches)?;
    let path = &path;

    let mut results: Vec<FileDiagnostics> = Vec::new();
    let mut io_errors: Vec<String> = Vec::new();
    let mut file_count: usize = 0;

    let total_start = Instant::now();

    if path.is_dir() {
        match walk_hsml_files(path, &ignore_patterns) {
            Ok(result) => {
                file_count = result.files.len();
                io_errors.extend(result.errors);
                for file in &result.files {
                    if let Err(e) = collect_file(file, &mut results) {
                        io_errors.push(e);
                    }
                }
            }
            Err(e) => io_errors.push(e),
        }
    } else if path.is_file() {
        file_count = 1;
        if let Err(e) = collect_file(path, &mut results) {
            io_errors.push(e);
        }
    } else {
        return Err("Path must be a file or directory".to_string());
    }

    // Always render diagnostics before reporting IO errors
    render_diagnostics(&results, format, no_color);

    if debug {
        print_summary(
            &results,
            file_count,
            io_errors.len(),
            total_start.elapsed(),
            no_color,
            "checked",
        );
    }

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

    validate_hsml_extension(file)?;

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
