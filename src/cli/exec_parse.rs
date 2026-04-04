use std::fs;
use std::path::Path;

use clap::ArgMatches;
use hsml::diagnostic::Diagnostic;
use hsml::parser::{HsmlNode, Span, parse::parse};
use hsml::validate::validate;
use serde::Serialize;

use super::common::{resolve_ignore_patterns, resolve_required_path, validate_hsml_extension};
use super::walker::walk_hsml_files;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ParseResult {
    nodes: Option<Vec<HsmlNode>>,
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
    let path = resolve_required_path(matches);
    let ignore_patterns = resolve_ignore_patterns(matches);

    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    if path.is_dir() {
        let result = walk_hsml_files(&path, &ignore_patterns)?;

        for error in &result.errors {
            eprintln!("{error}");
        }

        let mut file_results = Vec::new();

        for file in &result.files {
            file_results.push(FileParseResult {
                file_path: file.display().to_string(),
                result: parse_file(file),
            });
        }

        let json = serde_json::to_string_pretty(&file_results)
            .map_err(|e| format!("Failed to serialize AST: {e}"))?;
        println!("{json}");
    } else if path.is_file() {
        validate_hsml_extension(&path)?;

        let result = parse_file(&path);
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Failed to serialize AST: {e}"))?;
        println!("{json}");
    } else {
        return Err("Path must be a file or directory".to_string());
    }

    Ok(())
}

fn parse_file(path: &Path) -> ParseResult {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            return ParseResult {
                nodes: None,
                diagnostics: vec![Diagnostic::compiler_error(format!(
                    "Unable to read file {}: {e}",
                    path.display()
                ))],
            };
        }
    };

    let span = Span::new(&content);
    let (_, ast) = match parse(span) {
        Ok(result) => result,
        Err(e) => {
            let diag = Diagnostic::from(&e).with_file_path(path.display().to_string());
            return ParseResult {
                nodes: None,
                diagnostics: vec![diag],
            };
        }
    };

    let diagnostics: Vec<Diagnostic> = validate(&ast, &content)
        .into_iter()
        .map(|d| d.with_file_path(path.display().to_string()))
        .collect();

    ParseResult {
        nodes: Some(ast.nodes),
        diagnostics,
    }
}
