use nom::{IResult, bytes::complete::tag};

pub(super) fn process_doctype(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("doctype ")(input)?;

    let (input, doctype) = tag("html")(input)?;

    Ok((input, doctype))
}
