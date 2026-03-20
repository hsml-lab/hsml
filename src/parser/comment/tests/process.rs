use nom::error::ErrorKind;

use crate::parser::Span;
use crate::parser::comment::process::{process_dev_comment, process_native_comment};

#[test]
fn it_should_process_dev_comment() {
    let input = Span::new("// This is a dev comment\n");

    let (rest, comment) = process_dev_comment(input).unwrap();

    assert_eq!(*comment.fragment(), " This is a dev comment");
    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_process_native_comment() {
    let input = Span::new("//! This is a native comment\n");

    let (rest, comment) = process_native_comment(input).unwrap();

    assert_eq!(*comment.fragment(), " This is a native comment");
    assert_eq!(*rest.fragment(), "\n");
}

// Negative tests

#[test]
fn it_should_not_process_dev_comment() {
    let result = process_dev_comment(Span::new("//! This is not a dev comment\n"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "! This is not a dev comment\n");
        assert_eq!(err.code, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_native_comment() {
    let result = process_native_comment(Span::new("// This is not a native comment\n"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "// This is not a native comment\n");
        assert_eq!(err.code, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }
}
