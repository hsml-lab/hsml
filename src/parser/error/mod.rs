use nom::error::{ErrorKind, ParseError};

use super::Span;

/// Registry of all HSML-specific errors.
///
/// New errors must be added as variants here. Each variant carries a unique
/// code and message, so collisions are impossible — the compiler enforces
/// that every variant is distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Errors (E001-E006)
    /// E001: Tag name must start with an ASCII letter. (since 0.2.0)
    InvalidTagName,
    /// E002: Unclosed bracket. (since 0.2.0)
    UnclosedBracket,
    /// E003: Unclosed parenthesis. (since 0.2.0)
    UnclosedParenthesis,
    /// E004: Unclosed quote in attribute value. (since 0.2.0)
    UnclosedQuote,
    /// E005: Expected quoted attribute value. (since 0.2.0)
    ExpectedAttributeValue,
    /// E006: Invalid attribute key. (since 0.2.0)
    InvalidAttributeKey,

    // Warnings (W001-W005)
    /// W001: Duplicate attribute 'id' is not allowed. (since 0.2.0)
    DuplicateId,
    /// W002: Duplicate class. (since 0.2.0)
    DuplicateClass,
    /// W003: Mixed tabs and spaces in indentation. (since 0.2.0)
    MixedIndentation,
    /// W004: Duplicate attribute. (since 0.2.0)
    DuplicateAttribute,
    /// W005: Void element cannot have children or text content. (since 0.5.0)
    VoidElementContent,
}

impl ErrorCode {
    /// All registered error code variants.
    /// Update this when adding new variants.
    pub const ALL: &[ErrorCode] = &[
        Self::InvalidTagName,
        Self::UnclosedBracket,
        Self::UnclosedParenthesis,
        Self::UnclosedQuote,
        Self::ExpectedAttributeValue,
        Self::InvalidAttributeKey,
        Self::DuplicateId,
        Self::DuplicateClass,
        Self::MixedIndentation,
        Self::DuplicateAttribute,
        Self::VoidElementContent,
    ];

    /// Machine-readable error code (e.g., "E001").
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidTagName => "E001",
            Self::UnclosedBracket => "E002",
            Self::UnclosedParenthesis => "E003",
            Self::UnclosedQuote => "E004",
            Self::ExpectedAttributeValue => "E005",
            Self::InvalidAttributeKey => "E006",
            Self::DuplicateId => "W001",
            Self::DuplicateClass => "W002",
            Self::MixedIndentation => "W003",
            Self::DuplicateAttribute => "W004",
            Self::VoidElementContent => "W005",
        }
    }

    /// Human-readable error message.
    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidTagName => "Tag name must start with an ASCII letter",
            Self::UnclosedBracket => "Unclosed bracket",
            Self::UnclosedParenthesis => "Unclosed parenthesis",
            Self::UnclosedQuote => "Unclosed quote in attribute value",
            Self::ExpectedAttributeValue => "Expected quoted attribute value",
            Self::InvalidAttributeKey => "Invalid attribute key",
            Self::DuplicateId => "Duplicate attribute 'id' is not allowed",
            Self::DuplicateClass => "Duplicate class",
            Self::MixedIndentation => "Mixed tabs and spaces in indentation",
            Self::DuplicateAttribute => "Duplicate attribute",
            Self::VoidElementContent => "Void element cannot have content",
        }
    }

    /// Default severity for this error code.
    pub fn severity(&self) -> Severity {
        match self {
            Self::InvalidTagName => Severity::Error,
            Self::UnclosedBracket => Severity::Error,
            Self::UnclosedParenthesis => Severity::Error,
            Self::UnclosedQuote => Severity::Error,
            Self::ExpectedAttributeValue => Severity::Error,
            Self::InvalidAttributeKey => Severity::Error,
            Self::DuplicateId => Severity::Warning,
            Self::DuplicateClass => Severity::Warning,
            Self::MixedIndentation => Severity::Warning,
            Self::DuplicateAttribute => Severity::Warning,
            Self::VoidElementContent => Severity::Warning,
        }
    }
}

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
    /// Optional registered error code.
    pub error_code: Option<ErrorCode>,
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
            error_code: None,
            severity: Severity::Error,
        }
    }

    /// Create a descriptive HSML error with a message.
    pub fn new(span: Span<'a>, message: impl Into<String>) -> Self {
        Self {
            span,
            kind: ErrorKind::Fail,
            message: Some(message.into()),
            error_code: None,
            severity: Severity::Error,
        }
    }

    /// Create an error from a registered error code.
    pub fn from_code(span: Span<'a>, error_code: ErrorCode) -> Self {
        Self {
            span,
            kind: ErrorKind::Fail,
            message: Some(error_code.message().to_string()),
            error_code: Some(error_code),
            severity: error_code.severity(),
        }
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

    /// Return a non-recoverable nom error (`nom::Err::Failure`) from a registered error code.
    pub fn fail_code(span: Span<'a>, error_code: ErrorCode) -> nom::Err<Self> {
        nom::Err::Failure(Self::from_code(span, error_code))
    }

    /// Machine-readable code string, if this is a registered error.
    pub fn code(&self) -> Option<&'static str> {
        self.error_code.map(|c| c.code())
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
            error_code: None,
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
            if let Some(code) = self.code() {
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

#[cfg(test)]
mod tests;
