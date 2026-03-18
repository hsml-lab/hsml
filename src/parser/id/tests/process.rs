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
