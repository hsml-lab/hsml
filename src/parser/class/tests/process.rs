use nom::error::{Error, ErrorKind};

use crate::parser::class::process::process_class;

#[test]
fn it_should_process_class_with_text() {
    let input = ".text-red Text";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "text-red");
    assert_eq!(rest, " Text");
}

#[test]
fn it_should_process_class_with_colon() {
    let input = ".focus:outline-none Text";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "focus:outline-none");
    assert_eq!(rest, " Text");
}

#[test]
fn it_should_process_class_with_arbitrary_tailwind_value() {
    let input = ".bg-[#1da1f2]#name Text";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "bg-[#1da1f2]");
    assert_eq!(rest, "#name Text");
}

#[test]
fn it_should_process_class_with_arbitrary_tailwind_value_2() {
    let input = ".lg:[&:nth-child(3)]:hover:underline#name Text";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "lg:[&:nth-child(3)]:hover:underline");
    assert_eq!(rest, "#name Text");
}

#[test]
fn it_should_process_class_with_arbitrary_tailwind_value_3() {
    let input = ".bg-[url('/what_a_rush.png')]#name Text";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "bg-[url('/what_a_rush.png')]");
    assert_eq!(rest, "#name Text");
}

#[test]
fn it_should_process_class_with_id() {
    let input = ".text-red#name Text";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "text-red");
    assert_eq!(rest, "#name Text");
}

#[test]
fn it_should_process_class_with_attribute() {
    let input = ".text-red(disabled) Text";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "text-red");
    assert_eq!(rest, "(disabled) Text");
}

#[test]
fn it_should_process_class_with_whitespace() {
    let input = ".text-red Text";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "text-red");
    assert_eq!(rest, " Text");
}

#[test]
fn it_should_process_class_with_tab() {
    let input = ".text-red\t";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "text-red");
    assert_eq!(rest, "\t");
}

#[test]
fn it_should_process_class_with_line_ending() {
    let input = ".text-red\n";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "text-red");
    assert_eq!(rest, "\n");
}

#[test]
fn it_should_process_class_with_crlf() {
    let input = ".text-red\r\n";

    let (rest, class) = process_class(input).unwrap();

    assert_eq!(class, "text-red");
    assert_eq!(rest, "\r\n");
}

// Negative tests

#[test]
fn it_should_not_process_class_without_dot() {
    let input = "text-red(disabled) Text";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "text-red(disabled) Text",
            code: ErrorKind::Tag
        })),
        process_class(input)
    );

    let input = "#text-red(disabled) Text";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "#text-red(disabled) Text",
            code: ErrorKind::Tag
        })),
        process_class(input)
    );
}
