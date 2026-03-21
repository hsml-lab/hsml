use std::collections::HashSet;

use crate::parser::Span;
use crate::parser::error::{ErrorCode, HsmlError, Severity};

#[test]
fn error_codes_are_unique() {
    // Update this array when adding new ErrorCode variants.
    let codes = [
        ErrorCode::DuplicateId,
        ErrorCode::MixedIndentation,
        ErrorCode::DuplicateClass,
    ];

    let mut seen = HashSet::new();
    for code in &codes {
        assert!(
            seen.insert(code.code()),
            "duplicate error code: {}",
            code.code()
        );
    }
}

#[test]
fn from_code_uses_error_code_severity() {
    let span = Span::new("test");

    let result = HsmlError::from_code(span, ErrorCode::DuplicateId);
    assert_eq!(result.severity, Severity::Error);

    let result = HsmlError::from_code(span, ErrorCode::MixedIndentation);
    assert_eq!(result.severity, Severity::Warning);

    let result = HsmlError::from_code(span, ErrorCode::DuplicateClass);
    assert_eq!(result.severity, Severity::Warning);
}
