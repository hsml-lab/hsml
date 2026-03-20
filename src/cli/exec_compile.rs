use std::{env, fs, path::PathBuf};

use clap::ArgMatches;
use hsml::{
    compiler::{HsmlCompileOptions, compile},
    parser::{Span, parse::parse},
};

pub fn exec_compile(matches: &ArgMatches) -> Result<(), String> {
    println!("Compiling...");
    let path = matches.get_one::<PathBuf>("path");
    let out = matches.get_one::<PathBuf>("output");

    let fallback_path =
        env::current_dir().map_err(|e| format!("Unable to get current directory: {e}"))?;
    let path = path.unwrap_or(&fallback_path);

    if path.is_dir() {
        compile_hsml_files_in_dir(path)
    } else if path.is_file() {
        compile_file(path, out)
    } else {
        Err("Path must be a file or directory".to_string())
    }
}

fn compile_file(file: &PathBuf, out_file: Option<&PathBuf>) -> Result<(), String> {
    // check that file exists
    if !file.exists() {
        return Err("File does not exist".to_string());
    }

    // check that file is a file
    if !file.is_file() {
        return Err("Given file must be a file".to_string());
    }

    // check that file ends with .hsml
    file.extension()
        .filter(|&ext| ext == "hsml")
        .ok_or("File must have .hsml extension".to_string())?;

    println!("Compiling file {}...", file.display());

    // read the file
    let content = fs::read_to_string(file)
        .map_err(|e| format!("Unable to read file {}: {e}", file.display()))?;

    let fallback_out_file = file.with_extension("html");
    let out_file = out_file.unwrap_or(&fallback_out_file);

    // parse the file
    let (rest, hsml_ast) = parse(Span::new(&content))
        .map_err(|e| format!("Unable to parse file {}: {e:?}", file.display()))?;

    if !rest.fragment().is_empty() {
        return Err(format!(
            "Unable to parse file {}: unconsumed input at line {}, column {}",
            file.display(),
            rest.location_line(),
            rest.get_column()
        ));
    }

    // compile the AST
    let html_content = compile(&hsml_ast, &HsmlCompileOptions::default())
        .map_err(|e| format!("Unable to compile file {}: {e}", file.display()))?;

    fs::write(out_file, html_content)
        .map_err(|e| format!("Unable to write file {}: {e}", out_file.display()))?;

    println!(
        "Compiled HTML written to {} successfully",
        out_file.display()
    );

    Ok(())
}

fn compile_hsml_files_in_dir(dir: &PathBuf) -> Result<(), String> {
    let mut errors: Vec<String> = Vec::new();

    for entry in
        fs::read_dir(dir).map_err(|e| format!("Unable to read directory {}: {e}", dir.display()))?
    {
        let entry = entry
            .map_err(|e| format!("Unable to read directory entry in {}: {e}", dir.display()))?;
        let path = entry.path();

        if path.is_dir() {
            if let Err(e) = compile_hsml_files_in_dir(&path) {
                errors.push(e);
            }
        } else if path.is_file()
            && path.extension().is_some_and(|ext| ext == "hsml")
            && let Err(e) = compile_file(&path, None)
        {
            errors.push(e);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
