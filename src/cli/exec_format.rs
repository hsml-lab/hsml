use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::ArgMatches;
use hsml::formatter::{FormatOptions, format};
use hsml::parser::{Span, parse::parse};

use super::diagnostics::{DimCodes, format_duration, resolve_colors};
use super::walker::walk_hsml_files;

pub fn exec_format(matches: &ArgMatches) -> Result<(), String> {
    let path = match matches.get_one::<PathBuf>("path") {
        Some(p) => p.clone(),
        None => {
            std::env::current_dir().map_err(|e| format!("Unable to get current directory: {e}"))?
        }
    };

    let check = matches.get_flag("check");
    let debug = matches.get_flag("debug");
    let (_, dim) = resolve_colors(matches);

    let ignore_patterns: Vec<String> = matches
        .get_many::<String>("ignore_pattern")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();

    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    let options = FormatOptions::default();
    let mut has_diff = false;
    let mut has_errors = false;
    let file_count;

    let total_start = Instant::now();

    if path.is_dir() {
        let result = walk_hsml_files(&path, &ignore_patterns)?;

        if debug {
            println!(
                "{}Formatting {} file(s) from {}{}",
                dim.0,
                result.files.len(),
                path.display(),
                dim.1
            );
        }

        if !result.errors.is_empty() {
            has_errors = true;
            for error in &result.errors {
                eprintln!("{error}");
            }
        }

        file_count = result.files.len();
        for file in &result.files {
            if let Err(e) = format_file(file, check, debug, dim, &options, &mut has_diff) {
                has_errors = true;
                eprintln!("{e}");
            }
        }
    } else if path.is_file() {
        path.extension()
            .filter(|&ext| ext == "hsml")
            .ok_or("File must have .hsml extension".to_string())?;

        file_count = 1;
        format_file(&path, check, debug, dim, &options, &mut has_diff)?;
    } else {
        return Err("Path must be a file or directory".to_string());
    }

    if debug {
        let timing = format_duration(total_start.elapsed());
        let files = if file_count == 1 {
            "1 file"
        } else {
            &format!("{file_count} files")
        };
        let verb = if check { "checked" } else { "formatted" };
        println!("\n{}{files} {verb} in {timing}{}", dim.0, dim.1);
    }

    if has_errors {
        Err("Some files could not be processed".to_string())
    } else if check && has_diff {
        Err(String::new())
    } else {
        Ok(())
    }
}

fn format_file(
    path: &Path,
    check: bool,
    debug: bool,
    dim: DimCodes,
    options: &FormatOptions,
    has_diff: &mut bool,
) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Unable to read file {}: {e}", path.display()))?;

    let span = Span::new(&content);
    let (rest, ast) = parse(span).map_err(|e| format!("Parse error in {}: {e}", path.display()))?;

    if !rest.fragment().is_empty() {
        return Err(format!(
            "Parse error in {}: unconsumed input at line {}, column {}",
            path.display(),
            rest.location_line(),
            rest.get_column()
        ));
    }

    let start = Instant::now();
    let formatted = format(&ast, options);

    if content == formatted {
        if debug {
            let timing = format_duration(start.elapsed());
            println!("{}{} {timing} (unchanged){}", dim.0, path.display(), dim.1);
        }
        return Ok(());
    }

    if check {
        if debug {
            let timing = format_duration(start.elapsed());
            println!(
                "{}{} {timing} (needs formatting){}",
                dim.0,
                path.display(),
                dim.1
            );
        } else {
            eprintln!("{}", path.display());
        }
        *has_diff = true;
    } else {
        fs::write(path, &formatted)
            .map_err(|e| format!("Unable to write file {}: {e}", path.display()))?;
        if debug {
            let timing = format_duration(start.elapsed());
            println!("{}{} {timing}{}", dim.0, path.display(), dim.1);
        }
    }

    Ok(())
}
