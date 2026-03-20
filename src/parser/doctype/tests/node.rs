use crate::parser::Span;
use crate::parser::doctype::node::{DoctypeNode, doctype_node};

#[test]
fn it_should_return_doctype_node() {
    let input = Span::new("doctype html\n");

    let (rest, node) = doctype_node(input).unwrap();

    assert_eq!(
        node,
        DoctypeNode {
            doctype: String::from("html"),
        }
    );
    assert_eq!(*rest.fragment(), "\n");
}
