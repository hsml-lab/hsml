use nom::Input;
use nom_locate::LocatedSpan;

/// A located span over the input source, tracking line and column information.
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

use serde::Serialize;

/// The root of a parsed HSML document, containing all top-level nodes.
#[derive(Debug, PartialEq, Serialize)]
pub struct RootNode {
    pub nodes: Vec<HsmlNode>,
}

/// A single node in the HSML abstract syntax tree.
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
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

/// Mutable state carried through the parsing process.
#[derive(Debug, Default)]
pub struct HsmlProcessContext {
    // nested_tag_level is tracked but not yet used for logic.
    // It will be used by `hsml format` to normalize indentation.
    /// The tracked nested tag level
    pub nested_tag_level: usize,

    /// The tracked indentation string
    ///
    /// Can be a combination of spaces and tabs
    pub indent_string: String,
}

/// Helper to advance a span by n bytes, returning the remaining span.
pub fn advance<'a>(span: Span<'a>, n: usize) -> Span<'a> {
    span.take_split(n).0
}

/// Helper to take the first n bytes from a span.
pub fn take_prefix<'a>(span: Span<'a>, n: usize) -> Span<'a> {
    span.take_split(n).1
}

/// Find the byte length of an escape-aware delimited section (e.g. `[...]`, `(...)`).
/// Returns `Some(len)` including the opening and closing delimiters, or `None` if unclosed.
/// The string must start with `open`. Quoted substrings inside are treated as atomic
/// (a closing delimiter inside quotes is not matched).
pub fn delimited_section_len(s: &str, open: char, close: char) -> Option<usize> {
    if !s.starts_with(open) {
        return None;
    }

    let mut chars = s.char_indices();
    chars.next(); // skip the opening delimiter

    let mut is_escaped = false;

    while let Some((index, c)) = chars.next() {
        if c == '\\' {
            is_escaped = !is_escaped;
            continue;
        }

        // Skip quoted substrings atomically (but not when the delimiter itself is a quote)
        if !is_escaped && (c == '"' || c == '\'') && c != close {
            let quote = c;
            let mut quote_escaped = false;
            for (_, qc) in chars.by_ref() {
                if qc == '\\' {
                    quote_escaped = !quote_escaped;
                    continue;
                }
                if qc == quote && !quote_escaped {
                    break;
                }
                quote_escaped = false;
            }
            is_escaped = false;
            continue;
        }

        if c == close && !is_escaped {
            return Some(index + c.len_utf8());
        }

        is_escaped = false;
    }

    None
}

/// Find the byte length of an escape-aware quoted string (e.g. `"..."` or `'...'`).
/// Returns `Some(len)` including the surrounding quotes, or `None` if unclosed.
/// The string must start with `"` or `'`.
pub fn quoted_string_len(s: &str) -> Option<usize> {
    let quote = s.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }

    delimited_section_len(s, quote, quote)
}
