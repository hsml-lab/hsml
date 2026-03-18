use nom::{
    IResult, Needed,
    bytes::complete::tag,
    error::{Error, ErrorKind},
};

use crate::parser::HsmlProcessContext;

fn is_valid_attribute_key_start(c: char) -> bool {
    c.is_alphabetic() || c == ':' || c == '#' || c == '@' || c == '[' || c == '('
}

fn process_attribute_key(input: &str) -> IResult<&str, &str> {
    let first_char = input.chars().next().expect("input is empty");

    if first_char.is_numeric() {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::AlphaNumeric)));
    }

    if !is_valid_attribute_key_start(first_char) {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::AlphaNumeric)));
    }

    let mut remaining = input;

    let mut attribute_key_index = 0;

    loop {
        // get first char and check if it is a `(`
        // if so, find the closing brace, because otherwise the closing brace is the end of the attributes
        let first_char = remaining.get(..1);

        match first_char {
            Some(")") => {
                // we hit the end of the attributes, so we are done
                break;
            }
            Some(",") => {
                // we hit a comma, so we are done
                break;
            }
            Some("=") => {
                // we hit an equal sign, so we are done
                break;
            }
            Some(" ") => {
                // we hit a whitespace, so we are done
                break;
            }
            Some("\r") if remaining.get(1..2) == Some("\n") => {
                // we hit a newline, so we are done
                break;
            }
            Some("\r") => {}
            Some("\n") => {
                // we hit a newline, so we are done
                break;
            }
            Some("[") => {
                // find the closing bracket
                let closing_bracket = ']';

                let mut closing_bracket_index = 0;
                let mut is_escaped = false;

                for (index, c) in remaining.chars().enumerate() {
                    if index == 0 {
                        // skip first char, because it is the opening bracket
                        continue;
                    }

                    if c == '\\' {
                        is_escaped = true;
                        continue;
                    }

                    if c == closing_bracket && !is_escaped {
                        closing_bracket_index = index;
                        break;
                    }

                    is_escaped = false;
                }

                if closing_bracket_index == 0 {
                    return Err(nom::Err::Error(Error::new(remaining, ErrorKind::Tag)));
                }

                attribute_key_index += closing_bracket_index;
                remaining = input.get(attribute_key_index..).unwrap();

                continue;
            }
            Some("(") => {
                // find the closing brace
                let closing_brace = ')';

                let mut closing_brace_index = 0;
                let mut is_escaped = false;

                for (index, c) in remaining.chars().enumerate() {
                    if index == 0 {
                        // skip first char, because it is the opening brace
                        continue;
                    }

                    if c == '\\' {
                        is_escaped = true;
                        continue;
                    }

                    if c == closing_brace && !is_escaped {
                        closing_brace_index = index;
                        break;
                    }

                    is_escaped = false;
                }

                if closing_brace_index == 0 {
                    return Err(nom::Err::Error(Error::new(remaining, ErrorKind::Tag)));
                }

                attribute_key_index += closing_brace_index;
                remaining = input.get(attribute_key_index + 1..).unwrap();

                continue;
            }
            Some(_) => {
                attribute_key_index += 1;
                remaining = remaining.get(1..).unwrap();
                continue;
            }
            None => {
                return Err(nom::Err::Incomplete(Needed::Unknown));
            }
        }
    }

    let attribute_key = input.get(..attribute_key_index).unwrap();

    Ok((remaining, attribute_key))
}

fn process_attribute_value<'a>(
    input: &'a str,
    _context: &mut HsmlProcessContext,
) -> IResult<&'a str, &'a str> {
    // get first char
    let first_char = input.chars().next().unwrap();

    // if first char is a quote, then we need to find the closing quote and return the value in between (together with the surrounding quotes)
    if first_char == '"' || first_char == '\'' {
        let closing_quote = if first_char == '"' { '"' } else { '\'' };

        let mut closing_quote_index = 0;
        let mut is_escaped = false;

        for (index, c) in input.chars().enumerate() {
            if index == 0 {
                // skip first char, because it is the opening quote
                continue;
            }

            if c == '\\' {
                is_escaped = true;
                continue;
            }

            if c == closing_quote && !is_escaped {
                closing_quote_index = index;
                break;
            }

            is_escaped = false;
        }

        if closing_quote_index == 0 {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }

        let attribute_value = input.get(1..closing_quote_index).unwrap();

        // dbg!(attribute_value);

        return Ok((
            input.get(closing_quote_index + 1..).unwrap_or(""),
            attribute_value,
        ));
    }

    // otherwise it was not a valid attribute value
    Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
}

// An attribute key can only contain a-z, A-Z, 0-9, `-`, `_`, `:`, `#`, `@`, `[`, `]`, `(`, `)`, `{`, `}`
// There is the special case that an attribute key can contain a dot (`.`) if it is followed by a letter
// There is the special case that an attribute key can contain a space (` `) if it is surrounded by quotes (`"`)
// Quotes can only contained if they are surrounded by quotes (`"`)
// An attribute key must start with a-z, A-Z, `:`, `#`, `@`, `[`, `(`

// First take until the first potential equal sign (`=`)
//  If there is an equal sign, then test the output for being a valid attribute key
//  If there is no equal sign, then the attribute might be a boolean attribute

// If the attribute is a boolean attribute, then return the attribute and the remaining input

pub fn process_attribute<'a>(
    input: &'a str,
    context: &mut HsmlProcessContext,
) -> IResult<&'a str, &'a str> {
    let (remaining, attribute_key) = process_attribute_key(input)?;

    // check if remaining starts with an equal sign
    if let Ok((remaining_after_equal_sign, _)) = tag::<&str, &str, Error<&str>>("=")(remaining) {
        let (remaining_after_attribute_value, _attribute_value) =
            process_attribute_value(remaining_after_equal_sign, context)?;

        let attribute = input
            .get(..input.len() - remaining_after_attribute_value.len())
            .unwrap();

        return Ok((remaining_after_attribute_value, attribute));
    }

    Ok((remaining, attribute_key))
}

#[cfg(test)]
mod tests {
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

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

        assert_eq!(attribute, r#"src="https://github.com/""#);
        assert_eq!(rest, "");
    }

    #[test]
    fn it_should_process_attribute_without_value() {
        let input = "disabled ";

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

        assert_eq!(attribute, "disabled");
        assert_eq!(rest, " ");
    }

    #[test]
    fn it_should_process_attribute_followed_by_another_attribute() {
        let input = "disabled required";

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

        assert_eq!(attribute, "disabled");
        assert_eq!(rest, " required");
    }

    #[test]
    fn it_should_process_attribute_followed_by_another_attribute_separated_by_comma() {
        let input = "disabled, required";

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

        assert_eq!(attribute, "disabled");
        assert_eq!(rest, ", required");
    }

    #[test]
    fn it_should_process_attribute_with_angular_binding() {
        let input = r#"color="{{ color }}", required"#;

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

        assert_eq!(attribute, r#"color="{{ color }}""#);
        assert_eq!(rest, ", required");
    }

    #[test]
    fn it_should_process_attribute_with_angular_ng_model() {
        let input = r#"[(ngModel)]="name", required"#;

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

        assert_eq!(attribute, r#"[(ngModel)]="name""#);
        assert_eq!(rest, ", required");
    }

    #[test]
    fn it_should_process_attribute_with_angular_event() {
        let input = r#"(click)="setValue()", required"#;

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

        assert_eq!(attribute, r#"(click)="setValue()""#);
        assert_eq!(rest, ", required");
    }

    #[test]
    fn it_should_process_attribute_with_vue_binding() {
        let input = r#":src="image", alt="Image""#;

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

        assert_eq!(attribute, r#":src="image""#);
        assert_eq!(rest, r#", alt="Image""#);
    }

    #[test]
    fn it_should_process_attribute_with_vue_event() {
        let input = r#"@click="setValue()", color="primary""#;

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

        assert_eq!(attribute, r#"@click="setValue()""#);
        assert_eq!(rest, r#", color="primary""#);
    }

    #[test]
    fn it_should_process_attribute_with_vue_slot() {
        let input = r#"#header="slot""#;

        let (rest, attribute) =
            process_attribute(input, &mut HsmlProcessContext::default()).unwrap();

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
}
