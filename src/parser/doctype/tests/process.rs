use nom::error::{Error, ErrorKind};

use crate::parser::doctype::process::process_doctype;

#[test]
fn it_should_process_doctype_html() {
    let input = "doctype html";

    let (rest, doctype) = process_doctype(input).unwrap();

    assert_eq!(doctype, "html");
    assert_eq!(rest, "");
}

#[test]
fn it_should_process_doctype_html_with_trailing_newline() {
    let input = "doctype html\n";

    let (rest, doctype) = process_doctype(input).unwrap();

    assert_eq!(doctype, "html");
    assert_eq!(rest, "\n");
}

// Negative tests

#[test]
fn it_should_not_process_doctype_without_keyword() {
    let input = "html";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "html",
            code: ErrorKind::Tag
        })),
        process_doctype(input)
    );
}

#[test]
fn it_should_not_process_unsupported_doctype() {
    let input = "doctype xml";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "xml",
            code: ErrorKind::Tag
        })),
        process_doctype(input)
    );
}
