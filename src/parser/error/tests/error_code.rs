use std::collections::HashSet;

use crate::parser::error::ErrorCode;

#[test]
fn error_codes_are_unique() {
    // Update this array when adding new ErrorCode variants.
    let codes = [ErrorCode::DuplicateId];

    let mut seen = HashSet::new();
    for code in &codes {
        assert!(
            seen.insert(code.code()),
            "duplicate error code: {}",
            code.code()
        );
    }
}
