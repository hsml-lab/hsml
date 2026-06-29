use serde::Serialize;

use crate::common::{Location, Position};
use crate::parser::{
    HsmlNode, HsmlProcessContext, HsmlResult, Span, attribute,
    children::parse_children,
    class::node::{ClassNode, class_node},
    id::{self, node::IdNode},
    tag::process::process_tag,
    text::{self, node::TextNode},
};

#[derive(Debug, Serialize)]
pub struct TagNode {
    pub tag: String,
    /// Source location of the tag name.
    pub location: Location,
    /// All id selectors on this tag. Only the first is used in compilation;
    /// duplicates are reported as warnings by the validator.
    pub ids: Vec<IdNode>,
    pub classes: Option<Vec<ClassNode>>,
    pub attributes: Option<Vec<HsmlNode>>,
    pub text: Option<TextNode>,
    pub children: Option<Vec<HsmlNode>>,
}

impl TagNode {
    /// Create a TagNode without a meaningful source location.
    /// Useful in tests where location is not relevant.
    #[doc(hidden)]
    pub fn without_location(
        tag: impl Into<String>,
        ids: Vec<IdNode>,
        classes: Option<Vec<ClassNode>>,
        attributes: Option<Vec<HsmlNode>>,
        text: Option<TextNode>,
        children: Option<Vec<HsmlNode>>,
    ) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            tag: tag.into(),
            location: Location {
                start: zero,
                end: zero,
            },
            ids,
            classes,
            attributes,
            text,
            children,
        }
    }
}

// PartialEq excludes location so that tests comparing parsed ASTs
// don't need to specify exact location values for every tag.
impl PartialEq for TagNode {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
            && self.ids == other.ids
            && self.classes == other.classes
            && self.attributes == other.attributes
            && self.text == other.text
            && self.children == other.children
    }
}

pub fn tag_node<'a>(input: Span<'a>, context: &mut HsmlProcessContext) -> HsmlResult<'a, TagNode> {
    // tag node starts with a tag name or a dot/hash
    // if it starts with a dot/hash, the tag name is div

    let tag_start_span = input;

    let (mut input, tag_name) = if input.starts_with('.') || input.starts_with('#') {
        // Implicit div — location is the dot/hash position (zero-width)
        (input, "div")
    } else {
        let (rest, name) = process_tag(input)?;
        (rest, *name.fragment())
    };

    let tag_location = Location::from_spans(&tag_start_span, &input);

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
            // The tag header ended; gather any deeper-indented children.
            let (rest, children) = parse_children(input, context)?;
            child_nodes = children;
            input = rest;
            break;
        }

        break;
    }

    Ok((
        input,
        TagNode {
            tag: tag_name.to_string(),
            location: tag_location,
            ids: id_nodes,
            classes: (!class_nodes.is_empty()).then_some(class_nodes),
            attributes: attribute_nodes,
            text: text_node,
            children: (!child_nodes.is_empty()).then_some(child_nodes),
        },
    ))
}
