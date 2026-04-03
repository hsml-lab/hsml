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
fn it_should_return_comment_native_node() {
    let (rest, comment) = comment_native_node(Span::new("//! This is a native comment\n")).unwrap();

    assert_eq!(
        comment,
        CommentNode::new_without_location(" This is a native comment", false)
    );

    assert_eq!(*rest.fragment(), "\n");
}
