use crate::parser::{
    HsmlProcessContext,
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

    let (input, tag) = tag_node(
        r#"p.text-lg.font-medium.
        "Tailwind CSS is the only framework that I've seen scale
        on large teams. It's easy to customize, adapts to any design,
        and the build size is tiny."
    figcaption.font-medium"#,
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

    assert_eq!(input, "\n    figcaption.font-medium");
}

#[test]
fn it_should_error_on_mixed_tabs_and_spaces_indentation() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 0,
        indent_string: String::new(),
    };

    // Child indented with mixed tabs and spaces
    let input = "div\n \tchild";

    let result = tag_node(input, context);

    assert!(result.is_err());
}

#[test]
fn it_should_break_when_indentation_does_not_start_with_parent_indent() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("    "),
    };

    // Parent uses 4-space indent, but child uses 2-space indent
    let input = "div\n  span";

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

    assert_eq!(rest, "\n  span");
}
