use super::{Diagnostic, DiagnosticFormatter};

/// JSON formatter for machine-readable diagnostic output.
///
/// Outputs a JSON array of diagnostic objects using serde serialization.
pub struct JsonFormatter;

impl DiagnosticFormatter for JsonFormatter {
    fn format(&self, diagnostics: &[Diagnostic], _source: Option<&str>) -> String {
        serde_json::to_string(diagnostics).unwrap_or_else(|_| "[]".to_string())
    }
}
