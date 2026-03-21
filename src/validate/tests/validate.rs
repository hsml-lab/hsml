use crate::diagnostic::{Location, Severity};
use crate::parser::Span;
use crate::parser::error::ErrorCode;
use crate::parser::parse::parse;
use crate::validate::validate;

#[test]
fn it_should_warn_on_duplicate_class() {
    let source = "h1.text-red.text-red Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Severity::Warning);
    assert_eq!(diagnostics[0].message, "Duplicate class 'text-red'");
    assert_eq!(
        diagnostics[0].code,
        Some(ErrorCode::DuplicateClass.code().to_string())
    );
    assert_eq!(
        diagnostics[0].location,
        Some(Location {
            line: 1,
            column: 12
        })
    );
}

#[test]
fn it_should_not_warn_on_unique_classes() {
    let source = "h1.text-red.text-blue Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert!(diagnostics.is_empty());
}

#[test]
fn it_should_warn_on_duplicate_class_in_child() {
    let source = "div\n  h1.foo.foo Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Duplicate class 'foo'");
    assert_eq!(
        diagnostics[0].location,
        Some(Location { line: 2, column: 9 })
    );
}

#[test]
fn it_should_warn_on_multiple_duplicates() {
    let source = "h1.a.b.a.b Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message, "Duplicate class 'a'");
    assert_eq!(
        diagnostics[0].location,
        Some(Location { line: 1, column: 7 })
    );
    assert_eq!(diagnostics[1].message, "Duplicate class 'b'");
    assert_eq!(
        diagnostics[1].location,
        Some(Location { line: 1, column: 9 })
    );
}

#[test]
fn it_should_not_warn_on_no_classes() {
    let source = "h1 Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert!(diagnostics.is_empty());
}

#[test]
fn it_should_warn_on_correct_location_with_same_class_in_different_tags() {
    let source = "h1.foo Hello\nh2.foo.foo World\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    // Only the second tag has a duplicate — h1.foo is fine
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Duplicate class 'foo'");
    assert_eq!(
        diagnostics[0].location,
        Some(Location { line: 2, column: 7 })
    );
}

#[test]
fn it_should_warn_on_mixed_indentation() {
    let source = "div\n \tchild\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Severity::Warning);
    assert_eq!(
        diagnostics[0].message,
        "Mixed tabs and spaces in indentation"
    );
    assert_eq!(
        diagnostics[0].code,
        Some(ErrorCode::MixedIndentation.code().to_string())
    );
    assert_eq!(
        diagnostics[0].location,
        Some(Location { line: 2, column: 1 })
    );
}

#[test]
fn it_should_not_warn_on_consistent_space_indentation() {
    let source = "div\n  span Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert!(diagnostics.is_empty());
}

#[test]
fn it_should_not_warn_on_consistent_tab_indentation() {
    let source = "div\n\tspan Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert!(diagnostics.is_empty());
}

#[test]
fn it_should_warn_on_tab_then_space_indentation() {
    let source = "div\n\t span Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        Some(ErrorCode::MixedIndentation.code().to_string())
    );
}

#[test]
fn it_should_warn_on_multiple_mixed_lines() {
    let source = "div\n \tchild1\n \tchild2\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    // Both lines have mixed indentation
    let mixed_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::MixedIndentation.code().to_string()))
        .collect();
    assert_eq!(mixed_warnings.len(), 2);
}

#[test]
fn it_should_not_warn_on_whitespace_only_lines() {
    let source = "p.\n   text\n \t \n   more text\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    // The whitespace-only line " \t " should not trigger W003
    assert!(diagnostics.is_empty());
}

#[test]
fn it_should_warn_on_duplicate_id() {
    let source = "div#a#b Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    let id_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::DuplicateId.code().to_string()))
        .collect();
    assert_eq!(id_warnings.len(), 1);
    assert_eq!(id_warnings[0].severity, Severity::Warning);
    assert_eq!(id_warnings[0].message, "Duplicate id 'b' is not allowed");
    assert_eq!(
        id_warnings[0].location,
        Some(Location { line: 1, column: 6 })
    );
}

#[test]
fn it_should_not_warn_on_single_id() {
    let source = "div#my-id Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert!(diagnostics.is_empty());
}
