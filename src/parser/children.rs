use nom::bytes::complete::{take_till, take_till1};

use crate::parser::{
    HsmlNode, HsmlProcessContext, HsmlResult, Span, angular,
    comment::node::{comment_dev_node, comment_native_node},
    tag,
};

/// Parse all deeper-indented children that follow a parent's header line.
///
/// `input` must be positioned at the newline that ends the parent header line
/// (e.g. just after a tag's `h1.foo(...)` line or an `@if (cond)` line). Children
/// are gathered until indentation dedents to or below the parent's level (tracked
/// in `context.indent_string`). The returned span is positioned at the newline
/// preceding the dedented content, ready for the caller to continue from.
///
/// This is the shared indentation engine used by both `tag_node` and the Angular
/// block-bearing constructs, so they all nest identically.
pub fn parse_children<'a>(
    mut input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, Vec<HsmlNode>> {
    let mut child_nodes: Vec<HsmlNode> = vec![];

    loop {
        if !(input.starts_with('\n') || input.starts_with("\r\n")) {
            break;
        }

        let (rest, _) = advance_to_next_line(input)?;

        // If we've reached EOF (possibly with trailing whitespace), stop
        if rest.fragment().trim().is_empty() {
            break;
        }

        let (remaining, indentation) = take_till(|c: char| !c.is_whitespace())(rest)?;
        let indentation_str = *indentation.fragment();

        // No indentation at all — the next line is a sibling/ancestor, not a child
        if indentation_str.is_empty() {
            break;
        }

        // persist the indentation level so we can restore it after the child
        let nested_tag_level = context.nested_tag_level;
        let indent_string = context.indent_string.clone();

        // Mixed tabs and spaces are detected post-parse by the validator (W003).
        // Dedent (or same level) — children belong to an ancestor; stop here.
        if !indentation_str.starts_with(&context.indent_string)
            || indentation_str.len() <= context.indent_string.len()
        {
            break;
        }

        context.nested_tag_level += 1;
        context.indent_string = indentation_str.to_string();

        match parse_one_child(remaining, context) {
            Ok((after, node)) => {
                child_nodes.push(node);
                input = after;
            }
            Err(err) => {
                context.nested_tag_level = nested_tag_level;
                context.indent_string = indent_string;
                return Err(err);
            }
        }

        context.nested_tag_level = nested_tag_level;
        context.indent_string = indent_string;
    }

    Ok((input, child_nodes))
}

/// Consume the newline at `input` plus any subsequent blank (whitespace-only)
/// lines, returning the span positioned at the start of the next non-blank line
/// (with its leading indentation intact), or at end-of-input.
///
/// `input` must start with a newline. Shared by `parse_children` and the Angular
/// continuation detection so blank-line handling stays consistent.
pub fn advance_to_next_line<'a>(input: Span<'a>) -> HsmlResult<'a, ()> {
    let (mut rest, _) = take_till1(|c| c != '\r' && c != '\n')(input)?;

    loop {
        let (after_ws, ws) =
            take_till(|c: char| c == '\n' || c == '\r' || !c.is_whitespace())(rest)?;

        // EOF after whitespace — nothing more to parse
        if !ws.fragment().is_empty() && after_ws.fragment().is_empty() {
            break;
        }

        // whitespace-only line followed by a newline — skip the blank line
        if !ws.fragment().is_empty() && (after_ws.starts_with('\n') || after_ws.starts_with("\r\n"))
        {
            let (after_nl, _) = take_till1(|c| c != '\r' && c != '\n')(after_ws)?;
            rest = after_nl;
            continue;
        }
        break;
    }

    Ok((rest, ()))
}

/// Parse a single child node at the current position: a comment, an Angular
/// `@`-block, or a tag.
fn parse_one_child<'a>(
    remaining: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, HsmlNode> {
    if let Ok((rest, node)) = comment_native_node(remaining) {
        return Ok((rest, HsmlNode::Comment(node)));
    }

    if let Ok((rest, node)) = comment_dev_node(remaining) {
        return Ok((rest, HsmlNode::Comment(node)));
    }

    if remaining.starts_with('@') {
        let (rest, node) = angular::node::angular_node(remaining, context)?;
        return Ok((rest, HsmlNode::Angular(node)));
    }

    let (rest, node) = tag::node::tag_node(remaining, context)?;
    Ok((rest, HsmlNode::Tag(node)))
}
