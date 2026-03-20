use crate::parser::{
    HsmlProcessContext, Span,
    class::node::ClassNode,
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
        TagNode {
            tag: String::from("p"),
            id: None,
            classes: Some(vec![
                ClassNode {
                    name: String::from("text-lg"),
                },
                ClassNode {
                    name: String::from("font-medium"),
                },
            ]),
            attributes: None,
            text: Some(TextNode {
                text: String::from(
                    r#""Tailwind CSS is the only framework that I've seen scale
on large teams. It's easy to customize, adapts to any design,
and the build size is tiny.""#
                ),
            }),
            children: None,
        }
    );

    assert_eq!(*rest.fragment(), "\n    figcaption.font-medium");
}

#[test]
fn it_should_error_on_mixed_tabs_and_spaces_indentation() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 0,
        indent_string: String::new(),
    };

    // Child indented with mixed tabs and spaces
    let input = Span::new("div\n \tchild");

    let result = tag_node(input, context);

    assert!(result.is_err());
    // Error span should point to the mixed indentation, not the tag start
    if let Err(nom::Err::Failure(err)) = result {
        assert_eq!(*err.span.fragment(), " \t");
    } else {
        panic!("Expected Failure error");
    }
}

#[test]
fn it_should_propagate_duplicate_id_error_from_child() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 0,
        indent_string: String::new(),
    };

    let input = Span::new("div\n  span#a#b");

    let result = tag_node(input, context);

    assert!(result.is_err());
    if let Err(nom::Err::Failure(err)) = result {
        assert_eq!(
            err.message.as_deref(),
            Some("Duplicate attribute 'id' is not allowed")
        );
        assert_eq!(err.code(), Some("E001"));
        assert_eq!(*err.span.fragment(), "#b");
    } else {
        panic!("Expected Failure error with E001");
    }
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

    // Incomplete is propagated from process_tag which currently uses
    // nom::Err::Incomplete for non-alphabetic tag starts. This should
    // become a Failure with a descriptive message in a future refactor.
    assert!(matches!(result, Err(nom::Err::Incomplete(_))));
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
        TagNode {
            tag: String::from("div"),
            id: None,
            classes: None,
            attributes: None,
            text: None,
            children: None,
        }
    );

    assert_eq!(*rest.fragment(), "\n  span");
}
