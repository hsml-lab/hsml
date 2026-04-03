use std::fs;
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use hsml::diagnostic::Diagnostic;
use hsml::parser::{RootNode, Span, parse::parse};
use hsml::validate::validate;
use serde::Serialize;

use super::walker::walk_hsml_files;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ParseResult {
    #[serde(flatten)]
    ast: RootNode,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileParseResult {
    file_path: String,
    #[serde(flatten)]
    result: ParseResult,
}

pub fn exec_parse(matches: &ArgMatches) -> Result<(), String> {
    let path = matches.get_one::<PathBuf>("path").unwrap();

    let ignore_patterns: Vec<String> = matches
        .get_many::<String>("ignore_pattern")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();

    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    if path.is_dir() {
        let result = walk_hsml_files(path, &ignore_patterns)?;
        let mut file_results = Vec::new();

        for file in &result.files {
            let parse_result = parse_file(file)?;
            file_results.push(FileParseResult {
                file_path: file.display().to_string(),
                result: parse_result,
            });
        }

        let json = serde_json::to_string_pretty(&file_results)
            .map_err(|e| format!("Failed to serialize AST: {e}"))?;
        println!("{json}");
    } else if path.is_file() {
        path.extension()
            .filter(|&ext| ext == "hsml")
            .ok_or("File must have .hsml extension".to_string())?;

        let result = parse_file(path)?;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize AST: {e}"))?;
        println!("{json}");
    } else {
        return Err("Path must be a file or directory".to_string());
    }

    Ok(())
}

fn parse_file(path: &Path) -> Result<ParseResult, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Unable to read file {}: {e}", path.display()))?;

    let span = Span::new(&content);
    let (_, ast) = parse(span).map_err(|e| format!("Parse error in {}: {e}", path.display()))?;

    let diagnostics: Vec<Diagnostic> = validate(&ast, &content)
        .into_iter()
        .map(|d| d.with_file_path(path.display().to_string()))
        .collect();

    Ok(ParseResult { ast, diagnostics })
}
