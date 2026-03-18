use nom::error::{Error, ErrorKind};

use crate::parser::comment::process::{process_dev_comment, process_native_comment};

#[test]
fn it_should_process_dev_comment() {
    let input = "// This is a dev comment\n";

    let (rest, comment) = process_dev_comment(input).unwrap();

    assert_eq!(comment, " This is a dev comment");
    assert_eq!(rest, "\n");
}

#[test]
fn it_should_process_native_comment() {
    let input = "//! This is a native comment\n";

    let (rest, comment) = process_native_comment(input).unwrap();

    assert_eq!(comment, " This is a native comment");
    assert_eq!(rest, "\n");
}

// Negative tests

#[test]
fn it_should_not_process_dev_comment() {
    let input = "//! This is not a dev comment\n";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "! This is not a dev comment\n",
            code: ErrorKind::Tag
        })),
        process_dev_comment(input)
    );
}

#[test]
fn it_should_not_process_native_comment() {
    let input = "// This is not a native comment\n";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "// This is not a native comment\n",
            code: ErrorKind::Tag
        })),
        process_native_comment(input)
    );
}
