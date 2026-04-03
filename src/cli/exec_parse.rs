use std::fs;
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use hsml::parser::{RootNode, Span, parse::parse};
use serde::Serialize;

use super::walker::walk_hsml_files;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileAst {
    file_path: String,
    ast: RootNode,
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
        let mut file_asts = Vec::new();

        for file in &result.files {
            let ast = parse_file(file)?;
            file_asts.push(FileAst {
                file_path: file.display().to_string(),
                ast,
            });
        }

        let json = serde_json::to_string_pretty(&file_asts)
            .map_err(|e| format!("Failed to serialize AST: {e}"))?;
        println!("{json}");
    } else if path.is_file() {
        path.extension()
            .filter(|&ext| ext == "hsml")
            .ok_or("File must have .hsml extension".to_string())?;

        let ast = parse_file(path)?;
        let json = serde_json::to_string_pretty(&ast)
            .map_err(|e| format!("Failed to serialize AST: {e}"))?;
        println!("{json}");
    } else {
        return Err("Path must be a file or directory".to_string());
    }

    Ok(())
}

fn parse_file(path: &Path) -> Result<RootNode, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Unable to read file {}: {e}", path.display()))?;

    let span = Span::new(&content);
    let (_, ast) = parse(span).map_err(|e| format!("Parse error in {}: {e}", path.display()))?;

    Ok(ast)
}
