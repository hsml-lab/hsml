mod cli;

use std::process;

use cli::{
    cli, exec_check::exec_check, exec_compile::exec_compile, exec_format::exec_format,
    exec_lsp::exec_lsp, exec_parse::exec_parse,
};
use tokio::runtime::Runtime;

fn main() -> Result<(), String> {
    let matches = cli().get_matches();

    let result: Result<(), String> = match matches.subcommand() {
        Some(("compile", sub_matches)) => exec_compile(sub_matches),
        Some(("parse", sub_matches)) => exec_parse(sub_matches).map_err(|e| e.to_string()),
        Some(("fmt", sub_matches)) => exec_format(sub_matches).map_err(|e| e.to_string()),
        Some(("check", sub_matches)) => exec_check(sub_matches).map_err(|e| e.to_string()),
        Some(("lsp", sub_matches)) => {
            let rt =
                Runtime::new().map_err(|_| "Failed to initialize Tokio runtime".to_string())?;
            rt.block_on(exec_lsp(sub_matches));
            Ok(())
        }
        Some((ext, _)) => {
            panic!("Unknown subcommand: {}", ext);
        }
        _ => unreachable!("Subcommand required"),
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        process::exit(1);
    } else {
        process::exit(0);
    }
}
