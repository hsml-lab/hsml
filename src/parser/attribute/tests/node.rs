use crate::parser::{
    HsmlNode, HsmlProcessContext, Span,
    attribute::node::{AttributeNode, attribute_node, attribute_nodes},
    comment::node::CommentNode,
};

#[test]
fn it_should_return_attribute_node() {
    let mut context = HsmlProcessContext::default();

    let (rest, attribute) = attribute_node(Span::new(r#"key="value""#), &mut context).unwrap();

    assert_eq!(
        attribute,
        AttributeNode::new_without_location("key", Some("value"))
    );

    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_return_attribute_node_with_multiline() {
    let mut context = HsmlProcessContext::default();

    let (rest, attribute) = attribute_node(
        Span::new(
            r#"class="{
        'is-active': isActive,
        'is-disabled': isDisabled,
    }"
    :key="item.id""#,
        ),
        &mut context,
    )
    .unwrap();

    assert_eq!(
        attribute,
        AttributeNode::new_without_location(
            "class",
            Some(
                r#"{
        'is-active': isActive,
        'is-disabled': isDisabled,
    }"#
            )
        )
    );

    assert_eq!(
        *rest.fragment(),
        r#"
    :key="item.id""#
    );
}

#[test]
fn it_should_return_attribute_nodes() {
    let mut context = HsmlProcessContext::default();

    let (rest, attribute_nodes) =
        attribute_nodes(Span::new(r#"(key="value", :key2="value2")"#), &mut context).unwrap();

    assert_eq!(
        attribute_nodes,
        vec![
            HsmlNode::Attribute(AttributeNode::new_without_location("key", Some("value"))),
            HsmlNode::Attribute(AttributeNode::new_without_location(":key2", Some("value2")))
        ]
    );

    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_return_attribute_nodes_with_wrapped() {
    let mut context = HsmlProcessContext::default();

    let (rest, attribute_nodes) = attribute_nodes(
        Span::new(
            r#"(
    key="value"
    :key2="value2"
)
"#,
        ),
        &mut context,
    )
    .unwrap();

    assert_eq!(
        attribute_nodes,
        vec![
            HsmlNode::Attribute(AttributeNode::new_without_location("key", Some("value"))),
            HsmlNode::Attribute(AttributeNode::new_without_location(":key2", Some("value2")))
        ]
    );

    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_return_attribute_nodes_with_dev_comments() {
    let mut context = HsmlProcessContext::default();

    let (rest, attribute_nodes) = attribute_nodes(
        Span::new(
            r#"(
    // comment 1
    key="value"
    // comment 2
    :key2="value2"
)
"#,
        ),
        &mut context,
    )
    .unwrap();

    assert_eq!(
        attribute_nodes,
        vec![
            HsmlNode::Comment(CommentNode {
                is_dev: true,
                text: String::from(" comment 1"),
            }),
            HsmlNode::Attribute(AttributeNode::new_without_location("key", Some("value"))),
            HsmlNode::Comment(CommentNode {
                is_dev: true,
                text: String::from(" comment 2"),
            }),
            HsmlNode::Attribute(AttributeNode::new_without_location(":key2", Some("value2"))),
        ]
    );

    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_return_attribute_nodes_with_multiline() {
    let mut context = HsmlProcessContext::default();

    let (rest, attributes) = attribute_nodes(
        Span::new(
            r#"(class="{
        'is-active': isActive,
        'is-disabled': isDisabled,
    }"
    :key="item.id")"#,
        ),
        &mut context,
    )
    .unwrap();

    assert_eq!(
        attributes,
        vec![
            HsmlNode::Attribute(AttributeNode::new_without_location(
                "class",
                Some(
                    r#"{
        'is-active': isActive,
        'is-disabled': isDisabled,
    }"#
                )
            )),
            HsmlNode::Attribute(AttributeNode::new_without_location(":key", Some("item.id"))),
        ]
    );

    assert_eq!(*rest.fragment(), "");
}
