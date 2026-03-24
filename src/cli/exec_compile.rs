use std::{env, fs, path::PathBuf};

use clap::ArgMatches;
use hsml::{
    compile_content_diagnostics,
    diagnostic::{
        Diagnostic,
        format::{DiagnosticFormatter, default::DefaultFormatter, json::JsonFormatter},
    },
};

/// Result of compiling a single file.
struct CompileFileResult {
    diagnostics: Vec<Diagnostic>,
    source: String,
    out_file: PathBuf,
    html: Option<String>,
}

pub fn exec_compile(matches: &ArgMatches) -> Result<(), String> {
    let path = matches.get_one::<PathBuf>("path");
    let out = matches.get_one::<PathBuf>("output");
    let format = matches
        .get_one::<String>("report_format")
        .map(|s| s.as_str());
    let is_json = format == Some("json");

    let fallback_path =
        env::current_dir().map_err(|e| format!("Unable to get current directory: {e}"))?;
    let path = path.unwrap_or(&fallback_path);

    if !is_json {
        println!("Compiling...");
    }

    let mut results: Vec<CompileFileResult> = Vec::new();

    if path.is_dir() {
        collect_compile_dir(path, &mut results)?;
    } else if path.is_file() {
        collect_compile_file(path, out, &mut results)?;
    } else {
        return Err("Path must be a file or directory".to_string());
    }

    // Write HTML for successful compilations
    let mut has_errors = false;
    for result in &results {
        if let Some(html) = &result.html {
            fs::write(&result.out_file, html)
                .map_err(|e| format!("Unable to write file {}: {e}", result.out_file.display()))?;

            if !is_json {
                println!(
                    "Compiled HTML written to {} successfully",
                    result.out_file.display()
                );
            }
        } else {
            has_errors = true;
        }
    }

    // Render diagnostics
    let all_diagnostics: Vec<&Diagnostic> =
        results.iter().flat_map(|r| r.diagnostics.iter()).collect();

    if !all_diagnostics.is_empty() {
        match format {
            Some("json") => {
                let owned: Vec<_> = all_diagnostics.into_iter().cloned().collect();
                let output = JsonFormatter.format(&owned, None);
                eprintln!("{output}");
            }
            _ => {
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
    }

    if has_errors {
        Err(String::new())
    } else {
        Ok(())
    }
}

fn collect_compile_file(
    file: &PathBuf,
    out_file: Option<&PathBuf>,
    results: &mut Vec<CompileFileResult>,
) -> Result<(), String> {
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

    let fallback_out_file = file.with_extension("html");
    let out_file = out_file.unwrap_or(&fallback_out_file).clone();

    match compile_content_diagnostics(&content) {
        Ok(output) => {
            let diagnostics: Vec<_> = output
                .diagnostics
                .into_iter()
                .map(|d| d.with_file_path(file.display().to_string()))
                .collect();

            results.push(CompileFileResult {
                diagnostics,
                source: content,
                out_file,
                html: Some(output.html),
            });
        }
        Err(diagnostics) => {
            let diagnostics: Vec<_> = diagnostics
                .into_iter()
                .map(|d| d.with_file_path(file.display().to_string()))
                .collect();

            results.push(CompileFileResult {
                diagnostics,
                source: content,
                out_file,
                html: None,
            });
        }
    }

    Ok(())
}

fn collect_compile_dir(dir: &PathBuf, results: &mut Vec<CompileFileResult>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|e| format!("Unable to read directory {}: {e}", dir.display()))?
    {
        let entry = entry
            .map_err(|e| format!("Unable to read directory entry in {}: {e}", dir.display()))?;
        let path = entry.path();

        if path.is_dir() {
            collect_compile_dir(&path, results)?;
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "hsml") {
            collect_compile_file(&path, None, results)?;
        }
    }

    Ok(())
}
