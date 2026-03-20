use nom::error::{ErrorKind, ParseError};

use super::Span;

/// A registered error definition with a unique code and message template.
pub struct ErrorDef {
    pub code: &'static str,
    pub message: &'static str,
}

// --- Error registry ---
// All HSML-specific errors are defined here to prevent code collisions.

pub const DUPLICATE_ID: ErrorDef = ErrorDef {
    code: "E001",
    message: "Duplicate attribute 'id' is not allowed",
};

/// Severity level for parser diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Custom error type for the HSML parser.
///
/// Carries location information, a human-readable message, an optional
/// error code, and a severity level. Format-agnostic: does not depend
/// on any rendering/reporting crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HsmlError<'a> {
    /// The span where the error occurred (carries line/column via nom_locate).
    pub span: Span<'a>,
    /// The nom ErrorKind, preserved for compatibility with nom combinators.
    pub kind: ErrorKind,
    /// Optional human-readable description (only set for custom HSML errors,
    /// not for generic nom combinator errors).
    pub message: Option<String>,
    /// Optional machine-readable error code (e.g., "E001").
    pub code: Option<&'static str>,
    /// Severity level.
    pub severity: Severity,
}

impl<'a> HsmlError<'a> {
    /// Create a generic error from a nom ErrorKind (no descriptive message).
    pub fn from_kind(span: Span<'a>, kind: ErrorKind) -> Self {
        Self {
            span,
            kind,
            message: None,
            code: None,
            severity: Severity::Error,
        }
    }

    /// Create a descriptive HSML error with a message.
    pub fn new(span: Span<'a>, message: impl Into<String>) -> Self {
        Self {
            span,
            kind: ErrorKind::Fail,
            message: Some(message.into()),
            code: None,
            severity: Severity::Error,
        }
    }

    /// Builder: attach an error code.
    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }

    /// Return a recoverable nom error (`nom::Err::Error`) with a generic ErrorKind.
    pub fn err(span: Span<'a>, kind: ErrorKind) -> nom::Err<Self> {
        nom::Err::Error(Self::from_kind(span, kind))
    }

    /// Return a non-recoverable nom error (`nom::Err::Failure`) with a generic ErrorKind.
    pub fn fail(span: Span<'a>, kind: ErrorKind) -> nom::Err<Self> {
        nom::Err::Failure(Self::from_kind(span, kind))
    }

    /// Return a non-recoverable nom error (`nom::Err::Failure`) with a descriptive message.
    pub fn fail_msg(span: Span<'a>, message: impl Into<String>) -> nom::Err<Self> {
        nom::Err::Failure(Self::new(span, message))
    }

    /// Return a non-recoverable nom error (`nom::Err::Failure`) from a registered error definition.
    pub fn fail_def(span: Span<'a>, def: &ErrorDef) -> nom::Err<Self> {
        nom::Err::Failure(Self::new(span, def.message).with_code(def.code))
    }

    /// Line number (1-based) from nom_locate.
    pub fn line(&self) -> u32 {
        self.span.location_line()
    }

    /// Column number (1-based) from nom_locate.
    pub fn column(&self) -> usize {
        self.span.get_column()
    }
}

impl<'a> ParseError<Span<'a>> for HsmlError<'a> {
    fn from_error_kind(input: Span<'a>, kind: ErrorKind) -> Self {
        Self {
            span: input,
            kind,
            message: None,
            code: None,
            severity: Severity::Error,
        }
    }

    fn append(_input: Span<'a>, _kind: ErrorKind, other: Self) -> Self {
        // Like nom's default Error, keep the deeper error.
        other
    }
}

impl<'a> std::fmt::Display for HsmlError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref msg) = self.message {
            if let Some(code) = self.code {
                write!(
                    f,
                    "[{}] {} at line {}, column {}",
                    code,
                    msg,
                    self.line(),
                    self.column()
                )
            } else {
                write!(
                    f,
                    "{} at line {}, column {}",
                    msg,
                    self.line(),
                    self.column()
                )
            }
        } else {
            write!(
                f,
                "parse error ({:?}) at line {}, column {}",
                self.kind,
                self.line(),
                self.column()
            )
        }
    }
}
