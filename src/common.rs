use serde::Serialize;

/// A single point in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    /// Line number (1-based).
    pub line: u32,
    /// Column number (1-based).
    pub column: u32,
}

/// A span in source code, defined by a start and end position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Location {
    pub start: Position,
    pub end: Position,
}

impl Location {
    /// Returns true if this location represents a valid source position
    /// (not a sentinel value from `new_without_location`).
    pub fn is_valid(&self) -> bool {
        self.start.line > 0 && self.start.column > 0
    }
}
