use nom::{
    Needed,
    error::{Error, ErrorKind},
};

use crate::parser::tag::process::process_tag;

#[test]
fn it_should_process_tag_div_with_text() {
    let input = "div Text";

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(tag, "div");
    assert_eq!(rest, " Text");
}

#[test]
fn it_should_process_tag_h1_with_text() {
    let input = "h1 Text";

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(tag, "h1");
    assert_eq!(rest, " Text");
}

#[test]
fn it_should_process_tag_with_id() {
    let input = "input#name";

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(tag, "input");
    assert_eq!(rest, "#name");
}

#[test]
fn it_should_process_tag_with_class() {
    let input = "p.text-red";

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(tag, "p");
    assert_eq!(rest, ".text-red");
}

#[test]
fn it_should_process_tag_with_attribute() {
    let input = "p()";

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(tag, "p");
    assert_eq!(rest, "()");
}

#[test]
fn it_should_process_tag_without_content() {
    let input = "span\n";

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(tag, "span");
    assert_eq!(rest, "\n");
}

#[test]
fn it_should_process_tag_pascal_case() {
    let input = "CInput.input";

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(tag, "CInput");
    assert_eq!(rest, ".input");
}

#[test]
fn it_should_process_tag_kebab_case() {
    let input = "c-input.input";

    let (rest, tag) = process_tag(input).unwrap();

    assert_eq!(tag, "c-input");
    assert_eq!(rest, ".input");
}

// Negative tests

#[test]
fn it_should_not_process_tag_with_number() {
    let input = "42.input";

    assert_eq!(
        Err(nom::Err::Incomplete(Needed::Unknown)),
        process_tag(input)
    );
}

#[test]
fn it_should_not_process_tag_with_special_character() {
    let input = "$span.input";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "$span.input",
            code: ErrorKind::TakeTill1
        })),
        process_tag(input)
    );

    let input = "]span.input";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "]span.input",
            code: ErrorKind::TakeTill1
        })),
        process_tag(input)
    );

    let input = ")span.input";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: ")span.input",
            code: ErrorKind::TakeTill1
        })),
        process_tag(input)
    );
}

#[test]
fn it_should_not_process_tag_with_whitespace() {
    let input = " span.input";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: " span.input",
            code: ErrorKind::TakeTill1
        })),
        process_tag(input)
    );
}

#[test]
fn it_should_not_process_tag_with_dot() {
    let input = ".span.input";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: ".span.input",
            code: ErrorKind::TakeTill1
        })),
        process_tag(input)
    );
}

#[test]
fn it_should_not_process_tag_with_hash() {
    let input = "#span.input";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "#span.input",
            code: ErrorKind::TakeTill1
        })),
        process_tag(input)
    );
}

#[test]
fn it_should_not_process_tag_with_line_ending() {
    let input = "\nspan.input";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "\nspan.input",
            code: ErrorKind::TakeTill1
        })),
        process_tag(input)
    );
}
