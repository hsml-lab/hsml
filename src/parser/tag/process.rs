use nom::{Needed, bytes::complete::take_till1};

use crate::parser::{HsmlResult, Span};

fn starts_with_ascii_alphabetic(s: &str) -> bool {
    if let Some(c) = s.chars().next() {
        c.is_ascii_alphabetic()
    } else {
        false
    }
}

pub fn process_tag(input: Span<'_>) -> HsmlResult<'_, Span<'_>> {
    let (input, tag_name) = take_till1(|c: char| c != '-' && !c.is_ascii_alphanumeric())(input)?;

    if starts_with_ascii_alphabetic(&tag_name) {
        Ok((input, tag_name))
    } else {
        Err(nom::Err::Incomplete(Needed::Unknown))
    }
}
