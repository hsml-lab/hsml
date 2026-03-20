use nom::error::ErrorKind;

use crate::parser::Span;
use crate::parser::id::process::process_id;

#[test]
fn it_should_process_id_with_text() {
    let input = Span::new("#id1 Text");

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(*id.fragment(), "id1");
    assert_eq!(*rest.fragment(), " Text");
}

#[test]
fn it_should_process_id_with_class() {
    let input = Span::new("#id1.text-red Text");

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(*id.fragment(), "id1");
    assert_eq!(*rest.fragment(), ".text-red Text");
}

#[test]
fn it_should_process_id_with_start_attribute() {
    let input = Span::new("#id1(hidden) Text");

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(*id.fragment(), "id1");
    assert_eq!(*rest.fragment(), "(hidden) Text");
}

#[test]
fn it_should_process_id_with_hyphen() {
    let input = Span::new("#my-id Text");

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(*id.fragment(), "my-id");
    assert_eq!(*rest.fragment(), " Text");
}

#[test]
fn it_should_process_id_with_underscore() {
    let input = Span::new("#my_id.text-red");

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(*id.fragment(), "my_id");
    assert_eq!(*rest.fragment(), ".text-red");
}

#[test]
fn it_should_process_id_with_mixed_separators() {
    let input = Span::new("#my-complex_id-2(hidden)");

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(*id.fragment(), "my-complex_id-2");
    assert_eq!(*rest.fragment(), "(hidden)");
}

#[test]
fn it_should_process_id_with_unicode() {
    let input = Span::new("#caf\u{00e9}.text-red");

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(*id.fragment(), "caf\u{00e9}");
    assert_eq!(*rest.fragment(), ".text-red");
}

// Negative tests

#[test]
fn it_should_not_process_id_without_hash() {
    let result = process_id(Span::new("id1(disabled) Text"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "id1(disabled) Text");
        assert_eq!(err.code, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }

    let result = process_id(Span::new(".text-red(disabled) Text"));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), ".text-red(disabled) Text");
        assert_eq!(err.code, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }
}
