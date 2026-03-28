use hsml::common::Location;
use hsml::diagnostic::{Diagnostic, Severity};
use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, Range};

use crate::cli::exec_lsp::to_lsp_diagnostic;

#[test]
fn it_should_map_error_severity() {
    let d = Diagnostic {
        severity: Severity::Error,
        message: "parse error".to_string(),
        code: Some("E001".to_string()),
        location: Some(Location { line: 1, column: 1 }),
        file_path: None,
    };

    let lsp = to_lsp_diagnostic(&d);

    assert_eq!(lsp.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(lsp.message, "parse error");
    assert_eq!(lsp.code, Some(NumberOrString::String("E001".to_string())));
    assert_eq!(lsp.source, Some("hsml".to_string()));
}

#[test]
fn it_should_map_warning_severity() {
    let d = Diagnostic {
        severity: Severity::Warning,
        message: "duplicate class".to_string(),
        code: Some("W002".to_string()),
        location: Some(Location {
            line: 5,
            column: 10,
        }),
        file_path: None,
    };

    let lsp = to_lsp_diagnostic(&d);

    assert_eq!(lsp.severity, Some(DiagnosticSeverity::WARNING));
    assert_eq!(lsp.code, Some(NumberOrString::String("W002".to_string())));
}

#[test]
fn it_should_convert_1based_to_0based() {
    let d = Diagnostic {
        severity: Severity::Error,
        message: "test".to_string(),
        code: None,
        location: Some(Location {
            line: 10,
            column: 5,
        }),
        file_path: None,
    };

    let lsp = to_lsp_diagnostic(&d);

    assert_eq!(lsp.range.start.line, 9);
    assert_eq!(lsp.range.start.character, 4);
    assert_eq!(lsp.range.end, lsp.range.start);
}

#[test]
fn it_should_handle_no_location() {
    let d = Diagnostic {
        severity: Severity::Error,
        message: "no location".to_string(),
        code: None,
        location: None,
        file_path: None,
    };

    let lsp = to_lsp_diagnostic(&d);

    assert_eq!(lsp.range, Range::default());
}

#[test]
fn it_should_handle_no_code() {
    let d = Diagnostic {
        severity: Severity::Error,
        message: "no code".to_string(),
        code: None,
        location: None,
        file_path: None,
    };

    let lsp = to_lsp_diagnostic(&d);

    assert_eq!(lsp.code, None);
}
