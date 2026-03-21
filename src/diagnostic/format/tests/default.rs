use crate::diagnostic::format::DiagnosticFormatter;
use crate::diagnostic::format::default::DefaultFormatter;
use crate::diagnostic::{Diagnostic, Location, Severity};

#[test]
fn it_should_format_error_with_source_context() {
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "Duplicate attribute 'id' is not allowed".to_string(),
        code: Some("E001".to_string()),
        location: Some(Location { line: 1, column: 8 }),
        file_path: Some("example.hsml".to_string()),
    };

    let source = "div#foo#bar";
    let output = DefaultFormatter.format(&[diag], Some(source));

    assert_eq!(
        output,
        "\
error[E001]: Duplicate attribute 'id' is not allowed
 --> example.hsml:1:8
  |
1 | div#foo#bar
  |        ^
"
    );
}

#[test]
fn it_should_format_error_without_code() {
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "parse error".to_string(),
        code: None,
        location: Some(Location { line: 1, column: 1 }),
        file_path: None,
    };

    let source = "123invalid";
    let output = DefaultFormatter.format(&[diag], Some(source));

    assert_eq!(
        output,
        "\
error: parse error
 --> <input>:1:1
  |
1 | 123invalid
  | ^
"
    );
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
        code: Some("W001".to_string()),
        location: Some(Location {
            line: 1,
            column: 12,
        }),
        file_path: Some("test.hsml".to_string()),
    };

    let source = "h1.text-red.text-red Hello";
    let output = DefaultFormatter.format(&[diag], Some(source));

    assert_eq!(
        output,
        "\
warning[W001]: Duplicate class 'text-red'
 --> test.hsml:1:12
  |
1 | h1.text-red.text-red Hello
  |            ^
"
    );
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
