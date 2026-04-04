use std::path::PathBuf;

use clap::{Command, arg, value_parser};

use crate::cli::common::resolve_required_path;

#[test]
#[should_panic(expected = "path argument is required")]
fn it_should_panic_with_message_when_path_missing() {
    let matches = Command::new("test")
        .arg(arg!(path: [PATH] "Path").value_parser(value_parser!(PathBuf)))
        .get_matches_from(vec!["test"]);
    resolve_required_path(&matches);
}
