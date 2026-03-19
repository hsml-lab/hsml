use nom::{IResult, bytes::complete::tag, bytes::complete::take_while1};

pub(super) fn process_id(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("#")(input)?;

    let (input, id) =
        take_while1(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')(input)?;

    Ok((input, id))
}
