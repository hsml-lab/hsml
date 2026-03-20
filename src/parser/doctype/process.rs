use nom::{IResult, bytes::complete::tag};

use crate::parser::Span;

pub(super) fn process_doctype(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    let (input, _) = tag("doctype ")(input)?;

    let (input, doctype) = tag("html")(input)?;

    Ok((input, doctype))
}
