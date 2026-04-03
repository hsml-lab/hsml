use serde::Serialize;

use crate::parser::{HsmlResult, Span};

use super::process::process_doctype;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DoctypeNode {
    pub doctype: String,
}

pub fn doctype_node(input: Span<'_>) -> HsmlResult<'_, DoctypeNode> {
    let (input, doctype) = process_doctype(input)?;

    Ok((
        input,
        DoctypeNode {
            doctype: doctype.to_string(),
        },
    ))
}
