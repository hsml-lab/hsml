use crate::parser::{
    HsmlProcessContext, Span,
    text::process::{process_text, process_text_block},
};

#[test]
fn it_should_process_text_block() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let input = Span::new(
        r#".
   this is just some text
    it can be multiline

    	and also contain blank lines
span other text
"#,
    );

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(
        *text_block.fragment(),
        r#"   this is just some text
    it can be multiline

    	and also contain blank lines"#
    );
    assert_eq!(
        *rest.fragment(),
        r#"
span other text
"#
    );
}

#[test]
fn it_should_process_text_block_with_multibyte_chars() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let input = Span::new(".\n   héllo wörld 🌍\n    più tëst línés\nspan next\n");

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(
        *text_block.fragment(),
        "   héllo wörld 🌍\n    più tëst línés"
    );
    assert_eq!(*rest.fragment(), "\nspan next\n");
}

#[test]
fn it_should_process_text_block_with_cjk_chars() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let input = Span::new(".\n   こんにちは世界\n    テスト文字列\nspan next\n");

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(
        *text_block.fragment(),
        "   こんにちは世界\n    テスト文字列"
    );
    assert_eq!(*rest.fragment(), "\nspan next\n");
}

#[test]
fn it_should_process_text_block_with_emoji_and_blank_lines() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let input = Span::new(".\n   first 🎉 line\n\n    second 🚀 line\nspan next\n");

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(
        *text_block.fragment(),
        "   first 🎉 line\n\n    second 🚀 line"
    );
    assert_eq!(*rest.fragment(), "\nspan next\n");
}

#[test]
fn it_should_return_empty_when_first_line_lacks_indent() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line "hello" does not start with the indent string "  "
    let input = Span::new(".\nhello world\nspan next\n");

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(*text_block.fragment(), "");
    assert_eq!(*rest.fragment(), "hello world\nspan next\n");
}

#[test]
fn it_should_return_empty_when_first_line_has_indent_but_no_extra_space() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line starts with indent "  " but the next char is not a space or tab
    // This looks like a sibling tag, not text content
    let input = Span::new(".\n  span.foo\nspan next\n");

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(*text_block.fragment(), "");
    assert_eq!(*rest.fragment(), "  span.foo\nspan next\n");
}

#[test]
fn it_should_process_when_first_line_has_indent_plus_space() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line starts with indent "  " followed by a space (3 spaces total)
    let input = Span::new(".\n   valid text here\nspan next\n");

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(*text_block.fragment(), "   valid text here");
    assert_eq!(*rest.fragment(), "\nspan next\n");
}

#[test]
fn it_should_process_when_first_line_has_indent_plus_tab() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line starts with indent "  " followed by a tab
    let input = Span::new(".\n  \tvalid text here\nspan next\n");

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(*text_block.fragment(), "  \tvalid text here");
    assert_eq!(*rest.fragment(), "\nspan next\n");
}

#[test]
fn it_should_process_when_first_line_is_blank() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line is empty (blank), second line has valid indent
    let input = Span::new(".\n\n   text after blank\nspan next\n");

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(*text_block.fragment(), "\n   text after blank");
    assert_eq!(*rest.fragment(), "\nspan next\n");
}

#[test]
fn test_process_text() {
    let input = Span::new(" hello world\n");

    let (rest, text) = process_text(input).unwrap();

    assert_eq!(*text.fragment(), "hello world");
    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_process_text_containing_double_slashes() {
    let input = Span::new(" Visit https://example.com for more info\n");

    let (rest, text) = process_text(input).unwrap();

    assert_eq!(*text.fragment(), "Visit https://example.com for more info");
    assert_eq!(*rest.fragment(), "\n");
}
