use super::{Diagnostic, DiagnosticFormatter};
use crate::diagnostic::Severity;

/// Pretty terminal formatter with source context.
///
/// Output format:
/// ```text
/// error[E001]: Duplicate attribute 'id' is not allowed
///  --> example.hsml:3:5
///   |
/// 3 | div#foo#bar
///   |        ^
/// ```
pub struct DefaultFormatter;

impl DiagnosticFormatter for DefaultFormatter {
    fn format(&self, diagnostics: &[Diagnostic], source: Option<&str>) -> String {
        let mut output = String::new();

        for (i, diag) in diagnostics.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }

            // Severity + optional code + message
            let severity = match diag.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };

            if let Some(ref code) = diag.code {
                output.push_str(&format!("{severity}[{code}]: {}", diag.message));
            } else {
                output.push_str(&format!("{severity}: {}", diag.message));
            }
            output.push('\n');

            // Location
            if let Some(ref loc) = diag.location {
                let file = diag.file_path.as_deref().unwrap_or("<input>");
                output.push_str(&format!(" --> {file}:{}:{}\n", loc.line, loc.column));

                // Source context
                if let Some(source) = source
                    && let Some(source_line) =
                        source.lines().nth(loc.line.saturating_sub(1) as usize)
                {
                    let line_num = loc.line.to_string();
                    let padding = " ".repeat(line_num.len());

                    output.push_str(&format!("{padding} |\n"));
                    output.push_str(&format!("{line_num} | {source_line}\n"));

                    if loc.column > 0 {
                        let caret_padding = " ".repeat((loc.column - 1) as usize);
                        output.push_str(&format!("{padding} | {caret_padding}^\n"));
                    }
                }
            }
        }

        output
    }
}
