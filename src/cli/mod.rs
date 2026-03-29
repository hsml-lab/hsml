use std::path::PathBuf;

use clap::{Command, arg, command, value_parser};

mod diagnostics;
pub mod exec_check;
pub mod exec_compile;
pub mod exec_format;
pub mod exec_lsp;
pub mod exec_parse;
pub(crate) mod walker;

pub fn cli() -> Command {
    command!()
        .about("HSML command line tool")
        .subcommand_required(true)
        .arg(arg!(debug: --debug "Print debug status messages").global(true))
        .arg(arg!(no_color: --"no-color" "Disable colored output").global(true))
        .subcommand(
            Command::new("compile")
                .about("Compiles given .hsml file or directory to .html")
                .arg(
                    arg!(path: [PATH] "Path to .hsml file or directory containing .hsml files")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(output: -o --out <OUTPUT> "Output file or directory")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(report_format: --"report-format" <FORMAT> "Report format for diagnostics")
                        .value_parser(["default", "json", "github", "gitlab"])
                        .default_value("default"),
                )
                .arg(
                    arg!(ignore_pattern: --"ignore-pattern" <PATTERN> "Glob pattern for files/directories to ignore")
                        .action(clap::ArgAction::Append),
                ),
        )
        .subcommand(
            Command::new("parse")
                .about("Parse given .hsml file and print the AST to stdout as JSON"),
        )
        .subcommand(Command::new("fmt").about("Format given .hsml file or directory"))
        .subcommand(
            Command::new("check")
                .about("Check given .hsml file or directory for errors and warnings")
                .arg(
                    arg!(path: [PATH] "Path to .hsml file or directory containing .hsml files")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(report_format: --"report-format" <FORMAT> "Report format for diagnostics")
                        .value_parser(["default", "json", "github", "gitlab"])
                        .default_value("default"),
                )
                .arg(
                    arg!(ignore_pattern: --"ignore-pattern" <PATTERN> "Glob pattern for files/directories to ignore")
                        .action(clap::ArgAction::Append),
                ),
        )
        .subcommand(Command::new("lsp").about("Run Language Server Protocol"))
}
