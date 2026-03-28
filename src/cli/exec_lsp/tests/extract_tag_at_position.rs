use tower_lsp::lsp_types::Position;

use crate::cli::exec_lsp::extract_tag_at_position;

#[test]
fn it_should_extract_tag_at_start_of_line() {
    let source = "h1 Hello\n";
    assert_eq!(
        extract_tag_at_position(source, Position::new(0, 0)),
        Some("h1".to_string())
    );
}

#[test]
fn it_should_extract_tag_within_name() {
    let source = "div Hello\n";
    assert_eq!(
        extract_tag_at_position(source, Position::new(0, 1)),
        Some("div".to_string())
    );
}

#[test]
fn it_should_extract_tag_with_indentation() {
    let source = "body\n  h1 Hello\n";
    assert_eq!(
        extract_tag_at_position(source, Position::new(1, 2)),
        Some("h1".to_string())
    );
}

#[test]
fn it_should_extract_tag_before_class() {
    let source = "h1.text-red Hello\n";
    assert_eq!(
        extract_tag_at_position(source, Position::new(0, 0)),
        Some("h1".to_string())
    );
}

#[test]
fn it_should_extract_tag_before_id() {
    let source = "div#app\n";
    assert_eq!(
        extract_tag_at_position(source, Position::new(0, 1)),
        Some("div".to_string())
    );
}

#[test]
fn it_should_return_none_when_cursor_on_class() {
    let source = "h1.text-red Hello\n";
    assert_eq!(extract_tag_at_position(source, Position::new(0, 3)), None);
}

#[test]
fn it_should_return_none_when_cursor_on_text() {
    let source = "h1 Hello\n";
    assert_eq!(extract_tag_at_position(source, Position::new(0, 5)), None);
}

#[test]
fn it_should_extract_hyphenated_tag() {
    let source = "my-element Hello\n";
    assert_eq!(
        extract_tag_at_position(source, Position::new(0, 4)),
        Some("my-element".to_string())
    );
}

#[test]
fn it_should_normalize_tag_to_lowercase() {
    let source = "DIV Hello\n";
    assert_eq!(
        extract_tag_at_position(source, Position::new(0, 0)),
        Some("div".to_string())
    );
}

#[test]
fn it_should_return_none_for_class_only_line() {
    // Lines starting with . or # have no tag name
    let source = ".container\n";
    assert_eq!(extract_tag_at_position(source, Position::new(0, 0)), None);
}

#[test]
fn it_should_return_none_for_comment_line() {
    let source = "// comment\n";
    assert_eq!(extract_tag_at_position(source, Position::new(0, 0)), None);
}
