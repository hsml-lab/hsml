/// Source location, independent of any parser or diagnostic layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// Line number (1-based).
    pub line: u32,
    /// Column number (1-based).
    pub column: u32,
}
