use crate::common::Position;
use crate::parser::Span;
use crate::parser::comment::node::{CommentNode, comment_dev_node, comment_native_node};

#[test]
fn it_should_return_comment_dev_node() {
    let (rest, comment) = comment_dev_node(Span::new("// This is a dev comment\n")).unwrap();

    assert_eq!(
        comment,
        CommentNode::new_without_location(" This is a dev comment", true)
    );

    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_track_dev_comment_location() {
    let (_, comment) = comment_dev_node(Span::new("// Hello\n")).unwrap();

    assert_eq!(comment.location.start, Position { line: 1, column: 1 });
    assert_eq!(comment.location.end, Position { line: 1, column: 9 });
}

#[test]
fn it_should_track_native_comment_location() {
    let (_, comment) = comment_native_node(Span::new("//! Hello\n")).unwrap();

    assert_eq!(comment.location.start, Position { line: 1, column: 1 });
    assert_eq!(
        comment.location.end,
        Position {
            line: 1,
            column: 10
        }
    );
}

#[test]
fn it_should_return_comment_native_node() {
    let (rest, comment) = comment_native_node(Span::new("//! This is a native comment\n")).unwrap();

    assert_eq!(
        comment,
        CommentNode::new_without_location(" This is a native comment", false)
    );

    assert_eq!(*rest.fragment(), "\n");
}
