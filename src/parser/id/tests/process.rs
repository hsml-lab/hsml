use nom::error::{Error, ErrorKind};

use crate::parser::id::process::process_id;

#[test]
fn it_should_process_id_with_text() {
    let input = "#id1 Text";

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(id, "id1");
    assert_eq!(rest, " Text");
}

#[test]
fn it_should_process_id_with_class() {
    let input = "#id1.text-red Text";

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(id, "id1");
    assert_eq!(rest, ".text-red Text");
}

#[test]
fn it_should_process_id_with_start_attribute() {
    let input = "#id1(hidden) Text";

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(id, "id1");
    assert_eq!(rest, "(hidden) Text");
}

#[test]
fn it_should_process_id_with_hyphen() {
    let input = "#my-id Text";

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(id, "my-id");
    assert_eq!(rest, " Text");
}

#[test]
fn it_should_process_id_with_underscore() {
    let input = "#my_id.text-red";

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(id, "my_id");
    assert_eq!(rest, ".text-red");
}

#[test]
fn it_should_process_id_with_mixed_separators() {
    let input = "#my-complex_id-2(hidden)";

    let (rest, id) = process_id(input).unwrap();

    assert_eq!(id, "my-complex_id-2");
    assert_eq!(rest, "(hidden)");
}

// Negative tests

#[test]
fn it_should_not_process_id_without_hash() {
    let input = "id1(disabled) Text";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "id1(disabled) Text",
            code: ErrorKind::Tag
        })),
        process_id(input)
    );

    let input = ".text-red(disabled) Text";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: ".text-red(disabled) Text",
            code: ErrorKind::Tag
        })),
        process_id(input)
    );
}
