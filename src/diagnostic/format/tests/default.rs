use crate::diagnostic::format::DiagnosticFormatter;
use crate::diagnostic::format::default::DefaultFormatter;
use crate::diagnostic::{Diagnostic, Location, Position, Severity};
use crate::parser::error::ErrorCode;

#[test]
fn it_should_format_warning_with_source_context() {
    let diag = Diagnostic {
        severity: Severity::Warning,
        message: "Duplicate attribute 'id' is not allowed".to_string(),
        code: Some(ErrorCode::DuplicateId.code().to_string()),
        location: Some(Location {
            start: Position { line: 1, column: 8 },
            end: Position { line: 1, column: 8 },
        }),
        file_path: Some("example.hsml".to_string()),
    };

    let source = "div#foo#bar";
    let output = DefaultFormatter.format(&[diag], Some(source));

    insta::assert_snapshot!(output);
}

#[test]
fn it_should_format_error_without_code() {
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "parse error".to_string(),
        code: None,
        location: Some(Location {
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 1 },
        }),
        file_path: None,
    };

    let source = "123invalid";
    let output = DefaultFormatter.format(&[diag], Some(source));

    insta::assert_snapshot!(output);
}

#[test]
fn it_should_format_error_without_location() {
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "Unsupported node type".to_string(),
        code: None,
        location: None,
        file_path: None,
    };

    let output = DefaultFormatter.format(&[diag], None);

    assert_eq!(output, "error: Unsupported node type\n");
}

#[test]
fn it_should_format_warning() {
    let diag = Diagnostic {
        severity: Severity::Warning,
        message: "Duplicate class 'text-red'".to_string(),
        code: Some(ErrorCode::DuplicateClass.code().to_string()),
        location: Some(Location {
            start: Position {
                line: 1,
                column: 12,
            },
            end: Position {
                line: 1,
                column: 12,
            },
        }),
        file_path: Some("test.hsml".to_string()),
    };

    let source = "h1.text-red.text-red Hello";
    let output = DefaultFormatter.format(&[diag], Some(source));

    insta::assert_snapshot!(output);
}

#[test]
fn it_should_format_multiple_diagnostics() {
    let diags = vec![
        Diagnostic {
            severity: Severity::Error,
            message: "first error".to_string(),
            code: None,
            location: None,
            file_path: None,
        },
        Diagnostic {
            severity: Severity::Error,
            message: "second error".to_string(),
            code: None,
            location: None,
            file_path: None,
        },
    ];

    let output = DefaultFormatter.format(&diags, None);

    assert_eq!(output, "error: first error\n\nerror: second error\n");
}

#[test]
fn it_should_underline_span_range() {
    let diag = Diagnostic {
        severity: Severity::Warning,
        message: "Duplicate class 'foo'".to_string(),
        code: Some(ErrorCode::DuplicateClass.code().to_string()),
        location: Some(Location {
            start: Position { line: 1, column: 7 },
            end: Position {
                line: 1,
                column: 11,
            },
        }),
        file_path: Some("test.hsml".to_string()),
    };

    let source = "h1.foo.foo Hello";
    let output = DefaultFormatter.format(&[diag], Some(source));

    insta::assert_snapshot!(output);
}
