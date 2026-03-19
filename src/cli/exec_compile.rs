use std::{env, fs, path::PathBuf};

use clap::ArgMatches;
use hsml::{
    compiler::{HsmlCompileOptions, compile},
    parser::parse::parse,
};

pub fn exec_compile(matches: &ArgMatches) -> Result<(), &str> {
    println!("Compiling...");
    let path = matches.get_one::<PathBuf>("path");
    let out = matches.get_one::<PathBuf>("output");

    let fallback_path = env::current_dir().expect("Unable to get current directory");
    let path = path.unwrap_or(&fallback_path);

    if path.is_dir() {
        compile_hsml_files_in_dir(path)
    } else if path.is_file() {
        compile_file(path, out)
    } else {
        Err("Path must be a file or directory")
    }
}

fn compile_file(file: &PathBuf, out_file: Option<&PathBuf>) -> Result<(), &'static str> {
    // check that file exists
    if !file.exists() {
        return Err("File does not exist");
    }

    // check that file is a file
    if !file.is_file() {
        return Err("Given file must be a file");
    }

    // check that file ends with .hsml
    file.extension()
        .filter(|&ext| ext == "hsml")
        .ok_or("File must have .hsml extension")?;

    println!("Compiling file {}...", file.display());

    // check that file exists and read it
    let content = fs::read_to_string(file).expect("Unable to read file");

    let fallback_out_file = file.with_extension("html");
    let out_file = out_file.unwrap_or(&fallback_out_file);

    // parse the file
    let hsml_ast = if let Ok((_, hsml_ast)) = parse(&content) {
        hsml_ast
    } else {
        return Err("Unable to parse file");
    };

    // compile the AST
    let html_content =
        compile(&hsml_ast, &HsmlCompileOptions::default()).map_err(|_| "Unable to compile file")?;

    fs::write(out_file, html_content).expect("Unable to write file");

    println!(
        "Compiled HTML written to {} successfully",
        out_file.display()
    );

    Ok(())
}

fn compile_hsml_files_in_dir(dir: &PathBuf) -> Result<(), &'static str> {
    // compile all hsml files in the directory and call this function recursively on all subdirectories
    // if there is an error, ignore it and continue
    for entry in fs::read_dir(dir).expect("Unable to read directory") {
        let entry = entry.expect("Unable to read directory entry");
        let path = entry.path();

        if path.is_dir() {
            compile_hsml_files_in_dir(&path).ok();
        } else if path.is_file() {
            compile_file(&path, None).ok();
        }
    }

    Ok(())
}
