use crate::diagnostic::{Diagnostic, Location, Position, Severity};
use crate::parser::Span;
use crate::parser::error::{ErrorCode, HsmlError};

#[test]
fn it_should_convert_hsml_error_with_code() {
    let input = Span::new("div#a#b");
    let err = HsmlError::from_code(input, ErrorCode::DuplicateId);
    let diag = Diagnostic::from(&err);

    assert_eq!(diag.severity, Severity::Warning);
    assert_eq!(diag.message, "Duplicate attribute 'id' is not allowed");
    assert_eq!(diag.code, Some(ErrorCode::DuplicateId.code().to_string()));
    let pos = Position { line: 1, column: 1 };
    assert_eq!(
        diag.location,
        Some(Location {
            start: pos,
            end: pos,
        })
    );
    assert_eq!(diag.file_path, None);
}

#[test]
fn it_should_convert_hsml_error_without_message() {
    let input = Span::new("test");
    let err = HsmlError::from_kind(input, nom::error::ErrorKind::Tag);
    let diag = Diagnostic::from(&err);

    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.message, "parse error");
    assert_eq!(diag.code, None);
    assert!(diag.location.is_some());
}

#[test]
fn it_should_convert_nom_err_error() {
    let input = Span::new("test");
    let nom_err = nom::Err::Error(HsmlError::from_kind(input, nom::error::ErrorKind::Tag));
    let diag = Diagnostic::from(&nom_err);

    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.message, "parse error");
}

#[test]
fn it_should_convert_nom_err_failure() {
    let input = Span::new("div#a#b");
    let nom_err = nom::Err::Failure(HsmlError::from_code(input, ErrorCode::DuplicateId));
    let diag = Diagnostic::from(&nom_err);

    assert_eq!(diag.message, "Duplicate attribute 'id' is not allowed");
    assert_eq!(diag.code, Some(ErrorCode::DuplicateId.code().to_string()));
}

#[test]
fn it_should_convert_nom_err_incomplete() {
    let nom_err: nom::Err<HsmlError> = nom::Err::Incomplete(nom::Needed::Unknown);
    let diag = Diagnostic::from(&nom_err);

    assert_eq!(diag.message, "Unexpected end of input");
    assert_eq!(diag.location, None);
}

#[test]
fn it_should_create_compiler_error() {
    let diag = Diagnostic::compiler_error("Unsupported node type".to_string());

    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.message, "Unsupported node type");
    assert_eq!(diag.code, None);
    assert_eq!(diag.location, None);
}

#[test]
fn it_should_attach_file_path() {
    let diag = Diagnostic::compiler_error("error".to_string()).with_file_path("example.hsml");

    assert_eq!(diag.file_path, Some("example.hsml".to_string()));
}
