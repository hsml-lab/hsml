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
    pub indent_level: usize,
    pub indent_string: Option<String>,
}

pub fn process_newline(input: &str) -> IResult<&str, &str> {
    line_ending(input)
}
