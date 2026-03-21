use crate::diagnostic::Severity;
use crate::parser::Span;
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
    assert_eq!(diagnostics[0].code, Some("W001".to_string()));
    assert!(diagnostics[0].location.is_some());
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
}

#[test]
fn it_should_warn_on_multiple_duplicates() {
    let source = "h1.a.b.a.b Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message, "Duplicate class 'a'");
    assert_eq!(diagnostics[1].message, "Duplicate class 'b'");
}

#[test]
fn it_should_not_warn_on_no_classes() {
    let source = "h1 Hello\n";
    let (_, ast) = parse(Span::new(source)).unwrap();

    let diagnostics = validate(&ast, source);

    assert!(diagnostics.is_empty());
}
