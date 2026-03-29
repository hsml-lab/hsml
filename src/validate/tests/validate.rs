use crate::diagnostic::{Position, Severity};
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
        diagnostics[0].location.as_ref().map(|l| &l.start),
        Some(&Position {
            line: 1,
            column: 12
        })
    );
    assert_eq!(
        diagnostics[0].location.as_ref().map(|l| &l.end),
        Some(&Position {
            line: 1,
            column: 21
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
        diagnostics[0].location.as_ref().map(|l| &l.start),
        Some(&Position { line: 2, column: 9 })
    );
    assert_eq!(
        diagnostics[0].location.as_ref().map(|l| &l.end),
        Some(&Position {
            line: 2,
            column: 13
        })
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
        diagnostics[0].location.as_ref().map(|l| &l.start),
        Some(&Position { line: 1, column: 7 })
    );
    assert_eq!(
        diagnostics[0].location.as_ref().map(|l| &l.end),
        Some(&Position { line: 1, column: 9 })
    );
    assert_eq!(diagnostics[1].message, "Duplicate class 'b'");
    assert_eq!(
        diagnostics[1].location.as_ref().map(|l| &l.start),
        Some(&Position { line: 1, column: 9 })
    );
    assert_eq!(
        diagnostics[1].location.as_ref().map(|l| &l.end),
        Some(&Position {
            line: 1,
            column: 11
        })
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
        diagnostics[0].location.as_ref().map(|l| &l.start),
        Some(&Position { line: 2, column: 7 })
    );
    assert_eq!(
        diagnostics[0].location.as_ref().map(|l| &l.end),
        Some(&Position {
            line: 2,
            column: 11
        })
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
        diagnostics[0].location.as_ref().map(|l| &l.start),
        Some(&Position { line: 2, column: 1 })
    );
    assert_eq!(
        diagnostics[0].location.as_ref().map(|l| &l.end),
        Some(&Position { line: 2, column: 3 })
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
        id_warnings[0].location.as_ref().map(|l| &l.start),
        Some(&Position { line: 1, column: 6 })
    );
    assert_eq!(
        id_warnings[0].location.as_ref().map(|l| &l.end),
        Some(&Position { line: 1, column: 8 })
    );
}

#[test]
fn it_should_not_warn_on_single_id() {
    let source = "div#my-id Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert!(diagnostics.is_empty());
}

#[test]
fn it_should_warn_on_duplicate_attribute() {
    let source = "div(src=\"a\" src=\"b\")\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    let attr_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::DuplicateAttribute.code().to_string()))
        .collect();
    assert_eq!(attr_warnings.len(), 1);
    assert_eq!(attr_warnings[0].severity, Severity::Warning);
    assert_eq!(attr_warnings[0].message, "Duplicate attribute 'src'");
    assert_eq!(
        attr_warnings[0].location.as_ref().map(|l| &l.start),
        Some(&Position {
            line: 1,
            column: 13
        })
    );
    assert_eq!(
        attr_warnings[0].location.as_ref().map(|l| &l.end),
        Some(&Position {
            line: 1,
            column: 20
        })
    );
}

#[test]
fn it_should_not_warn_on_unique_attributes() {
    let source = "img(src=\"img.jpg\" alt=\"Image\")\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert!(diagnostics.is_empty());
}

#[test]
fn it_should_not_warn_on_duplicate_class_attribute() {
    let source = "div(class=\"a\" class=\"b\")\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    // class is mergeable, should not warn
    let attr_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::DuplicateAttribute.code().to_string()))
        .collect();
    assert_eq!(attr_warnings.len(), 0);
}

#[test]
fn it_should_not_warn_on_mergeable_vue_class_and_style() {
    let source = "div(:class=\"a\" :class=\"b\" :style=\"x\" :style=\"y\")\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    let attr_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::DuplicateAttribute.code().to_string()))
        .collect();
    assert_eq!(attr_warnings.len(), 0);
}

#[test]
fn it_should_warn_on_duplicate_vue_event_bindings() {
    let source = "div(@click=\"x\" @click=\"y\")\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    let attr_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::DuplicateAttribute.code().to_string()))
        .collect();
    assert_eq!(attr_warnings.len(), 1);
    assert_eq!(attr_warnings[0].message, "Duplicate attribute '@click'");
}

#[test]
fn it_should_warn_on_duplicate_data_attributes() {
    let source = "div(data-x=\"a\" data-x=\"b\")\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    let attr_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::DuplicateAttribute.code().to_string()))
        .collect();
    assert_eq!(attr_warnings.len(), 1);
    assert_eq!(attr_warnings[0].message, "Duplicate attribute 'data-x'");
}

// Void element content warnings

#[test]
fn it_should_warn_on_void_element_with_text() {
    let source = "br Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    let void_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::VoidElementContent.code().to_string()))
        .collect();
    assert_eq!(void_warnings.len(), 1);
    assert_eq!(
        void_warnings[0].message,
        "Void element cannot have content '<br>'"
    );
}

#[test]
fn it_should_warn_on_void_element_with_children() {
    let source = "hr\n  div Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    let void_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::VoidElementContent.code().to_string()))
        .collect();
    assert_eq!(void_warnings.len(), 1);
    assert_eq!(
        void_warnings[0].message,
        "Void element cannot have content '<hr>'"
    );
}

#[test]
fn it_should_not_warn_on_void_element_without_content() {
    let source = "br\nimg(alt=\"photo\")\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    let void_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::VoidElementContent.code().to_string()))
        .collect();
    assert_eq!(void_warnings.len(), 0);
}

#[test]
fn it_should_not_warn_on_non_void_element_without_content() {
    let source = "div\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    let void_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code == Some(ErrorCode::VoidElementContent.code().to_string()))
        .collect();
    assert_eq!(void_warnings.len(), 0);
}
