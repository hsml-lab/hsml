use serde::Serialize;

use crate::common::{Location, Position};
use crate::parser::{HsmlResult, Span};

use super::process::{process_dev_comment, process_native_comment};

#[derive(Debug, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentNode {
    pub text: String,
    pub is_dev: bool,
    pub location: Location,
}

// PartialEq excludes location so that tests comparing parsed ASTs
// don't need to specify exact location values for every comment.
impl PartialEq for CommentNode {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.is_dev == other.is_dev
    }
}

impl CommentNode {
    /// Create a CommentNode without a meaningful source location.
    /// Useful in tests where location is not relevant.
    #[doc(hidden)]
    pub fn new_without_location(text: impl Into<String>, is_dev: bool) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            text: text.into(),
            is_dev,
            location: Location {
                start: zero,
                end: zero,
            },
        }
    }
}

pub fn comment_dev_node(input: Span<'_>) -> HsmlResult<'_, CommentNode> {
    let start = Position {
        line: input.location_line(),
        column: input.get_column() as u32,
    };

    let (rest, comment) = process_dev_comment(input)?;

    Ok((
        rest,
        CommentNode {
            text: comment.to_string(),
            is_dev: true,
            location: Location {
                start,
                end: Position {
                    line: rest.location_line(),
                    column: rest.get_column() as u32,
                },
            },
        },
    ))
}

pub fn comment_native_node(input: Span<'_>) -> HsmlResult<'_, CommentNode> {
    let start = Position {
        line: input.location_line(),
        column: input.get_column() as u32,
    };

    let (rest, comment) = process_native_comment(input)?;

    Ok((
        rest,
        CommentNode {
            text: comment.to_string(),
            is_dev: false,
            location: Location {
                start,
                end: Position {
                    line: rest.location_line(),
                    column: rest.get_column() as u32,
                },
            },
        },
    ))
}
