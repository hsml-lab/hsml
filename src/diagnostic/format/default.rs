use super::{Diagnostic, DiagnosticFormatter};
use crate::diagnostic::Severity;

/// Pretty terminal formatter with source context (no colors).
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

/// Pretty terminal formatter with ANSI colors.
pub struct DefaultColorFormatter;

struct Colors {
    red: &'static str,
    yellow: &'static str,
    blue: &'static str,
    bold: &'static str,
    dim: &'static str,
    reset: &'static str,
}

const NO_COLORS: Colors = Colors {
    red: "",
    yellow: "",
    blue: "",
    bold: "",
    dim: "",
    reset: "",
};

const ANSI_COLORS: Colors = Colors {
    red: "\x1b[31m",
    yellow: "\x1b[33m",
    blue: "\x1b[34m",
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    reset: "\x1b[0m",
};

fn format_diagnostics(diagnostics: &[Diagnostic], source: Option<&str>, c: &Colors) -> String {
    let mut output = String::new();

    for (i, diag) in diagnostics.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }

        // Severity + optional code + message
        let (severity_label, severity_color) = match diag.severity {
            Severity::Error => ("error", c.red),
            Severity::Warning => ("warning", c.yellow),
        };

        if let Some(ref code) = diag.code {
            output.push_str(&format!(
                "{c_bold}{severity_color}{severity_label}[{code}]{reset}: {}",
                diag.message,
                c_bold = c.bold,
                reset = c.reset,
            ));
        } else {
            output.push_str(&format!(
                "{c_bold}{severity_color}{severity_label}{reset}: {}",
                diag.message,
                c_bold = c.bold,
                reset = c.reset,
            ));
        }
        output.push('\n');

        // Location
        if let Some(ref loc) = diag.location {
            let file = diag.file_path.as_deref().unwrap_or("<input>");
            output.push_str(&format!(
                " {dim}-->{reset} {file}:{}:{}\n",
                loc.start.line,
                loc.start.column,
                dim = c.dim,
                reset = c.reset,
            ));

            // Source context
            if let Some(source) = source
                && let Some(line_idx) = loc.start.line.checked_sub(1)
                && let Some(source_line) = source.lines().nth(line_idx as usize)
            {
                let line_num = loc.start.line.to_string();
                let padding = " ".repeat(line_num.len());

                output.push_str(&format!(
                    "{blue}{padding} |{reset}\n",
                    blue = c.blue,
                    reset = c.reset,
                ));
                output.push_str(&format!(
                    "{blue}{line_num} |{reset} {source_line}\n",
                    blue = c.blue,
                    reset = c.reset,
                ));

                if loc.start.column > 0 {
                    let caret_padding = " ".repeat((loc.start.column - 1) as usize);
                    output.push_str(&format!(
                        "{blue}{padding} |{reset} {caret_padding}{severity_color}^{reset}\n",
                        blue = c.blue,
                        reset = c.reset,
                    ));
                }
            }
        }
    }

    output
}

impl DiagnosticFormatter for DefaultFormatter {
    fn format(&self, diagnostics: &[Diagnostic], source: Option<&str>) -> String {
        format_diagnostics(diagnostics, source, &NO_COLORS)
    }
}

impl DiagnosticFormatter for DefaultColorFormatter {
    fn format(&self, diagnostics: &[Diagnostic], source: Option<&str>) -> String {
        format_diagnostics(diagnostics, source, &ANSI_COLORS)
    }
}
