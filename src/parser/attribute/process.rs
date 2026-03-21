use nom::{
    bytes::complete::{tag, take_while1},
    error::ErrorKind,
};

use crate::parser::{
    HsmlProcessContext, HsmlResult, Span, advance, delimited_section_len, error::HsmlError,
    quoted_string_len, take_prefix,
};

fn is_valid_attribute_key_start(c: char) -> bool {
    c.is_alphabetic() || c == ':' || c == '#' || c == '@' || c == '[' || c == '('
}

/// Returns true if the character is an attribute key delimiter (stops parsing).
fn is_key_delimiter(c: char) -> bool {
    matches!(c, ')' | ',' | '=' | ' ' | '\r' | '\n' | '[' | '(')
}

pub(super) fn process_attribute_key(input: Span<'_>) -> HsmlResult<'_, Span<'_>> {
    let Some(first_char) = input.fragment().chars().next() else {
        return Err(HsmlError::err(input, ErrorKind::AlphaNumeric));
    };

    if first_char.is_numeric() {
        return Err(HsmlError::err(input, ErrorKind::AlphaNumeric));
    }

    if !is_valid_attribute_key_start(first_char) {
        return Err(HsmlError::err(input, ErrorKind::AlphaNumeric));
    }

    let mut remaining = input;
    let mut key_len = 0;

    loop {
        // Consume regular (non-delimiter) characters
        if let Ok((rest, taken)) =
            take_while1::<_, Span, HsmlError>(|c: char| !is_key_delimiter(c))(remaining)
        {
            key_len += taken.fragment().len();
            remaining = rest;
            continue;
        }

        // Check for bracket section [...]
        if remaining.fragment().starts_with('[') {
            if let Some(len) = delimited_section_len(remaining.fragment(), '[', ']') {
                key_len += len;
                remaining = advance(input, key_len);
                continue;
            }
            return Err(HsmlError::err(remaining, ErrorKind::Tag));
        }

        // Check for parenthesized section (...)
        if remaining.fragment().starts_with('(') {
            if let Some(len) = delimited_section_len(remaining.fragment(), '(', ')') {
                key_len += len;
                remaining = advance(input, key_len);
                continue;
            }
            return Err(HsmlError::err(remaining, ErrorKind::Tag));
        }

        // Any other character is a delimiter — stop
        break;
    }

    let attribute_key = take_prefix(input, key_len);

    Ok((remaining, attribute_key))
}

pub(super) fn process_attribute_value<'a>(
    input: Span<'a>,
    _context: &mut HsmlProcessContext,
) -> HsmlResult<'a, Span<'a>> {
    let Some(first_char) = input.fragment().chars().next() else {
        return Err(HsmlError::err(input, ErrorKind::Tag));
    };

    if first_char != '"' && first_char != '\'' {
        return Err(HsmlError::err(input, ErrorKind::Tag));
    }

    if let Some(len) = quoted_string_len(input.fragment()) {
        // value between quotes (excluding the quotes themselves)
        let attribute_value = take_prefix(advance(input, 1), len - 2);
        let remaining = advance(input, len);

        return Ok((remaining, attribute_value));
    }

    // Unclosed quote
    Err(HsmlError::err(input, ErrorKind::Tag))
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
) -> HsmlResult<'a, Span<'a>> {
    let (remaining, attribute_key) = process_attribute_key(input)?;

    // check if remaining starts with an equal sign
    if let Ok((remaining_after_equal_sign, _)) = tag::<&str, Span, HsmlError>("=")(remaining) {
        let (remaining_after_attribute_value, _attribute_value) =
            process_attribute_value(remaining_after_equal_sign, context)?;

        let consumed = input.fragment().len() - remaining_after_attribute_value.fragment().len();
        let attribute = take_prefix(input, consumed);

        return Ok((remaining_after_attribute_value, attribute));
    }

    Ok((remaining, attribute_key))
}
