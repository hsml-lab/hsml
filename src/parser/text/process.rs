use nom::{
    Parser,
    branch::alt,
    bytes::complete::{tag, take_until1},
};

use crate::parser::{HsmlProcessContext, HsmlResult, Span, advance, take_prefix};

/// Check if a line belongs to the current text block based on indentation.
/// A line belongs if it starts with the indent string followed by a space or tab.
fn is_text_block_line(line: &str, indent: &str) -> bool {
    if let Some(after_indent) = line.strip_prefix(indent) {
        after_indent.starts_with(' ') || after_indent.starts_with('\t')
    } else {
        false
    }
}

/// Find the newline length at the given position (1 for `\n`, 2 for `\r\n`, 0 if none).
fn newline_len_at(s: &str) -> usize {
    if s.starts_with("\r\n") {
        2
    } else if s.starts_with('\n') {
        1
    } else {
        0
    }
}

pub(super) fn process_text_block<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, Span<'a>> {
    let (rest, _) = tag(".")(input)?;

    // eat one \r\n or \n
    let (rest, _) = alt((tag("\r\n"), tag("\n"))).parse(rest)?;

    let rest_str = *rest.fragment();

    // Validate first line
    if let Some(first_line) = rest_str.lines().next()
        && !first_line.is_empty()
        && !is_text_block_line(first_line, &context.indent_string)
    {
        return Ok((rest, take_prefix(rest, 0)));
    }

    // Scan line by line to find where the text block ends
    let mut text_block_end = 0;
    let mut pos = 0;

    while pos < rest_str.len() {
        // Validate the current line (skip for first iteration — already validated above)
        if pos > 0 {
            let line_end = rest_str[pos..]
                .find('\n')
                .map_or(rest_str.len(), |i| pos + i);
            let line = &rest_str[pos..line_end].trim_end_matches('\r');

            if !line.is_empty() && !is_text_block_line(line, &context.indent_string) {
                break;
            }
        }

        // Find the end of the current line
        let line_end = rest_str[pos..]
            .find('\n')
            .map_or(rest_str.len(), |i| pos + i);

        // Include the content up to the end of this line
        text_block_end = line_end;

        // If there's no newline, we've reached the end of input
        if line_end >= rest_str.len() {
            break;
        }

        // Move past the newline
        let next_pos = line_end + 1;

        // Check if the next line is blank (consecutive newline or \r\n)
        let nl = newline_len_at(&rest_str[next_pos..]);
        if nl > 0 {
            text_block_end = next_pos + nl;
            pos = next_pos + nl;
            continue;
        }

        // Check if the next line belongs to the text block
        let next_line_end = rest_str[next_pos..]
            .find('\n')
            .map_or(rest_str.len(), |i| next_pos + i);
        let next_line = rest_str[next_pos..next_line_end].trim_end_matches('\r');

        // TODO @Shinigami92 2025-03-16: right now this does not support mixed indentations on tag level indentation, but only within the text block
        if !is_text_block_line(next_line, &context.indent_string) {
            break;
        }

        pos = next_pos;
    }

    let text_block = take_prefix(rest, text_block_end);
    let rest = advance(rest, text_block_end);

    Ok((rest, text_block))
}

pub(super) fn process_text(input: Span<'_>) -> HsmlResult<'_, Span<'_>> {
    let (input, _) = tag(" ")(input)?;
    take_until1("\n")(input)
}
