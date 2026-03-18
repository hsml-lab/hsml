use nom::{
    IResult,
    bytes::complete::{tag, take_until},
    error::{Error, ErrorKind},
};

pub(super) fn process_dev_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("//")(input)?;

    // check next char is not a `!`
    if let Some(c) = input.chars().next() {
        if c == '!' {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
    }

    // read until end of line
    let (input, comment) = take_until("\n")(input)?;

    Ok((input, comment))
}

pub(super) fn process_native_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("//!")(input)?;

    // read until end of line
    let (input, comment) = take_until("\n")(input)?;

    Ok((input, comment))
}
