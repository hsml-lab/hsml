use nom::{IResult, bytes::complete::tag, bytes::complete::take_while1};

use crate::parser::Span;

pub(super) fn process_id(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    let (input, _) = tag("#")(input)?;

    // HTML5 IDs can contain any characters except ASCII whitespace.
    // We allow alphanumeric (including Unicode), hyphens, and underscores.
    let (input, id) = take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)?;

    Ok((input, id))
}
