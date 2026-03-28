use std::hash::{Hash, Hasher};

use crate::common::{Location, Position};
use crate::parser::{HsmlResult, Span};

use super::process::process_class;

#[derive(Debug, Eq)]
pub struct ClassNode {
    pub name: String,
    /// Source location where this class appears.
    pub location: Location,
}

// PartialEq and Hash only use `name` so that tests comparing parsed ASTs
// don't need to specify exact location values for every class, and the
// equality/hash invariant is preserved for hash-based collections.
impl PartialEq for ClassNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for ClassNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl ClassNode {
    /// Create a ClassNode with only a name (no source location).
    /// Useful in tests where location is not relevant.
    #[doc(hidden)]
    pub fn new_without_location(name: impl Into<String>) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            name: name.into(),
            location: Location {
                start: zero.clone(),
                end: zero,
            },
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
                start: Position {
                    line: input.location_line(),
                    column: input.get_column() as u32,
                },
                end: Position {
                    line: rest.location_line(),
                    column: rest.get_column() as u32,
                },
            },
        },
    ))
}
