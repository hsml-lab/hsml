use nom::error::ErrorKind;

use crate::parser::Span;
use crate::parser::class::process::process_class;

#[test]
fn it_should_process_class_with_text() {
    let input = Span::new(".text-red Text");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "text-red");
    assert_eq!(*rest.fragment(), " Text");
}

#[test]
fn it_should_process_class_with_colon() {
    let input = Span::new(".focus:outline-none Text");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "focus:outline-none");
    assert_eq!(*rest.fragment(), " Text");
}

#[test]
fn it_should_process_class_with_arbitrary_tailwind_value() {
    let input = Span::new(".bg-[#1da1f2]#name Text");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "bg-[#1da1f2]");
    assert_eq!(*rest.fragment(), "#name Text");
}

#[test]
fn it_should_process_class_with_arbitrary_tailwind_value_2() {
    let input = Span::new(".lg:[&:nth-child(3)]:hover:underline#name Text");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "lg:[&:nth-child(3)]:hover:underline");
    assert_eq!(*rest.fragment(), "#name Text");
}

#[test]
fn it_should_process_class_with_arbitrary_tailwind_value_3() {
    let input = Span::new(".bg-[url('/what_a_rush.png')]#name Text");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "bg-[url('/what_a_rush.png')]");
    assert_eq!(*rest.fragment(), "#name Text");
}

#[test]
fn it_should_process_class_with_id() {
    let input = Span::new(".text-red#name Text");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "text-red");
    assert_eq!(*rest.fragment(), "#name Text");
}

#[test]
fn it_should_process_class_with_attribute() {
    let input = Span::new(".text-red(disabled) Text");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "text-red");
    assert_eq!(*rest.fragment(), "(disabled) Text");
}

#[test]
fn it_should_process_class_with_whitespace() {
    let input = Span::new(".text-red Text");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "text-red");
    assert_eq!(*rest.fragment(), " Text");
}

#[test]
fn it_should_process_class_with_tab() {
    let input = Span::new(".text-red\t");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "text-red");
    assert_eq!(*rest.fragment(), "\t");
}

#[test]
fn it_should_process_class_with_line_ending() {
    let input = Span::new(".text-red\n");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "text-red");
    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_process_class_with_crlf() {
    let input = Span::new(".text-red\r\n");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "text-red");
    assert_eq!(*rest.fragment(), "\r\n");
}

#[test]
fn it_should_process_class_with_lone_cr() {
    let input = Span::new(".text-red\rmore");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "text-red");
    assert_eq!(*rest.fragment(), "\rmore");
}

#[test]
fn it_should_process_class_with_multibyte_chars() {
    let input = Span::new(".café#id");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "café");
    assert_eq!(*rest.fragment(), "#id");
}

#[test]
fn it_should_process_class_with_multibyte_arbitrary_value() {
    let input = Span::new(".bg-[ä]#id");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "bg-[ä]");
    assert_eq!(*rest.fragment(), "#id");
}

#[test]
fn it_should_process_class_with_char_after_bracket() {
    let input = Span::new(".a[x]b#name");

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(*class.fragment(), "a[x]b");
    assert_eq!(*rest.fragment(), "#name");
}

// Negative tests

#[test]
fn it_should_not_process_class_without_dot() {
    let result = process_class(Span::new("text-red(disabled) Text"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "text-red(disabled) Text");
        assert_eq!(err.code, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }

    let result = process_class(Span::new("#text-red(disabled) Text"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "#text-red(disabled) Text");
        assert_eq!(err.code, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }
}
