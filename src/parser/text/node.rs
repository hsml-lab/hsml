use nom::IResult;

use crate::parser::HsmlProcessContext;

use super::process::{process_text, process_text_block};

#[derive(Debug, PartialEq, Eq)]
pub struct TextNode {
    pub text: String,
}

pub fn text_block_node<'a>(
    input: &'a str,
    context: &mut HsmlProcessContext,
) -> IResult<&'a str, TextNode> {
    let (input, text) = process_text_block(input, context)?;

    // On every line, replace all leading spaces and tabs with an empty string
    let text = text
        .lines()
        .map(|line| line.trim_start())
        .collect::<Vec<&str>>()
        .join("\n");

    Ok((input, TextNode { text }))
}

pub fn text_node(input: &str) -> IResult<&str, TextNode> {
    let (input, text) = process_text(input)?;

    Ok((
        input,
        TextNode {
            text: text.to_string(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        HsmlProcessContext,
        text::node::{TextNode, text_block_node},
    };

    #[test]
    fn it_should_return_text_block_node() {
        let context = &mut HsmlProcessContext {
            nested_tag_level: 3,
            indent_string: String::from("      "),
        };

        let (input, text_block) = text_block_node(
            r#".
        "Tailwind CSS is the only framework that I've seen scale
        on large teams. It's easy to customize, adapts to any design,
        and the build size is tiny."
    figcaption.font-medium"#,
            context,
        )
        .unwrap();

        assert_eq!(
            text_block,
            TextNode {
                text: String::from(
                    r#""Tailwind CSS is the only framework that I've seen scale
on large teams. It's easy to customize, adapts to any design,
and the build size is tiny.""#
                ),
            }
        );

        assert_eq!(input, "\n    figcaption.font-medium");
    }

    #[test]
    fn it_should_stop_before_next_tag_node() {
        let context = &mut HsmlProcessContext {
            nested_tag_level: 1,
            indent_string: String::from("  "),
        };

        let (input, text_block) = text_block_node(
            r#".
    Sarah Dayan
  .text-[#af05c9].dark:text-slate-500.
    Staff Engineer, Algolia"#,
            context,
        )
        .unwrap();

        assert_eq!(
            text_block,
            TextNode {
                text: String::from(r#"Sarah Dayan"#),
            }
        );

        assert_eq!(
            input,
            "\n  .text-[#af05c9].dark:text-slate-500.\n    Staff Engineer, Algolia"
        );
    }
}
