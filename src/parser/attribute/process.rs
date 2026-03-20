use nom::{
    IResult, Needed,
    bytes::complete::tag,
    error::{Error, ErrorKind},
};

use crate::parser::{HsmlProcessContext, Span, advance, take_prefix};

fn is_valid_attribute_key_start(c: char) -> bool {
    c.is_alphabetic() || c == ':' || c == '#' || c == '@' || c == '[' || c == '('
}

pub(super) fn process_attribute_key(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    let Some(first_char) = input.fragment().chars().next() else {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::AlphaNumeric)));
    };

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
        let first_char = remaining.fragment().chars().next();

        match first_char {
            Some(')') => {
                // we hit the end of the attributes, so we are done
                break;
            }
            Some(',') => {
                // we hit a comma, so we are done
                break;
            }
            Some('=') => {
                // we hit an equal sign, so we are done
                break;
            }
            Some(' ') => {
                // we hit a whitespace, so we are done
                break;
            }
            Some('\r') if remaining.fragment().as_bytes().get(1) == Some(&b'\n') => {
                // we hit a newline, so we are done
                break;
            }
            Some('\r') => {
                // lone \r (old Mac line ending) — treat as line ending
                break;
            }
            Some('\n') => {
                // we hit a newline, so we are done
                break;
            }
            Some('[') => {
                // find the closing bracket
                let closing_bracket = ']';

                let mut closing_bracket_index = 0;
                let mut is_escaped = false;

                for (index, c) in remaining.fragment().char_indices() {
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
                remaining = advance(input, attribute_key_index);

                continue;
            }
            Some('(') => {
                // find the closing brace
                let closing_brace = ')';

                let mut closing_brace_index = 0;
                let mut is_escaped = false;

                for (index, c) in remaining.fragment().char_indices() {
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
                remaining = advance(input, attribute_key_index + 1);

                continue;
            }
            Some(_) => {
                attribute_key_index += remaining.fragment().chars().next().unwrap().len_utf8();
                remaining = advance(input, attribute_key_index);
                continue;
            }
            None => {
                return Err(nom::Err::Incomplete(Needed::Unknown));
            }
        }
    }

    let attribute_key = take_prefix(input, attribute_key_index);

    Ok((remaining, attribute_key))
}

pub(super) fn process_attribute_value<'a>(
    input: Span<'a>,
    _context: &mut HsmlProcessContext,
) -> IResult<Span<'a>, Span<'a>> {
    // get first char
    let Some(first_char) = input.fragment().chars().next() else {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    };

    // if first char is a quote, then we need to find the closing quote and return the value in between (together with the surrounding quotes)
    if first_char == '"' || first_char == '\'' {
        let closing_quote = if first_char == '"' { '"' } else { '\'' };

        let mut closing_quote_index = 0;
        let mut is_escaped = false;

        for (index, c) in input.fragment().char_indices() {
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

        // value between quotes (excluding the quotes themselves)
        let attribute_value = take_prefix(advance(input, 1), closing_quote_index - 1);

        let remaining = advance(input, closing_quote_index + 1);

        return Ok((remaining, attribute_value));
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

pub(super) fn process_attribute<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> IResult<Span<'a>, Span<'a>> {
    let (remaining, attribute_key) = process_attribute_key(input)?;

    // check if remaining starts with an equal sign
    if let Ok((remaining_after_equal_sign, _)) = tag::<&str, Span, Error<Span>>("=")(remaining) {
        let (remaining_after_attribute_value, _attribute_value) =
            process_attribute_value(remaining_after_equal_sign, context)?;

        let consumed = input.fragment().len() - remaining_after_attribute_value.fragment().len();
        let attribute = take_prefix(input, consumed);

        return Ok((remaining_after_attribute_value, attribute));
    }

    Ok((remaining, attribute_key))
}
