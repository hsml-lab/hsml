use crate::common::Location;
use crate::parser::{HsmlResult, Span};

use super::process::process_class;

#[derive(Debug, Eq)]
pub struct ClassNode {
    pub name: String,
    /// Source location where this class appears.
    pub location: Location,
}

// PartialEq only compares `name` so that tests comparing parsed ASTs
// don't need to specify exact location values for every class.
impl PartialEq for ClassNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl ClassNode {
    /// Create a ClassNode with only a name (location defaults to 0:0).
    /// Useful in tests and compiler code where location is not relevant.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: Location { line: 0, column: 0 },
        }
    }
}

pub fn class_node(input: Span<'_>) -> HsmlResult<'_, ClassNode> {
    let (rest, class_name) = process_class(input)?;

    Ok((
        rest,
        ClassNode {
            name: class_name.to_string(),
            location: Location {
                line: input.location_line(),
                column: input.get_column() as u32,
            },
        },
    ))
}
