use nom::IResult;

use super::process::process_doctype;

#[derive(Debug, PartialEq, Eq)]
pub struct DoctypeNode {
    pub doctype: String,
}

pub fn doctype_node(input: &str) -> IResult<&str, DoctypeNode> {
    let (input, doctype) = process_doctype(input)?;

    Ok((
        input,
        DoctypeNode {
            doctype: doctype.to_string(),
        },
    ))
}
