use nom::{
    bytes::complete::{tag, take_until},
    error::ErrorKind,
};

use crate::parser::{HsmlResult, Span, error::HsmlError};

pub(super) fn process_dev_comment(input: Span<'_>) -> HsmlResult<'_, Span<'_>> {
    let (input, _) = tag("//")(input)?;

    // check next char is not a `!`
    if input.starts_with('!') {
        return Err(nom::Err::Error(HsmlError::from_kind(input, ErrorKind::Tag)));
    }

    // read until end of line
    let (input, comment) = take_until("\n")(input)?;

    Ok((input, comment))
}

pub(super) fn process_native_comment(input: Span<'_>) -> HsmlResult<'_, Span<'_>> {
    let (input, _) = tag("//!")(input)?;

    // read until end of line
    let (input, comment) = take_until("\n")(input)?;

    Ok((input, comment))
}
