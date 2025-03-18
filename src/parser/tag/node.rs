use nom::{
    IResult,
    bytes::complete::{take_till, take_till1},
    error::{Error, ErrorKind},
};

use crate::parser::{
    HsmlNode, HsmlProcessContext, attribute,
    class::node::{ClassNode, class_node},
    comment::node::{comment_dev_node, comment_native_node},
    id::{self, node::IdNode},
    tag::process::process_tag,
    text::{self, node::TextNode},
};

#[derive(Debug, PartialEq)]
pub struct TagNode {
    pub tag: String,
    pub id: Option<IdNode>,
    pub classes: Option<Vec<ClassNode>>,
    pub attributes: Option<Vec<HsmlNode>>,
    pub text: Option<TextNode>,
    pub children: Option<Vec<HsmlNode>>,
}

pub fn tag_node<'a>(input: &'a str, context: &mut HsmlProcessContext) -> IResult<&'a str, TagNode> {
    // tag node starts with a tag name or a dot/hash
    // if it starts with a dot/hash, the tag name is div

    let (mut input, tag_name) = if input.starts_with('.') || input.starts_with('#') {
        (input, "div")
    } else {
        process_tag(input)?
    };

    // if the next char is a dot, we have a id node
    // if the next char is a dot, we have a class node
    // collect id and class nodes until we hit a whitespace, newline, start of attributes or single dot without trailing alphabetical char

    let mut id_node: Option<IdNode> = None;
    let mut class_nodes: Vec<ClassNode> = vec![];
    let mut attribute_nodes: Option<Vec<HsmlNode>> = None;
    let mut text_node: Option<TextNode> = None;
    let mut child_nodes: Vec<HsmlNode> = vec![];

    loop {
        let first_char = input.get(..1);
        let first_two_chars = input.get(..2);

        if first_char == Some("#") {
            // we hit an id node

            // if there was already an id node, throw an error
            if id_node.is_some() {
                // TODO @Shinigami92 2023-05-25: This error could be more specific
                // Duplicate attribute "id" is not allowed.
                return Err(nom::Err::Failure(Error::new(input, ErrorKind::Tag)));
            }

            let (rest, node) = id::node::id_node(input)?;
            id_node = Some(node);
            input = rest;

            continue;
        }

        if first_char == Some(".") {
            if first_two_chars == Some(".\n") {
                // we hit piped text
                let (rest, node) = text::node::text_block_node(input, context)?;
                text_node = Some(node);
                input = rest;

                break;
            }

            // we hit a class node
            let (rest, node) = class_node(input)?;
            class_nodes.push(node);
            input = rest;

            continue;
        }

        if first_char == Some("(") {
            // we hit the start of attributes

            let (rest, nodes) = attribute::node::attribute_nodes(input, context)?;
            attribute_nodes = Some(nodes);
            input = rest;

            continue;
        }

        if first_char == Some(" ") {
            // we hit a whitespace and there should be text

            let (rest, node) = text::node::text_node(input)?;
            text_node = Some(node);
            input = rest;

            // TODO @Shinigami92 2023-05-22: Theoretically here could also follow a comment

            // there could be child tag nodes, but this will be handled in the next loop iteration by the line ending check

            break;
        }

        if first_char == Some("\n") || first_two_chars == Some("\r\n") {
            // we hit a newline and the tag ended but could have child tag nodes

            // check indentation
            let (rest, _) = take_till1(|c| c != '\r' && c != '\n')(input)?;

            // check if the next char is a tab or whitespace
            // if yes, check for indentation level
            // if no, we have no child tag nodes and can break the loop

            let (remaining, indentation) = take_till(|c: char| !c.is_whitespace())(rest)?;

            if !indentation.is_empty() {
                // check that the indentation is consistent and does not include tabs and spaces at the same time
                // if it does, collect an error for diagnostics

                if indentation.contains('\t') && indentation.contains(' ') {
                    // TODO @Shinigami92 2025-03-16: This should collect an error or diagnostics
                }

                // persist the indentation level so we can restore it later
                let nested_tag_level = context.nested_tag_level;
                let indent_string = context.indent_string.clone();

                // check that we are at the correct indentation level, otherwise break out of the loop
                if indentation.len() <= context.indent_string.len() {
                    // dbg!("break out of loop");
                    break;
                }

                context.nested_tag_level += 1;
                context.indent_string = indentation.to_string();

                // we are at the correct indentation level, so we can continue parsing the child tag nodes

                // there could be a comment (dev or native) node
                if let Ok((rest, node)) = comment_native_node(remaining) {
                    child_nodes.push(HsmlNode::Comment(node));
                    input = rest;
                } else if let Ok((rest, node)) = comment_dev_node(remaining) {
                    child_nodes.push(HsmlNode::Comment(node));
                    input = rest;
                }
                // or we have now a child tag node
                else {
                    // now we have a child tag node
                    if let Ok((rest, node)) = tag_node(remaining, context) {
                        child_nodes.push(HsmlNode::Tag(node));
                        input = rest;
                    } else {
                        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
                    }
                }

                // restore the nested_tag_level level
                context.nested_tag_level = nested_tag_level;
                context.indent_string = indent_string;

                continue;
            }

            // we have no child tag nodes
            break;
        }

        break;
    }

    Ok((
        input,
        TagNode {
            tag: tag_name.to_string(),
            id: id_node,
            classes: (!class_nodes.is_empty()).then_some(class_nodes),
            attributes: attribute_nodes,
            text: text_node,
            children: (!child_nodes.is_empty()).then_some(child_nodes),
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        HsmlProcessContext,
        class::node::ClassNode,
        tag::node::{TagNode, tag_node},
        text::node::TextNode,
    };

    #[test]
    fn it_should_return_tag_node_with_piped_text() {
        let context = &mut HsmlProcessContext {
            nested_tag_level: 3,
            indent_string: String::from("      "),
        };

        let (input, tag) = tag_node(
            r#"p.text-lg.font-medium.
        "Tailwind CSS is the only framework that I've seen scale
        on large teams. It's easy to customize, adapts to any design,
        and the build size is tiny."
    figcaption.font-medium"#,
            context,
        )
        .unwrap();

        assert_eq!(
            tag,
            TagNode {
                tag: String::from("p"),
                id: None,
                classes: Some(vec![
                    ClassNode {
                        name: String::from("text-lg"),
                    },
                    ClassNode {
                        name: String::from("font-medium"),
                    },
                ]),
                attributes: None,
                text: Some(TextNode {
                    text: String::from(
                        r#""Tailwind CSS is the only framework that I've seen scale
on large teams. It's easy to customize, adapts to any design,
and the build size is tiny.""#
                    ),
                }),
                children: None,
            }
        );

        assert_eq!(input, "\n    figcaption.font-medium");
    }
}
