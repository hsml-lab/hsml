use crate::parser::{
    HsmlProcessContext, Span,
    text::node::{TextNode, text_block_node},
};

#[test]
fn it_should_return_text_block_node() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 3,
        indent_string: String::from("      "),
    };

    let (rest, text_block) = text_block_node(
        Span::new(
            r#".
        "Tailwind CSS is the only framework that I've seen scale
        on large teams. It's easy to customize, adapts to any design,
        and the build size is tiny."
    figcaption.font-medium"#,
        ),
        context,
    )
    .unwrap();

    assert_eq!(
        text_block,
        TextNode {
            text: String::from(
                r#""Tailwind CSS is the only framework that I've seen scale
on large teams. It's easy to customize, adapts to any design,
and the build size is tiny.""#
            ),
        }
    );

    assert_eq!(*rest.fragment(), "\n    figcaption.font-medium");
}

#[test]
fn it_should_stop_before_next_tag_node() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let (rest, text_block) = text_block_node(
        Span::new(
            r#".
    Sarah Dayan
  .text-[#af05c9].dark:text-slate-500.
    Staff Engineer, Algolia"#,
        ),
        context,
    )
    .unwrap();

    assert_eq!(
        text_block,
        TextNode {
            text: String::from(r#"Sarah Dayan"#),
        }
    );

    assert_eq!(
        *rest.fragment(),
        "\n  .text-[#af05c9].dark:text-slate-500.\n    Staff Engineer, Algolia"
    );
}

#[test]
fn it_should_return_text_block_node_with_multibyte_chars() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let (rest, text_block) = text_block_node(
        Span::new(".\n    héllo wörld 🌍\n    più línés café\nspan next"),
        context,
    )
    .unwrap();

    assert_eq!(
        text_block,
        TextNode {
            text: String::from("héllo wörld 🌍\npiù línés café"),
        }
    );

    assert_eq!(*rest.fragment(), "\nspan next");
}

#[test]
fn it_should_return_empty_text_block_when_first_line_not_indented_enough() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line "span.foo" doesn't have indent_string + extra space/tab,
    // so the text block should be empty.
    let (rest, text_block) =
        text_block_node(Span::new(".\n  span.foo\n  span.bar"), context).unwrap();

    assert_eq!(
        text_block,
        TextNode {
            text: String::from(""),
        }
    );

    assert_eq!(*rest.fragment(), "  span.foo\n  span.bar");
}

#[test]
fn it_should_strip_indent_from_first_nonempty_line() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // The indentation prefix of the first non-empty line ("    " = 4 spaces)
    // should be stripped from all lines.
    let (rest, text_block) = text_block_node(
        Span::new(".\n    line one\n    line two\nspan next"),
        context,
    )
    .unwrap();

    assert_eq!(
        text_block,
        TextNode {
            text: String::from("line one\nline two"),
        }
    );

    assert_eq!(*rest.fragment(), "\nspan next");
}

#[test]
fn it_should_return_text_block_node_with_cjk_and_blank_lines() {
    let context = &mut HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let (rest, text_block) = text_block_node(
        Span::new(".\n    こんにちは\n\n    世界テスト\nspan next"),
        context,
    )
    .unwrap();

    assert_eq!(
        text_block,
        TextNode {
            text: String::from("こんにちは\n\n世界テスト"),
        }
    );

    assert_eq!(*rest.fragment(), "\nspan next");
}
