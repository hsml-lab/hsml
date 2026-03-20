use nom::{Input, character::complete::line_ending};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

use self::{
    attribute::node::AttributeNode, class::node::ClassNode, comment::node::CommentNode,
    doctype::node::DoctypeNode, id::node::IdNode, tag::node::TagNode, text::node::TextNode,
};

pub mod attribute;
pub mod class;
pub mod comment;
pub mod doctype;
pub mod error;
pub mod id;
pub mod parse;
pub mod tag;
pub mod text;

/// Convenience alias for parser results using the custom HSML error type.
pub type HsmlResult<'a, T> = nom::IResult<Span<'a>, T, error::HsmlError<'a>>;

#[derive(Debug, PartialEq)]
pub struct RootNode {
    pub nodes: Vec<HsmlNode>,
}

#[derive(Debug, PartialEq)]
pub enum HsmlNode {
    Root(RootNode),
    Tag(TagNode),
    Comment(CommentNode),
    Doctype(DoctypeNode),
    Id(IdNode),
    Class(ClassNode),
    Attribute(AttributeNode),
    Text(TextNode),
}

#[derive(Debug, Default)]
pub struct HsmlProcessContext {
    // TODO @Shinigami92 2025-03-16: Currently nested_tag_level is not used, but should be later to allow mixed spaces and tabs in indentation
    /// The tracked nested tag level
    pub nested_tag_level: usize,

    /// The tracked indentation string
    ///
    /// Can be a combination of spaces and tabs
    pub indent_string: String,
}

pub fn process_newline(input: Span<'_>) -> HsmlResult<'_, Span<'_>> {
    line_ending(input)
}

/// Helper to advance a span by n bytes, returning the remaining span.
pub fn advance<'a>(span: Span<'a>, n: usize) -> Span<'a> {
    span.take_split(n).0
}

/// Helper to take the first n bytes from a span.
pub fn take_prefix<'a>(span: Span<'a>, n: usize) -> Span<'a> {
    span.take_split(n).1
}
