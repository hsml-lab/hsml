use nom::bytes::complete::take_till;

use super::{
    HsmlNode, HsmlProcessContext, HsmlResult, RootNode, Span,
    comment::node::{comment_dev_node, comment_native_node},
    doctype::node::doctype_node,
    error::HsmlError,
    tag::node::tag_node,
};

pub fn parse(input: Span<'_>) -> HsmlResult<'_, RootNode> {
    let mut nodes: Vec<HsmlNode> = vec![];

    let mut context = HsmlProcessContext::default();

    let mut input = input;

    loop {
        // eat leading and trailing newlines and whitespace if there are any
        if let Ok((rest, taken)) =
            take_till::<_, Span, HsmlError>(|c: char| !c.is_whitespace())(input)
        {
            // take the leading spaces and tabs after the last newline as indentation
            context.indent_string = taken
                .fragment()
                .chars()
                .rev()
                .take_while(|c| c.is_whitespace() && *c != '\n')
                .collect::<String>()
                .chars()
                .rev()
                .collect();

            input = rest;

            if input.fragment().is_empty() {
                break;
            }
        }

        if let Ok((rest, node)) = doctype_node(input) {
            nodes.push(HsmlNode::Doctype(node));
            input = rest;
            continue;
        }

        if let Ok((rest, node)) = comment_native_node(input) {
            nodes.push(HsmlNode::Comment(node));
            input = rest;
            continue;
        }

        if let Ok((rest, node)) = comment_dev_node(input) {
            nodes.push(HsmlNode::Comment(node));
            input = rest;
            continue;
        }

        match tag_node(input, &mut context) {
            Ok((rest, node)) => {
                nodes.push(HsmlNode::Tag(node));
                input = rest;
                continue;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok((input, RootNode { nodes }))
}
