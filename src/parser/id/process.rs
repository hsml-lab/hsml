use nom::{IResult, bytes::complete::tag, bytes::complete::take_while1};

pub(super) fn process_id(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("#")(input)?;

    // HTML5 IDs can contain any characters except ASCII whitespace.
    // We allow alphanumeric (including Unicode), hyphens, and underscores.
    let (input, id) = take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)?;

    Ok((input, id))
}
