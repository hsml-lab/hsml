use nom::error::ErrorKind;

use crate::parser::{
    HsmlProcessContext, Span,
    attribute::process::{process_attribute, process_attribute_key, process_attribute_value},
};

#[test]
fn it_should_process_attribute_key() {
    let input = Span::new(r#"#spoiler)"#);

    let (rest, attribute_key) = process_attribute_key(input).unwrap();

    assert_eq!(*attribute_key.fragment(), "#spoiler");
    assert_eq!(*rest.fragment(), ")");
}

#[test]
fn it_should_process_attribute_key_with_parentheses() {
    let input = Span::new(r#"(click)="handler""#);

    let (rest, attribute_key) = process_attribute_key(input).unwrap();

    assert_eq!(*attribute_key.fragment(), "(click)");
    assert_eq!(*rest.fragment(), r#"="handler""#);
}

#[test]
fn it_should_process_attribute_key_with_lone_cr() {
    let input = Span::new("src\rmore");

    let (rest, attribute_key) = process_attribute_key(input).unwrap();

    assert_eq!(*attribute_key.fragment(), "src");
    assert_eq!(*rest.fragment(), "\rmore");
}

#[test]
fn it_should_process_attribute_value() {
    let input = Span::new(r#""https://github.com/""#);

    let (rest, attribute_value) =
        process_attribute_value(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute_value.fragment(), "https://github.com/");
    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_process_attribute() {
    let input = Span::new(r#"src="https://github.com/""#);

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), r#"src="https://github.com/""#);
    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_process_attribute_without_value() {
    let input = Span::new("disabled ");

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), "disabled");
    assert_eq!(*rest.fragment(), " ");
}

#[test]
fn it_should_process_attribute_followed_by_another_attribute() {
    let input = Span::new("disabled required");

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), "disabled");
    assert_eq!(*rest.fragment(), " required");
}

#[test]
fn it_should_process_attribute_followed_by_another_attribute_separated_by_comma() {
    let input = Span::new("disabled, required");

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), "disabled");
    assert_eq!(*rest.fragment(), ", required");
}

#[test]
fn it_should_process_attribute_with_angular_binding() {
    let input = Span::new(r#"color="{{ color }}", required"#);

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), r#"color="{{ color }}""#);
    assert_eq!(*rest.fragment(), ", required");
}

#[test]
fn it_should_process_attribute_with_angular_ng_model() {
    let input = Span::new(r#"[(ngModel)]="name", required"#);

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), r#"[(ngModel)]="name""#);
    assert_eq!(*rest.fragment(), ", required");
}

#[test]
fn it_should_process_attribute_with_angular_event() {
    let input = Span::new(r#"(click)="setValue()", required"#);

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), r#"(click)="setValue()""#);
    assert_eq!(*rest.fragment(), ", required");
}

#[test]
fn it_should_process_attribute_with_vue_binding() {
    let input = Span::new(r#":src="image", alt="Image""#);

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), r#":src="image""#);
    assert_eq!(*rest.fragment(), r#", alt="Image""#);
}

#[test]
fn it_should_process_attribute_with_vue_event() {
    let input = Span::new(r#"@click="setValue()", color="primary""#);

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), r#"@click="setValue()""#);
    assert_eq!(*rest.fragment(), r#", color="primary""#);
}

#[test]
fn it_should_process_attribute_with_vue_slot() {
    let input = Span::new(r#"#header="slot""#);

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), r#"#header="slot""#);
    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_process_attribute_with_multiline_value() {
    let input = Span::new(
        r#"class="{
       'is-active': isActive,
         'is-disabled': isDisabled,
    }"
     :key="item.id""#,
    );

    let (rest, attribute) = process_attribute(
        input,
        &mut HsmlProcessContext {
            nested_tag_level: 1,
            indent_string: String::from("    "),
        },
    )
    .unwrap();

    assert_eq!(
        *attribute.fragment(),
        r#"class="{
       'is-active': isActive,
         'is-disabled': isDisabled,
    }""#
    );
    assert_eq!(
        *rest.fragment(),
        r#"
     :key="item.id""#
    );
}

#[test]
fn it_should_process_attribute_with_multibyte_value() {
    let input = Span::new(r#"alt="Ünïcödé" next"#);

    let (rest, attribute) = process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

    assert_eq!(*attribute.fragment(), r#"alt="Ünïcödé""#);
    assert_eq!(*rest.fragment(), " next");
}

#[test]
fn it_should_process_attribute_key_with_multibyte_bracket() {
    let (rest, key) = process_attribute_key(Span::new("[ä]=")).unwrap();

    assert_eq!(*key.fragment(), "[ä]");
    assert_eq!(*rest.fragment(), "=");
}

// Negative tests

#[test]
fn it_should_not_process_attribute_with_number() {
    let result = process_attribute(
        Span::new(r#"1src="https://github.com""#),
        &mut HsmlProcessContext::default(),
    );

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), r#"1src="https://github.com""#);
        assert_eq!(err.code, ErrorKind::AlphaNumeric);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_attribute_with_whitespace() {
    let result = process_attribute(
        Span::new(r#" src="https://github.com""#),
        &mut HsmlProcessContext::default(),
    );

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), r#" src="https://github.com""#);
        assert_eq!(err.code, ErrorKind::AlphaNumeric);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_attribute_with_dot() {
    let result = process_attribute(
        Span::new(r#".src="https://github.com""#),
        &mut HsmlProcessContext::default(),
    );

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), r#".src="https://github.com""#);
        assert_eq!(err.code, ErrorKind::AlphaNumeric);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_attribute_with_comma() {
    let result = process_attribute(
        Span::new(r#",src="https://github.com""#),
        &mut HsmlProcessContext::default(),
    );

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), r#",src="https://github.com""#);
        assert_eq!(err.code, ErrorKind::AlphaNumeric);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_attribute_without_quoted_value() {
    let result = process_attribute(Span::new("src=imgSrc"), &mut HsmlProcessContext::default());

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "imgSrc");
        assert_eq!(err.code, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_attribute_key_with_empty_input() {
    let result = process_attribute_key(Span::new(""));

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "");
        assert_eq!(err.code, ErrorKind::AlphaNumeric);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_attribute_value_with_empty_input() {
    let result = process_attribute_value(Span::new(""), &mut HsmlProcessContext::default());

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(*err.input.fragment(), "");
        assert_eq!(err.code, ErrorKind::Tag);
    } else {
        panic!("Expected Error");
    }
}

#[test]
fn it_should_not_process_attribute_with_line_ending() {
    let result = process_attribute(
        Span::new(
            r#"
src="https://github.com""#,
        ),
        &mut HsmlProcessContext::default(),
    );

    assert!(result.is_err());
    if let Err(nom::Err::Error(err)) = result {
        assert_eq!(
            *err.input.fragment(),
            r#"
src="https://github.com""#
        );
        assert_eq!(err.code, ErrorKind::AlphaNumeric);
    } else {
        panic!("Expected Error");
    }
}
