use crate::diagnostic::format::DiagnosticFormatter;
use crate::diagnostic::format::github::GithubFormatter;
use crate::diagnostic::{Diagnostic, Location, Position, Severity};
use crate::parser::error::ErrorCode;

#[test]
fn it_should_format_warning_with_location() {
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
        file_path: Some("example.hsml".to_string()),
    };

    let output = GithubFormatter.format(&[diag], None);

    assert_eq!(
        output,
        "::warning file=example.hsml,line=1,col=7,endLine=1,endColumn=11,title=W002::Duplicate class 'foo'\n"
    );
}

#[test]
fn it_should_format_error_without_end_location() {
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "parse error".to_string(),
        code: None,
        location: Some(Location {
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 1 },
        }),
        file_path: Some("test.hsml".to_string()),
    };

    let output = GithubFormatter.format(&[diag], None);

    assert_eq!(output, "::error file=test.hsml,line=1,col=1::parse error\n");
}

#[test]
fn it_should_format_diagnostic_without_location() {
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "Unsupported node".to_string(),
        code: None,
        location: None,
        file_path: None,
    };

    let output = GithubFormatter.format(&[diag], None);

    assert_eq!(output, "::error::Unsupported node\n");
}

#[test]
fn it_should_format_multiple_diagnostics() {
    let diags = vec![
        Diagnostic {
            severity: Severity::Error,
            message: "first".to_string(),
            code: Some("E001".to_string()),
            location: Some(Location {
                start: Position { line: 1, column: 1 },
                end: Position { line: 1, column: 1 },
            }),
            file_path: Some("a.hsml".to_string()),
        },
        Diagnostic {
            severity: Severity::Warning,
            message: "second".to_string(),
            code: Some("W002".to_string()),
            location: Some(Location {
                start: Position { line: 3, column: 5 },
                end: Position {
                    line: 3,
                    column: 10,
                },
            }),
            file_path: Some("b.hsml".to_string()),
        },
    ];

    let output = GithubFormatter.format(&diags, None);

    assert_eq!(
        output,
        "::error file=a.hsml,line=1,col=1,title=E001::first\n::warning file=b.hsml,line=3,col=5,endLine=3,endColumn=10,title=W002::second\n"
    );
}
