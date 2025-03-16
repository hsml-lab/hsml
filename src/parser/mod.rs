use nom::{IResult, character::complete::line_ending};

use self::{
    attribute::node::AttributeNode, class::node::ClassNode, comment::node::CommentNode,
    id::node::IdNode, tag::node::TagNode, text::node::TextNode,
};

pub mod attribute;
pub mod class;
pub mod comment;
pub mod id;
pub mod parse;
pub mod tag;
pub mod text;

#[derive(Debug, PartialEq)]
pub struct RootNode {
    pub nodes: Vec<HsmlNode>,
}

#[derive(Debug, PartialEq)]
pub enum HsmlNode {
    Root(RootNode),
    Tag(TagNode),
    Comment(CommentNode),
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

pub fn process_newline(input: &str) -> IResult<&str, &str> {
    line_ending(input)
}
