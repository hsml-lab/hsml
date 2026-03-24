use std::{
    env, fs,
    path::{Path, PathBuf},
};

use clap::ArgMatches;
use hsml::compile_content_diagnostics;

use super::diagnostics::{FileDiagnostics, has_errors, render_diagnostics};
use super::walker::walk_hsml_files;

pub fn exec_compile(matches: &ArgMatches) -> Result<(), String> {
    let out = matches.get_one::<PathBuf>("output");
    let format = matches
        .get_one::<String>("report_format")
        .map(|s| s.as_str());
    let is_json = format == Some("json");

    let ignore_patterns: Vec<String> = matches
        .get_many::<String>("ignore_pattern")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();

    let path = match matches.get_one::<PathBuf>("path") {
        Some(p) => p.clone(),
        None => env::current_dir().map_err(|e| format!("Unable to get current directory: {e}"))?,
    };
    let path = &path;

    if !is_json {
        println!("Compiling...");
    }

    let mut diagnostics: Vec<FileDiagnostics> = Vec::new();
    let mut io_errors: Vec<String> = Vec::new();

    if path.is_dir() {
        match walk_hsml_files(path, &ignore_patterns) {
            Ok(result) => {
                io_errors.extend(result.errors);
                for file in &result.files {
                    if let Err(e) =
                        compile_file(file, None, is_json, &mut diagnostics, &mut io_errors)
                    {
                        io_errors.push(e);
                    }
                }
            }
            Err(e) => io_errors.push(e),
        }
    } else if path.is_file() {
        if let Err(e) = compile_file(path, out, is_json, &mut diagnostics, &mut io_errors) {
            io_errors.push(e);
        }
    } else {
        return Err("Path must be a file or directory".to_string());
    }

    // Always render diagnostics before reporting errors
    render_diagnostics(&diagnostics, format);

    if !io_errors.is_empty() {
        Err(io_errors.join("\n"))
    } else if has_errors(&diagnostics) {
        Err(String::new())
    } else {
        Ok(())
    }
}

fn compile_file(
    file: &Path,
    out_file: Option<&PathBuf>,
    is_json: bool,
    diagnostics: &mut Vec<FileDiagnostics>,
    io_errors: &mut Vec<String>,
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
    let out_file = out_file.unwrap_or(&fallback_out_file);

    match compile_content_diagnostics(&content) {
        Ok(output) => {
            // Write HTML immediately — don't buffer
            if let Err(e) = fs::write(out_file, &output.html) {
                io_errors.push(format!("Unable to write file {}: {e}", out_file.display()));
            } else if !is_json {
                println!(
                    "Compiled HTML written to {} successfully",
                    out_file.display()
                );
            }

            let file_diags: Vec<_> = output
                .diagnostics
                .into_iter()
                .map(|d| d.with_file_path(file.display().to_string()))
                .collect();

            // Only retain source if there are diagnostics to render
            if !file_diags.is_empty() {
                diagnostics.push(FileDiagnostics {
                    diagnostics: file_diags,
                    source: content,
                });
            }
        }
        Err(errs) => {
            let file_diags: Vec<_> = errs
                .into_iter()
                .map(|d| d.with_file_path(file.display().to_string()))
                .collect();

            diagnostics.push(FileDiagnostics {
                diagnostics: file_diags,
                source: content,
            });
        }
    }

    Ok(())
}
