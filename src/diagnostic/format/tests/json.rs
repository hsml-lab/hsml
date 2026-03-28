use crate::diagnostic::format::DiagnosticFormatter;
use crate::diagnostic::format::json::JsonFormatter;
use crate::diagnostic::{Diagnostic, Location, Position, Severity};
use crate::parser::error::ErrorCode;

#[test]
fn it_should_format_single_diagnostic() {
    let diag = Diagnostic {
        severity: Severity::Warning,
        message: "Duplicate attribute 'id' is not allowed".to_string(),
        code: Some(ErrorCode::DuplicateId.code().to_string()),
        location: Some(Location {
            start: Position { line: 3, column: 5 },
            end: Position { line: 3, column: 5 },
        }),
        file_path: Some("example.hsml".to_string()),
    };

    let output = JsonFormatter.format(&[diag], None);

    assert_eq!(
        output,
        r#"[{"severity":"warning","message":"Duplicate attribute 'id' is not allowed","code":"W001","location":{"start":{"line":3,"column":5},"end":{"line":3,"column":5}},"filePath":"example.hsml"}]"#
    );
}

#[test]
fn it_should_format_diagnostic_without_optional_fields() {
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "parse error".to_string(),
        code: None,
        location: None,
        file_path: None,
    };

    let output = JsonFormatter.format(&[diag], None);

    assert_eq!(output, r#"[{"severity":"error","message":"parse error"}]"#);
}

#[test]
fn it_should_format_empty_diagnostics() {
    let output = JsonFormatter.format(&[], None);

    assert_eq!(output, "[]");
}

#[test]
fn it_should_format_multiple_diagnostics() {
    let diags = vec![
        Diagnostic {
            severity: Severity::Error,
            message: "first".to_string(),
            code: None,
            location: None,
            file_path: None,
        },
        Diagnostic {
            severity: Severity::Warning,
            message: "second".to_string(),
            code: None,
            location: None,
            file_path: None,
        },
    ];

    let output = JsonFormatter.format(&diags, None);

    assert_eq!(
        output,
        r#"[{"severity":"error","message":"first"},{"severity":"warning","message":"second"}]"#
    );
}

#[test]
fn it_should_escape_special_characters_in_json() {
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unexpected \"quote\" and \\backslash".to_string(),
        code: None,
        location: None,
        file_path: None,
    };

    let output = JsonFormatter.format(&[diag], None);

    assert_eq!(
        output,
        r#"[{"severity":"error","message":"unexpected \"quote\" and \\backslash"}]"#
    );
}
