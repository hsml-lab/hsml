use crate::parser::{HsmlProcessContext, HsmlResult, Span};

use super::process::{process_text, process_text_block};

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct TextNode {
    pub text: String,
}

pub fn text_block_node<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, TextNode> {
    let (input, text) = process_text_block(input, context)?;

    let text_str = *text.fragment();

    // Strip the first non-empty line's indentation prefix from each line
    let block_indent = text_str
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .collect::<String>()
        })
        .unwrap_or_default();
    let text = text_str
        .lines()
        .map(|line| line.strip_prefix(&block_indent).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");

    Ok((input, TextNode { text }))
}

pub fn text_node(input: Span<'_>) -> HsmlResult<'_, TextNode> {
    let (input, text) = process_text(input)?;

    Ok((
        input,
        TextNode {
            text: text.to_string(),
        },
    ))
}
