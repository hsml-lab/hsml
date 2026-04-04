use std::path::Path;

use crate::cli::common::validate_hsml_extension;

#[test]
fn it_should_accept_hsml_files() {
    assert!(validate_hsml_extension(Path::new("test.hsml")).is_ok());
    assert!(validate_hsml_extension(Path::new("path/to/file.hsml")).is_ok());
}

#[test]
fn it_should_reject_non_hsml_files() {
    assert!(validate_hsml_extension(Path::new("test.txt")).is_err());
    assert!(validate_hsml_extension(Path::new("test.html")).is_err());
    assert!(validate_hsml_extension(Path::new("test")).is_err());
    assert!(validate_hsml_extension(Path::new("test.hsml.bak")).is_err());
    assert!(validate_hsml_extension(Path::new("test.HSML")).is_err());
}
