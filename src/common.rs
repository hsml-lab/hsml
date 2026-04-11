use serde::Serialize;

/// A single point in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    /// Line number (1-based).
    pub line: u32,
    /// Column number (1-based).
    pub column: u32,
}

impl Position {
    /// Create a Position from a parser span's current location.
    pub fn from_span(span: &nom_locate::LocatedSpan<&str>) -> Self {
        Self {
            line: span.location_line(),
            column: span.get_column() as u32,
        }
    }
}

/// A span in source code, defined by a start and end position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Location {
    pub start: Position,
    pub end: Position,
}

/// HTML void elements that cannot have children and must not have a closing tag.
/// See: https://html.spec.whatwg.org/multipage/syntax.html#void-elements
pub const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// HTML raw text elements whose content is not parsed as HTML.
/// See: https://html.spec.whatwg.org/multipage/syntax.html#raw-text-elements
pub const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style"];

/// HTML RCDATA elements whose content allows character references but no child elements.
/// See: https://html.spec.whatwg.org/multipage/syntax.html#rcdata-elements
pub const RCDATA_ELEMENTS: &[&str] = &["textarea", "title"];

/// Check if a tag name is an HTML void element.
pub fn is_void_element(tag: &str) -> bool {
    let lower = tag.to_ascii_lowercase();
    VOID_ELEMENTS.contains(&lower.as_str())
}

/// Check if a tag name is an HTML raw text element.
pub fn is_raw_text_element(tag: &str) -> bool {
    let lower = tag.to_ascii_lowercase();
    RAW_TEXT_ELEMENTS.contains(&lower.as_str())
}

/// Check if a tag name is an HTML RCDATA element.
pub fn is_rcdata_element(tag: &str) -> bool {
    let lower = tag.to_ascii_lowercase();
    RCDATA_ELEMENTS.contains(&lower.as_str())
}

impl Location {
    /// Create a Location from two parser spans (start and end positions).
    pub fn from_spans(
        start: &nom_locate::LocatedSpan<&str>,
        end: &nom_locate::LocatedSpan<&str>,
    ) -> Self {
        Self {
            start: Position::from_span(start),
            end: Position::from_span(end),
        }
    }

    /// Returns true if this location represents a valid source position
    /// (not a sentinel value from `new_without_location`).
    pub fn is_valid(&self) -> bool {
        self.start.line > 0 && self.start.column > 0
    }
}
