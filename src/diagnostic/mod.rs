pub mod format;

pub use crate::common::Location;
use crate::parser::error::{self, HsmlError};
use serde::Serialize;

/// Severity level for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// A fatal error that prevents successful compilation.
    Error,
    /// A non-fatal issue that does not prevent compilation.
    Warning,
}

/// A format-agnostic diagnostic message.
///
/// This is the stable output type for errors and warnings.
/// Both parser and compiler errors are converted into this type
/// before being rendered by formatters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

    /// Serialize a slice of diagnostics as a JSON array string.
    pub fn slice_to_json(diagnostics: &[Diagnostic]) -> String {
        serde_json::to_string(diagnostics).unwrap_or_else(|_| "[]".to_string())
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
