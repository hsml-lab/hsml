use nom::bytes::complete::{take_till, take_till1};

use crate::parser::{
    HsmlNode, HsmlProcessContext, HsmlResult, Span, attribute,
    class::node::{ClassNode, class_node},
    comment::node::{comment_dev_node, comment_native_node},
    id::{self, node::IdNode},
    tag::process::process_tag,
    text::{self, node::TextNode},
};

#[derive(Debug, PartialEq)]
pub struct TagNode {
    pub tag: String,
    /// All id selectors on this tag. Only the first is used in compilation;
    /// duplicates are reported as warnings by the validator.
    pub ids: Vec<IdNode>,
    pub classes: Option<Vec<ClassNode>>,
    pub attributes: Option<Vec<HsmlNode>>,
    pub text: Option<TextNode>,
    pub children: Option<Vec<HsmlNode>>,
}

pub fn tag_node<'a>(input: Span<'a>, context: &mut HsmlProcessContext) -> HsmlResult<'a, TagNode> {
    // tag node starts with a tag name or a dot/hash
    // if it starts with a dot/hash, the tag name is div

    let (mut input, tag_name) = if input.starts_with('.') || input.starts_with('#') {
        (input, "div")
    } else {
        let (rest, name) = process_tag(input)?;
        (rest, *name.fragment())
    };

    // if the next char is a dot, we have a id node
    // if the next char is a dot, we have a class node
    // collect id and class nodes until we hit a whitespace, newline, start of attributes or single dot without trailing alphabetical char

    let mut id_nodes: Vec<IdNode> = vec![];
    let mut class_nodes: Vec<ClassNode> = vec![];
    let mut attribute_nodes: Option<Vec<HsmlNode>> = None;
    let mut text_node: Option<TextNode> = None;
    let mut child_nodes: Vec<HsmlNode> = vec![];

    loop {
        let first_char = input.fragment().get(..1);
        let first_two_chars = input.fragment().get(..2);

        if first_char == Some("#") {
            // Collect all id nodes — duplicates are detected post-parse by the validator.
            let (rest, node) = id::node::id_node(input)?;
            id_nodes.push(node);
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

            // Inline comments after text are intentionally not supported because
            // text content can contain sequences like "//" (e.g. URLs: https://example.com).

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

            let indentation_str = *indentation.fragment();

            if !indentation_str.is_empty() {
                // Mixed tabs and spaces are detected post-parse by the validator (W003).

                // persist the indentation level so we can restore it later
                let nested_tag_level = context.nested_tag_level;
                let indent_string = context.indent_string.clone();

                // check that we are at the correct indentation level, otherwise break out of the loop
                if !indentation_str.starts_with(&context.indent_string)
                    || indentation_str.len() <= context.indent_string.len()
                {
                    // dbg!("break out of loop");
                    break;
                }

                context.nested_tag_level += 1;
                context.indent_string = indentation_str.to_string();

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
                    match tag_node(remaining, context) {
                        Ok((rest, node)) => {
                            child_nodes.push(HsmlNode::Tag(node));
                            input = rest;
                        }
                        Err(err) => {
                            context.nested_tag_level = nested_tag_level;
                            context.indent_string = indent_string;
                            return Err(err);
                        }
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
            ids: id_nodes,
            classes: (!class_nodes.is_empty()).then_some(class_nodes),
            attributes: attribute_nodes,
            text: text_node,
            children: (!child_nodes.is_empty()).then_some(child_nodes),
        },
    ))
}
