use nom::IResult;

use crate::parser::Span;

use super::process::{process_dev_comment, process_native_comment};

#[derive(Debug, PartialEq, Eq)]
pub struct CommentNode {
    pub text: String,
    pub is_dev: bool,
}

pub fn comment_dev_node(input: Span<'_>) -> IResult<Span<'_>, CommentNode> {
    let (input, comment) = process_dev_comment(input)?;

    Ok((
        input,
        CommentNode {
            text: comment.to_string(),
            is_dev: true,
        },
    ))
}

pub fn comment_native_node(input: Span<'_>) -> IResult<Span<'_>, CommentNode> {
    let (input, comment) = process_native_comment(input)?;

    Ok((
        input,
        CommentNode {
            text: comment.to_string(),
            is_dev: false,
        },
    ))
}
