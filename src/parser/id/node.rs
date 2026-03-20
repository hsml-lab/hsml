use crate::parser::{HsmlResult, Span};

use super::process::process_id;

#[derive(Debug, PartialEq, Eq)]
pub struct IdNode {
    pub id: String,
}

pub fn id_node(input: Span<'_>) -> HsmlResult<'_, IdNode> {
    let (input, id) = process_id(input)?;

    Ok((input, IdNode { id: id.to_string() }))
}
