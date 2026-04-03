use std::hash::{Hash, Hasher};

use crate::common::{Location, Position};
use crate::parser::{HsmlResult, Span};

use super::process::process_id;

#[derive(Debug, Eq, serde::Serialize)]
pub struct IdNode {
    pub id: String,
    /// Source location where this id appears.
    pub location: Location,
}

// PartialEq and Hash only use `id` so that tests comparing parsed ASTs
// don't need to specify exact location values.
impl PartialEq for IdNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for IdNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl IdNode {
    /// Create an IdNode with only an id (no source location).
    /// Useful in tests where location is not relevant.
    #[doc(hidden)]
    pub fn new_without_location(id: impl Into<String>) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            id: id.into(),
            location: Location {
                start: zero,
                end: zero,
            },
        }
    }
}

pub fn id_node(input: Span<'_>) -> HsmlResult<'_, IdNode> {
    let (rest, id) = process_id(input)?;

    Ok((
        rest,
        IdNode {
            id: id.to_string(),
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
