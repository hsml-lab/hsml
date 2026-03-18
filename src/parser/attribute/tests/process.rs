use nom::error::{Error, ErrorKind};

use crate::parser::{
    HsmlProcessContext,
    attribute::process::{process_attribute, process_attribute_key, process_attribute_value},
};

#[test]
fn it_should_process_attribute_key() {
    let input = r#"#spoiler)"#;

    let (rest, attribute_key) = process_attribute_key(input).unwrap();

    assert_eq!(attribute_key, "#spoiler");
    assert_eq!(rest, ")");
}

#[test]
fn it_should_process_attribute_value() {
    let input = r#""https://github.com/""#;

    let (rest, attribute_value) =
        process_attribute_value(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute_value, "https://github.com/");
    assert_eq!(rest, "");
}

#[test]
fn it_should_process_attribute() {
    let input = r#"src="https://github.com/""#;

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, r#"src="https://github.com/""#);
    assert_eq!(rest, "");
}

#[test]
fn it_should_process_attribute_without_value() {
    let input = "disabled ";

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, "disabled");
    assert_eq!(rest, " ");
}

#[test]
fn it_should_process_attribute_followed_by_another_attribute() {
    let input = "disabled required";

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, "disabled");
    assert_eq!(rest, " required");
}

#[test]
fn it_should_process_attribute_followed_by_another_attribute_separated_by_comma() {
    let input = "disabled, required";

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, "disabled");
    assert_eq!(rest, ", required");
}

#[test]
fn it_should_process_attribute_with_angular_binding() {
    let input = r#"color="{{ color }}", required"#;

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, r#"color="{{ color }}""#);
    assert_eq!(rest, ", required");
}

#[test]
fn it_should_process_attribute_with_angular_ng_model() {
    let input = r#"[(ngModel)]="name", required"#;

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, r#"[(ngModel)]="name""#);
    assert_eq!(rest, ", required");
}

#[test]
fn it_should_process_attribute_with_angular_event() {
    let input = r#"(click)="setValue()", required"#;

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, r#"(click)="setValue()""#);
    assert_eq!(rest, ", required");
}

#[test]
fn it_should_process_attribute_with_vue_binding() {
    let input = r#":src="image", alt="Image""#;

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, r#":src="image""#);
    assert_eq!(rest, r#", alt="Image""#);
}

#[test]
fn it_should_process_attribute_with_vue_event() {
    let input = r#"@click="setValue()", color="primary""#;

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, r#"@click="setValue()""#);
    assert_eq!(rest, r#", color="primary""#);
}

#[test]
fn it_should_process_attribute_with_vue_slot() {
    let input = r#"#header="slot""#;

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(attribute, r#"#header="slot""#);
    assert_eq!(rest, "");
}

#[test]
fn it_should_process_attribute_with_multiline_value() {
    let input = r#"class="{
       'is-active': isActive,
         'is-disabled': isDisabled,
    }"
     :key="item.id""#;

    let (rest, attribute) = process_attribute(
        input,
        &mut HsmlProcessContext {
            nested_tag_level: 1,
            indent_string: String::from("    "),
        },
    )
    .unwrap();

    assert_eq!(
        attribute,
        r#"class="{
       'is-active': isActive,
         'is-disabled': isDisabled,
    }""#
    );
    assert_eq!(
        rest,
        r#"
     :key="item.id""#
    );
}

// Negative tests

#[test]
fn it_should_not_process_attribute_with_number() {
    let input = r#"1src="https://github.com""#;

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: r#"1src="https://github.com""#,
            code: ErrorKind::AlphaNumeric
        })),
        process_attribute(input, &mut HsmlProcessContext::default())
    );
}

#[test]
fn it_should_not_process_attribute_with_whitespace() {
    let input = r#" src="https://github.com""#;

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: r#" src="https://github.com""#,
            code: ErrorKind::AlphaNumeric
        })),
        process_attribute(input, &mut HsmlProcessContext::default())
    );
}

#[test]
fn it_should_not_process_attribute_with_dot() {
    let input = r#".src="https://github.com""#;

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: r#".src="https://github.com""#,
            code: ErrorKind::AlphaNumeric
        })),
        process_attribute(input, &mut HsmlProcessContext::default())
    );
}

#[test]
fn it_should_not_process_attribute_with_comma() {
    let input = r#",src="https://github.com""#;

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: r#",src="https://github.com""#,
            code: ErrorKind::AlphaNumeric
        })),
        process_attribute(input, &mut HsmlProcessContext::default())
    );
}

#[test]
fn it_should_not_process_attribute_without_quoted_value() {
    let input = "src=imgSrc";

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: "imgSrc",
            code: ErrorKind::Tag
        })),
        process_attribute(input, &mut HsmlProcessContext::default())
    );
}

#[test]
fn it_should_not_process_attribute_with_line_ending() {
    let input = r#"
src="https://github.com""#;

    assert_eq!(
        Err(nom::Err::Error(Error {
            input: r#"
src="https://github.com""#,
            code: ErrorKind::AlphaNumeric
        })),
        process_attribute(input, &mut HsmlProcessContext::default())
    );
}
