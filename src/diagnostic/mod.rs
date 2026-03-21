pub mod format;

pub use crate::common::Location;
use crate::parser::error::{self, HsmlError};

/// Severity level for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A format-agnostic diagnostic message.
///
/// This is the stable output type for errors and warnings.
/// Both parser and compiler errors are converted into this type
/// before being rendered by formatters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub code: Option<String>,
    pub location: Option<Location>,
    pub file_path: Option<String>,
}

impl Diagnostic {
    /// Create a compiler error diagnostic (no location info).
    pub fn compiler_error(message: String) -> Self {
        Self {
            severity: Severity::Error,
            message,
            code: None,
            location: None,
            file_path: None,
        }
    }

    /// Attach a file path to this diagnostic.
    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Serialize this diagnostic as a JSON object string.
    pub fn to_json(&self) -> String {
        use format::json::escape_json;

        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };

        let mut output = format!(
            "{{\"severity\":\"{severity}\",\"message\":\"{}\"",
            escape_json(&self.message)
        );

        if let Some(ref code) = self.code {
            output.push_str(&format!(",\"code\":\"{}\"", escape_json(code)));
        }

        if let Some(ref loc) = self.location {
            output.push_str(&format!(",\"line\":{},\"column\":{}", loc.line, loc.column));
        }

        if let Some(ref path) = self.file_path {
            output.push_str(&format!(",\"file\":\"{}\"", escape_json(path)));
        }

        output.push('}');
        output
    }

    /// Serialize a slice of diagnostics as a JSON array string.
    pub fn slice_to_json(diagnostics: &[Diagnostic]) -> String {
        let mut output = String::from("[");
        for (i, diag) in diagnostics.iter().enumerate() {
            if i > 0 {
                output.push(',');
            }
            output.push_str(&diag.to_json());
        }
        output.push(']');
        output
    }
}

impl From<&error::Severity> for Severity {
    fn from(s: &error::Severity) -> Self {
        match s {
            error::Severity::Error => Severity::Error,
            error::Severity::Warning => Severity::Warning,
        }
    }
}

impl<'a> From<&HsmlError<'a>> for Diagnostic {
    fn from(e: &HsmlError<'a>) -> Self {
        Diagnostic {
            severity: Severity::from(&e.severity),
            message: e
                .message
                .clone()
                .unwrap_or_else(|| "parse error".to_string()),
            code: e.code().map(String::from),
            location: Some(Location {
                line: e.line(),
                column: e.column() as u32,
            }),
            file_path: None,
        }
    }
}

impl<'a> From<&nom::Err<HsmlError<'a>>> for Diagnostic {
    fn from(e: &nom::Err<HsmlError<'a>>) -> Self {
        match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => Diagnostic::from(e),
            nom::Err::Incomplete(_) => Diagnostic {
                severity: Severity::Error,
                message: "Unexpected end of input".to_string(),
                code: None,
                location: None,
                file_path: None,
            },
        }
    }
}

#[cfg(test)]
mod tests;
