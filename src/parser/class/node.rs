use crate::parser::{HsmlResult, Span};

use super::process::process_class;

#[derive(Debug, Eq)]
pub struct ClassNode {
    pub name: String,
    /// Line number (1-based) where this class appears in the source.
    pub line: u32,
    /// Column number (1-based) where this class appears in the source.
    pub column: u32,
}

// PartialEq only compares `name` so that tests comparing parsed ASTs
// don't need to specify exact line/column values for every class.
impl PartialEq for ClassNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl ClassNode {
    /// Create a ClassNode with only a name (line/column default to 0).
    /// Useful in tests and compiler code where location is not relevant.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            line: 0,
            column: 0,
        }
    }
}

pub fn class_node(input: Span<'_>) -> HsmlResult<'_, ClassNode> {
    let (rest, class_name) = process_class(input)?;

    Ok((
        rest,
        ClassNode {
            name: class_name.to_string(),
            line: input.location_line(),
            column: input.get_column() as u32,
        },
    ))
}
