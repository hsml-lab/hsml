use crate::diagnostic::format::DiagnosticFormatter;
use crate::diagnostic::format::gitlab::GitlabFormatter;
use crate::diagnostic::{Diagnostic, Location, Position, Severity};
use crate::parser::error::ErrorCode;

#[test]
fn it_should_format_warning_as_codeclimate_issue() {
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

    let output = GitlabFormatter.format(&[diag], None);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let arr = parsed.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["description"], "Duplicate class 'foo'");
    assert_eq!(arr[0]["check_name"], "W002");
    assert_eq!(arr[0]["severity"], "minor");
    assert_eq!(arr[0]["location"]["path"], "example.hsml");
    assert_eq!(arr[0]["location"]["lines"]["begin"], 1);
    assert!(arr[0]["fingerprint"].is_string());
}

#[test]
fn it_should_format_error_as_major_severity() {
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

    let output = GitlabFormatter.format(&[diag], None);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let arr = parsed.as_array().unwrap();

    assert_eq!(arr[0]["severity"], "major");
    assert_eq!(arr[0]["check_name"], "hsml");
}

#[test]
fn it_should_format_empty_diagnostics() {
    let output = GitlabFormatter.format(&[], None);
    assert_eq!(output, "[]");
}

#[test]
fn it_should_produce_stable_fingerprints() {
    let diag = Diagnostic {
        severity: Severity::Warning,
        message: "test".to_string(),
        code: Some("W001".to_string()),
        location: Some(Location {
            start: Position { line: 5, column: 3 },
            end: Position { line: 5, column: 3 },
        }),
        file_path: Some("a.hsml".to_string()),
    };

    let output1 = GitlabFormatter.format(&[diag.clone()], None);
    let output2 = GitlabFormatter.format(&[diag], None);

    let parsed1: serde_json::Value = serde_json::from_str(&output1).unwrap();
    let parsed2: serde_json::Value = serde_json::from_str(&output2).unwrap();

    assert_eq!(
        parsed1.as_array().unwrap()[0]["fingerprint"],
        parsed2.as_array().unwrap()[0]["fingerprint"]
    );
}
