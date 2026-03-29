use crate::parser::{
    HsmlProcessContext, Span,
    class::node::ClassNode,
    error::ErrorCode,
    id::node::IdNode,
    tag::node::{TagNode, tag_node},
    text::node::TextNode,
};

#[test]
fn it_should_return_tag_node_with_piped_text() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 3,
        indent_string: String::from("      "),
    };

    let (rest, tag) = tag_node(
        Span::new(
            r#"p.text-lg.font-medium.
        "Tailwind CSS is the only framework that I've seen scale
        on large teams. It's easy to customize, adapts to any design,
        and the build size is tiny."
    figcaption.font-medium"#,
        ),
        context,
    )
    .unwrap();

    assert_eq!(
        tag,
        TagNode::without_location(
            "p",
            vec![],
            Some(vec![
                ClassNode::new_without_location("text-lg"),
                ClassNode::new_without_location("font-medium"),
            ]),
            None,
            Some(TextNode {
                text: String::from(
                    r#""Tailwind CSS is the only framework that I've seen scale
on large teams. It's easy to customize, adapts to any design,
and the build size is tiny.""#
                ),
            }),
            None,
        )
    );

    assert_eq!(*rest.fragment(), "\n    figcaption.font-medium");
}

#[test]
fn it_should_parse_mixed_tabs_and_spaces_indentation() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 0,
        indent_string: String::new(),
    };

    // Mixed indentation is accepted by the parser (validator warns about it)
    let input = Span::new("div\n \tchild\n");

    let (rest, tag) = tag_node(input, context).unwrap();

    assert_eq!(tag.tag, "div");
    assert!(tag.children.is_some());
    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_parse_child_with_multiple_ids() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 0,
        indent_string: String::new(),
    };

    let input = Span::new("div\n  span#a#b");

    let (rest, tag) = tag_node(input, context).unwrap();

    assert_eq!(tag.tag, "div");
    let children = tag.children.unwrap();
    assert_eq!(children.len(), 1);
    if let crate::parser::HsmlNode::Tag(child) = &children[0] {
        assert_eq!(child.tag, "span");
        assert_eq!(child.ids.len(), 2);
        assert_eq!(child.ids[0], IdNode::new_without_location("a"));
        assert_eq!(child.ids[1], IdNode::new_without_location("b"));
    } else {
        panic!("Expected Tag node");
    }

    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_error_on_invalid_child() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 0,
        indent_string: String::new(),
    };

    // Child starts with a number, which is not a valid tag
    let input = Span::new("div\n  123invalid");

    let result = tag_node(input, context);

    // Failure is propagated from process_tag (E004: InvalidTagName).
    if let Err(nom::Err::Failure(err)) = result {
        assert_eq!(err.code(), Some(ErrorCode::InvalidTagName.code()));
        assert_eq!(
            err.message.as_deref(),
            Some("Tag name must start with an ASCII letter")
        );
    } else {
        panic!("Expected Failure error with E004");
    }
}

#[test]
fn it_should_break_when_indentation_does_not_start_with_parent_indent() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("    "),
    };

    // Parent uses 4-space indent, but child uses 2-space indent
    let input = Span::new("div\n  span");

    let (rest, tag) = tag_node(input, context).unwrap();

    assert_eq!(
        tag,
        TagNode::without_location("div", vec![], None, None, None, None)
    );

    assert_eq!(*rest.fragment(), "\n  span");
}
