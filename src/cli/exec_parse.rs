use std::{fs, path::PathBuf};

use clap::ArgMatches;
use hsml::parser::{Span, parse::parse};

pub fn exec_parse(matches: &ArgMatches) -> Result<(), String> {
    let path = matches.get_one::<PathBuf>("path").unwrap();

    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    if !path.is_file() {
        return Err("Given path must be a file".to_string());
    }

    path.extension()
        .filter(|&ext| ext == "hsml")
        .ok_or("File must have .hsml extension".to_string())?;

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Unable to read file {}: {e}", path.display()))?;

    let span = Span::new(&content);
    let (_, ast) = parse(span).map_err(|e| format!("Parse error: {e}"))?;

    let json =
        serde_json::to_string_pretty(&ast).map_err(|e| format!("Failed to serialize AST: {e}"))?;

    println!("{json}");

    Ok(())
}
