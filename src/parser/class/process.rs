use nom::{
    bytes::complete::{tag, take_while1},
    error::ErrorKind,
};

use crate::parser::{
    HsmlResult, Span, advance, delimited_section_len, error::HsmlError, take_prefix,
};

/// Returns true if the character is a class name delimiter (stops parsing).
fn is_class_delimiter(c: char) -> bool {
    matches!(c, '#' | '.' | '(' | ' ' | '\t' | '\r' | '\n' | '[')
}

pub(super) fn process_class(input: Span<'_>) -> HsmlResult<'_, Span<'_>> {
    let (input, _) = tag(".")(input)?;

    let mut remaining = input;
    let mut class_len = 0;

    loop {
        // Consume regular (non-delimiter, non-bracket) characters
        if let Ok((rest, taken)) =
            take_while1::<_, Span, HsmlError>(|c: char| !is_class_delimiter(c))(remaining)
        {
            class_len += taken.fragment().len();
            remaining = rest;
            continue;
        }

        // Check for bracket section
        if remaining.fragment().starts_with('[') {
            if let Some(len) = delimited_section_len(remaining.fragment(), '[', ']') {
                class_len += len;
                remaining = advance(input, class_len);
                continue;
            }
            return Err(HsmlError::err(remaining, ErrorKind::Tag));
        }

        // Any other character is a delimiter — stop
        break;
    }

    if class_len == 0 {
        return Err(HsmlError::err(input, ErrorKind::Alpha));
    }

    let class = take_prefix(input, class_len);

    Ok((remaining, class))
}
