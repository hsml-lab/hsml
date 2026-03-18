use crate::parser::{
    HsmlProcessContext,
    text::process::{process_text, process_text_block},
};

#[test]
fn it_should_process_text_block() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let input = r#".
   this is just some text
    it can be multiline

    	and also contain blank lines
span other text
"#;

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(
        text_block,
        r#"   this is just some text
    it can be multiline

    	and also contain blank lines"#
    );
    assert_eq!(
        rest,
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

    let input = ".\n   héllo wörld 🌍\n    più tëst línés\nspan next\n";

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(text_block, "   héllo wörld 🌍\n    più tëst línés");
    assert_eq!(rest, "\nspan next\n");
}

#[test]
fn it_should_process_text_block_with_cjk_chars() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let input = ".\n   こんにちは世界\n    テスト文字列\nspan next\n";

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(text_block, "   こんにちは世界\n    テスト文字列");
    assert_eq!(rest, "\nspan next\n");
}

#[test]
fn it_should_process_text_block_with_emoji_and_blank_lines() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    let input = ".\n   first 🎉 line\n\n    second 🚀 line\nspan next\n";

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(text_block, "   first 🎉 line\n\n    second 🚀 line");
    assert_eq!(rest, "\nspan next\n");
}

#[test]
fn it_should_return_empty_when_first_line_lacks_indent() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line "hello" does not start with the indent string "  "
    let input = ".\nhello world\nspan next\n";

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(text_block, "");
    assert_eq!(rest, "hello world\nspan next\n");
}

#[test]
fn it_should_return_empty_when_first_line_has_indent_but_no_extra_space() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line starts with indent "  " but the next char is not a space or tab
    // This looks like a sibling tag, not text content
    let input = ".\n  span.foo\nspan next\n";

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(text_block, "");
    assert_eq!(rest, "  span.foo\nspan next\n");
}

#[test]
fn it_should_process_when_first_line_has_indent_plus_space() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line starts with indent "  " followed by a space (3 spaces total)
    let input = ".\n   valid text here\nspan next\n";

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(text_block, "   valid text here");
    assert_eq!(rest, "\nspan next\n");
}

#[test]
fn it_should_process_when_first_line_has_indent_plus_tab() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line starts with indent "  " followed by a tab
    let input = ".\n  \tvalid text here\nspan next\n";

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(text_block, "  \tvalid text here");
    assert_eq!(rest, "\nspan next\n");
}

#[test]
fn it_should_process_when_first_line_is_blank() {
    let mut context = HsmlProcessContext {
        nested_tag_level: 1,
        indent_string: String::from("  "),
    };

    // First line is empty (blank), second line has valid indent
    let input = ".\n\n   text after blank\nspan next\n";

    let (rest, text_block) = process_text_block(input, &mut context).unwrap();

    assert_eq!(text_block, "\n   text after blank");
    assert_eq!(rest, "\nspan next\n");
}

#[test]
fn test_process_text() {
    let input = " hello world\n";

    let (rest, text) = process_text(input).unwrap();

    assert_eq!(text, "hello world");
    assert_eq!(rest, "\n");
}
