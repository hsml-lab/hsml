use nom::{IResult, bytes::complete::tag};

pub(super) fn process_id(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("#")(input)?;

    let (input, id) = nom::character::complete::alphanumeric1(input)?;

    Ok((input, id))
}
