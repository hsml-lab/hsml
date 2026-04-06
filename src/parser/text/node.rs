use crate::parser::{HsmlProcessContext, HsmlResult, Span};

use super::process::{process_text, process_text_block};

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextNode {
    pub text: String,
    /// Whether this text was written as a block (trailing dot syntax).
    pub is_block: bool,
}

pub fn text_block_node<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, TextNode> {
    let (input, text) = process_text_block(input, context)?;

    let text_str = *text.fragment();

    // Strip the first non-empty line's indentation prefix from each line.
    // This removes the text block's structural indentation but also removes
    // any content whitespace that matches (a known HSML language limitation
    // for whitespace-sensitive content like <pre> tags).
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

    Ok((
        input,
        TextNode {
            text,
            is_block: true,
        },
    ))
}

pub fn text_node(input: Span<'_>) -> HsmlResult<'_, TextNode> {
    let (input, text) = process_text(input)?;

    Ok((
        input,
        TextNode {
            text: text.to_string(),
            is_block: false,
        },
    ))
}
