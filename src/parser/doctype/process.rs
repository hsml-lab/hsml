use nom::bytes::complete::tag;

use crate::parser::{HsmlResult, Span};

pub(super) fn process_doctype(input: Span<'_>) -> HsmlResult<'_, Span<'_>> {
    let (input, _) = tag("doctype ")(input)?;

    let (input, doctype) = tag("html")(input)?;

    Ok((input, doctype))
}
