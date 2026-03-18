use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until1},
};

use crate::parser::HsmlProcessContext;

pub(super) fn process_text_block<'a>(
    input: &'a str,
    context: &mut HsmlProcessContext,
) -> IResult<&'a str, &'a str> {
    let (rest, _) = tag(".")(input)?;

    // eat one \r\n or \n
    let (rest, _) = alt((tag("\r\n"), tag("\n"))).parse(rest)?;

    let mut text_block_end = 0;

    // Validate first line as well (the loop below only validates lines after a '\n').
    if let Some(first_line) = rest.lines().next()
        && !first_line.is_empty()
    {
        if !first_line.starts_with(&context.indent_string) {
            return Ok((rest, &rest[..0]));
        }

        let after_indent = &first_line[context.indent_string.len()..];
        if !after_indent.starts_with(' ') && !after_indent.starts_with('\t') {
            return Ok((rest, &rest[..0]));
        }
    }

    // loop over each line until we find a line that does not starts with the current indent string
    for (index, c) in rest.char_indices() {
        if c == '\n' {
            // if next char is also a \n, then continue
            let line_start = index + 1;
            let next_char = rest[line_start..].chars().next();
            if next_char == Some('\n') {
                text_block_end = line_start + 1;
                continue;
            }

            let line = &rest[line_start..];

            // otherwise check the indentation and if it does not fulfill the indentation, then break
            // TODO @Shinigami92 2025-03-16: right now this does not support mixed indentations on tag level indentation, but only withing the text block
            if !line.starts_with(&context.indent_string) {
                break;
            }

            let line = &line[context.indent_string.len()..];

            // break out if the first character is not a space or tab
            if !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
        } else {
            text_block_end = index + c.len_utf8();
            continue;
        }
    }

    let text_block = &rest[..text_block_end];

    let rest = &rest[text_block_end..];

    Ok((rest, text_block))
}

pub(super) fn process_text(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag(" ")(input)?;
    take_until1("\n")(input)
}
