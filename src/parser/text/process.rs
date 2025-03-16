use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until1},
};

use crate::parser::HsmlProcessContext;

pub fn process_text_block<'a>(
    input: &'a str,
    context: &mut HsmlProcessContext,
) -> IResult<&'a str, &'a str> {
    let (rest, _) = tag(".")(input)?;

    // eat one \r\n or \n
    let (rest, _) = alt((tag("\r\n"), tag("\n"))).parse(rest)?;

    let mut text_block_index = 0;

    // loop over each line until we find a line that does not starts with the current indent string
    for (index, c) in rest.chars().enumerate() {
        if c == '\n' {
            // if next char is also a \n, then continue
            let next_char = rest.chars().nth(index + 1);
            if next_char == Some('\n') {
                text_block_index = index + 1;
                continue;
            }

            let line = &rest[index + 1..];

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
            text_block_index = index;
            continue;
        }
    }

    let text_block = &rest[..text_block_index + 1];

    let rest = &rest[text_block_index + 1..];

    Ok((rest, text_block))
}

pub fn process_text(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag(" ")(input)?;
    take_until1("\n")(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        HsmlProcessContext,
        text::process::{process_text, process_text_block},
    };

    #[test]
    fn it_should_process_text_block() {
        let mut context = HsmlProcessContext {
            nested_tag_level: 1,
            indent_string: String::from("  "),
        };

        let input = r#".
   this is just some text
    it can be multiline

    	and also contain blank lines
span other text
"#;

        let (rest, text_block) = process_text_block(input, &mut context).unwrap();

        assert_eq!(
            text_block,
            r#"   this is just some text
    it can be multiline

    	and also contain blank lines"#
        );
        assert_eq!(
            rest,
            r#"
span other text
"#
        );
    }

    #[test]
    fn test_process_text() {
        let input = " hello world\n";

        let (rest, text) = process_text(input).unwrap();

        assert_eq!(text, "hello world");
        assert_eq!(rest, "\n");
    }
}
