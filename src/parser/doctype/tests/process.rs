use nom::error::ErrorKind;

use crate::parser::Span;
use crate::parser::doctype::process::process_doctype;

#[test]
fn it_should_process_doctype_html() {
    let input = Span::new("doctype html");

    let (rest, doctype) = process_doctype(input).unwrap();

    assert_eq!(*doctype.fragment(), "html");
    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_process_doctype_html_with_trailing_newline() {
    let input = Span::new("doctype html\n");

    let (rest, doctype) = process_doctype(input).unwrap();

    assert_eq!(*doctype.fragment(), "html");
    assert_eq!(*rest.fragment(), "\n");
}

// Negative tests

#[test]
fn it_should_not_process_doctype_without_keyword() {
    let result = process_doctype(Span::new("html"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.span.fragment(), "html");
        assert_eq!(err.kind, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_unsupported_doctype() {
    let result = process_doctype(Span::new("doctype xml"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.span.fragment(), "xml");
        assert_eq!(err.kind, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }
}
