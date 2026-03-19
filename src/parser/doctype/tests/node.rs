use crate::parser::doctype::node::{DoctypeNode, doctype_node};

#[test]
fn it_should_return_doctype_node() {
    let input = "doctype html\n";

    let (rest, node) = doctype_node(input).unwrap();

    assert_eq!(
        node,
        DoctypeNode {
            doctype: String::from("html"),
        }
    );
    assert_eq!(rest, "\n");
}
