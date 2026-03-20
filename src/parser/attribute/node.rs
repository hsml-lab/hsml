use nom::bytes::complete::{tag, take_till};

use crate::parser::{
    HsmlNode, HsmlProcessContext, HsmlResult, Span, comment::node::comment_dev_node,
};

use super::process::process_attribute;

#[derive(Debug, PartialEq, Eq)]
pub struct AttributeNode {
    pub key: String,
    pub value: Option<String>,
}

pub fn attribute_node<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, AttributeNode> {
    let (input, attribute) = process_attribute(input, context)?;

    let attribute_str = *attribute.fragment();
    let equal_sign_index = attribute_str.find('=').unwrap_or(attribute_str.len());
    let (key, value) = attribute_str.split_at(equal_sign_index);

    // Remove surrounding quotes and leading `=` from value
    let value = value
        .strip_prefix(r#"=""#)
        .and_then(|v| v.strip_suffix('"'))
        .map(|v| v.to_string());

    Ok((
        input,
        AttributeNode {
            key: key.to_string(),
            value,
        },
    ))
}

pub fn attribute_nodes<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, Vec<HsmlNode>> {
    let (mut input, _) = tag("(")(input)?;

    let mut nodes: Vec<HsmlNode> = vec![];

    // loop until `)`
    // take until attr starts (trim , and whitespace)
    // collect attr
    // if attr is empty, break
    loop {
        let (remaining, _) = take_till(|c: char| !c.is_whitespace() && c != ',')(input)?;

        if remaining.starts_with(')') {
            input = remaining;
            break;
        }

        // if remaining starts with `//`, it is a dev comment
        if remaining.starts_with("//") {
            let (remaining, comment) = comment_dev_node(remaining)?;
            nodes.push(HsmlNode::Comment(comment));

            input = remaining;
            continue;
        }

        let (remaining, attribute) = attribute_node(remaining, context)?;

        nodes.push(HsmlNode::Attribute(attribute));
        input = remaining;
    }

    let (input, _) = tag(")")(input)?;

    Ok((input, nodes))
}
