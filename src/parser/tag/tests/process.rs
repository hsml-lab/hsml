use nom::{Needed, error::ErrorKind};

use crate::parser::Span;
use crate::parser::tag::process::process_tag;

#[test]
fn it_should_process_tag_div_with_text() {
    let input = Span::new("div Text");

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(*tag.fragment(), "div");
    assert_eq!(*rest.fragment(), " Text");
}

#[test]
fn it_should_process_tag_h1_with_text() {
    let input = Span::new("h1 Text");

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(*tag.fragment(), "h1");
    assert_eq!(*rest.fragment(), " Text");
}

#[test]
fn it_should_process_tag_with_id() {
    let input = Span::new("input#name");

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(*tag.fragment(), "input");
    assert_eq!(*rest.fragment(), "#name");
}

#[test]
fn it_should_process_tag_with_class() {
    let input = Span::new("p.text-red");

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(*tag.fragment(), "p");
    assert_eq!(*rest.fragment(), ".text-red");
}

#[test]
fn it_should_process_tag_with_attribute() {
    let input = Span::new("p()");

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(*tag.fragment(), "p");
    assert_eq!(*rest.fragment(), "()");
}

#[test]
fn it_should_process_tag_without_content() {
    let input = Span::new("span\n");

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(*tag.fragment(), "span");
    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_process_tag_pascal_case() {
    let input = Span::new("CInput.input");

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(*tag.fragment(), "CInput");
    assert_eq!(*rest.fragment(), ".input");
}

#[test]
fn it_should_process_tag_kebab_case() {
    let input = Span::new("c-input.input");

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(*tag.fragment(), "c-input");
    assert_eq!(*rest.fragment(), ".input");
}

// Negative tests

#[test]
fn it_should_not_process_tag_with_number() {
    let input = Span::new("42.input");

    assert_eq!(
        Err(nom::Err::Incomplete(Needed::Unknown)),
        process_tag(input)
    );
}

#[test]
fn it_should_not_process_tag_with_special_character() {
    let result = process_tag(Span::new("$span.input"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "$span.input");
        assert_eq!(err.code, ErrorKind::TakeTill1);
    } else {
        panic!("Expected Error");
    }

    let result = process_tag(Span::new("]span.input"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "]span.input");
        assert_eq!(err.code, ErrorKind::TakeTill1);
    } else {
        panic!("Expected Error");
    }

    let result = process_tag(Span::new(")span.input"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), ")span.input");
        assert_eq!(err.code, ErrorKind::TakeTill1);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_tag_with_whitespace() {
    let result = process_tag(Span::new(" span.input"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), " span.input");
        assert_eq!(err.code, ErrorKind::TakeTill1);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_tag_with_dot() {
    let result = process_tag(Span::new(".span.input"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), ".span.input");
        assert_eq!(err.code, ErrorKind::TakeTill1);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_tag_with_hash() {
    let result = process_tag(Span::new("#span.input"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "#span.input");
        assert_eq!(err.code, ErrorKind::TakeTill1);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_tag_with_line_ending() {
    let result = process_tag(Span::new("\nspan.input"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "\nspan.input");
        assert_eq!(err.code, ErrorKind::TakeTill1);
    } else {
        panic!("Expected Error");
    }
}
