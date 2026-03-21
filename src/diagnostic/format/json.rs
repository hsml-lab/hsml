use super::{Diagnostic, DiagnosticFormatter};
use crate::diagnostic::Severity;

/// JSON formatter for machine-readable diagnostic output.
///
/// Outputs a JSON array of diagnostic objects:
/// ```json
/// [{"severity":"error","message":"...","code":"E001","line":3,"column":5,"file":"example.hsml"}]
/// ```
pub struct JsonFormatter;

pub(crate) fn escape_json(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}

impl DiagnosticFormatter for JsonFormatter {
    fn format(&self, diagnostics: &[Diagnostic], _source: Option<&str>) -> String {
        let mut output = String::from("[");

        for (i, diag) in diagnostics.iter().enumerate() {
            if i > 0 {
                output.push(',');
            }

            let severity = match diag.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };

            output.push('{');
            output.push_str(&format!(
                "\"severity\":\"{severity}\",\"message\":\"{}\"",
                escape_json(&diag.message)
            ));

            if let Some(ref code) = diag.code {
                output.push_str(&format!(",\"code\":\"{}\"", escape_json(code)));
            }

            if let Some(ref loc) = diag.location {
                output.push_str(&format!(",\"line\":{},\"column\":{}", loc.line, loc.column));
            }

            if let Some(ref path) = diag.file_path {
                output.push_str(&format!(",\"file\":\"{}\"", escape_json(path)));
            }

            output.push('}');
        }

        output.push(']');
        output
    }
}
