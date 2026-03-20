use nom::IResult;

use crate::parser::Span;

use super::process::process_id;

#[derive(Debug, PartialEq, Eq)]
pub struct IdNode {
    pub id: String,
}

pub fn id_node(input: Span<'_>) -> IResult<Span<'_>, IdNode> {
    let (input, id) = process_id(input)?;

    Ok((input, IdNode { id: id.to_string() }))
}
