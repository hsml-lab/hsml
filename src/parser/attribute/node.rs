use nom::bytes::complete::{tag, take_till};

use crate::common::{Location, Position};
use crate::parser::{
    HsmlNode, HsmlProcessContext, HsmlResult, Span, comment::node::comment_dev_node,
};

use super::process::process_attribute;

#[derive(Debug, Eq, serde::Serialize)]
pub struct AttributeNode {
    pub key: String,
    pub value: Option<String>,
    /// Source location where this attribute appears.
    pub location: Location,
}

// PartialEq only compares key and value so that tests comparing parsed ASTs
// don't need to specify exact location values for every attribute.
impl PartialEq for AttributeNode {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl AttributeNode {
    /// Create an AttributeNode with key and value (no source location).
    /// Useful in tests where location is not relevant.
    #[doc(hidden)]
    pub fn new_without_location(key: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        Self {
            key: key.into(),
            value: value.map(|v| v.into()),
            location: Location {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
        }
    }
}

pub fn attribute_node<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, AttributeNode> {
    let start = Position {
        line: input.location_line(),
        column: input.get_column() as u32,
    };

    let (input, attribute) = process_attribute(input, context)?;

    let location = Location {
        start,
        end: Position {
            line: input.location_line(),
            column: input.get_column() as u32,
        },
    };

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
            location,
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
