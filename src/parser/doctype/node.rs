use nom::IResult;

use crate::parser::Span;

use super::process::process_doctype;

#[derive(Debug, PartialEq, Eq)]
pub struct DoctypeNode {
    pub doctype: String,
}

pub fn doctype_node(input: Span<'_>) -> IResult<Span<'_>, DoctypeNode> {
    let (input, doctype) = process_doctype(input)?;

    Ok((
        input,
        DoctypeNode {
            doctype: doctype.to_string(),
        },
    ))
}
