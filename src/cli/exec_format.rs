use std::fs;
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use hsml::formatter::{FormatOptions, format};
use hsml::parser::{Span, parse::parse};

use super::walker::walk_hsml_files;

pub fn exec_format(matches: &ArgMatches) -> Result<(), String> {
    let path = match matches.get_one::<PathBuf>("path") {
        Some(p) => p.clone(),
        None => {
            std::env::current_dir().map_err(|e| format!("Unable to get current directory: {e}"))?
        }
    };

    let check = matches.get_flag("check");

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

    if path.is_dir() {
        let result = walk_hsml_files(&path, &ignore_patterns)?;

        if !result.errors.is_empty() {
            has_errors = true;
            for error in &result.errors {
                eprintln!("{error}");
            }
        }

        for file in &result.files {
            if let Err(e) = format_file(file, check, &options, &mut has_diff) {
                has_errors = true;
                eprintln!("{e}");
            }
        }
    } else if path.is_file() {
        path.extension()
            .filter(|&ext| ext == "hsml")
            .ok_or("File must have .hsml extension".to_string())?;

        format_file(&path, check, &options, &mut has_diff)?;
    } else {
        return Err("Path must be a file or directory".to_string());
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

    let formatted = format(&ast, options);

    if content == formatted {
        return Ok(());
    }

    if check {
        eprintln!("{}", path.display());
        *has_diff = true;
    } else {
        fs::write(path, &formatted)
            .map_err(|e| format!("Unable to write file {}: {e}", path.display()))?;
    }

    Ok(())
}
