use crate::parser::{
    HsmlNode, HsmlProcessContext,
    attribute::node::{AttributeNode, attribute_node, attribute_nodes},
    comment::node::CommentNode,
};

#[test]
fn it_should_return_attribute_node() {
    let mut context = HsmlProcessContext::default();

    let (input, attribute) = attribute_node(r#"key="value""#, &mut context).unwrap();

    assert_eq!(
        attribute,
        AttributeNode {
            key: String::from("key"),
            value: Some(String::from("value"))
        }
    );

    assert_eq!(input, "");
}

#[test]
fn it_should_return_attribute_node_with_multiline() {
    let mut context = HsmlProcessContext::default();

    let (input, attribute) = attribute_node(
        r#"class="{
        'is-active': isActive,
        'is-disabled': isDisabled,
    }"
    :key="item.id""#,
        &mut context,
    )
    .unwrap();

    assert_eq!(
        attribute,
        AttributeNode {
            key: String::from("class"),
            value: Some(String::from(
                r#"{
        'is-active': isActive,
        'is-disabled': isDisabled,
    }"#
            ))
        }
    );

    assert_eq!(
        input,
        r#"
    :key="item.id""#
    );
}

#[test]
fn it_should_return_attribute_nodes() {
    let mut context = HsmlProcessContext::default();

    let (input, attribute_nodes) =
        attribute_nodes(r#"(key="value", :key2="value2")"#, &mut context).unwrap();

    assert_eq!(
        attribute_nodes,
        vec![
            HsmlNode::Attribute(AttributeNode {
                key: String::from("key"),
                value: Some(String::from("value"))
            }),
            HsmlNode::Attribute(AttributeNode {
                key: String::from(":key2"),
                value: Some(String::from("value2"))
            })
        ]
    );

    assert_eq!(input, "");
}

#[test]
fn it_should_return_attribute_nodes_with_wrapped() {
    let mut context = HsmlProcessContext::default();

    let (input, attribute_nodes) = attribute_nodes(
        r#"(
    key="value"
    :key2="value2"
)
"#,
        &mut context,
    )
    .unwrap();

    assert_eq!(
        attribute_nodes,
        vec![
            HsmlNode::Attribute(AttributeNode {
                key: String::from("key"),
                value: Some(String::from("value"))
            }),
            HsmlNode::Attribute(AttributeNode {
                key: String::from(":key2"),
                value: Some(String::from("value2"))
            })
        ]
    );

    assert_eq!(input, "\n");
}

#[test]
fn it_should_return_attribute_nodes_with_dev_comments() {
    let mut context = HsmlProcessContext::default();

    let (input, attribute_nodes) = attribute_nodes(
        r#"(
    // comment 1
    key="value"
    // comment 2
    :key2="value2"
)
"#,
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
            HsmlNode::Attribute(AttributeNode {
                key: String::from("key"),
                value: Some(String::from("value")),
            }),
            HsmlNode::Comment(CommentNode {
                is_dev: true,
                text: String::from(" comment 2"),
            }),
            HsmlNode::Attribute(AttributeNode {
                key: String::from(":key2"),
                value: Some(String::from("value2")),
            }),
        ]
    );

    assert_eq!(input, "\n");
}

#[test]
fn it_should_return_attribute_nodes_with_multiline() {
    let mut context = HsmlProcessContext::default();

    let (input, attributes) = attribute_nodes(
        r#"(class="{
        'is-active': isActive,
        'is-disabled': isDisabled,
    }"
    :key="item.id")"#,
        &mut context,
    )
    .unwrap();

    assert_eq!(
        attributes,
        vec![
            HsmlNode::Attribute(AttributeNode {
                key: String::from("class"),
                value: Some(String::from(
                    r#"{
        'is-active': isActive,
        'is-disabled': isDisabled,
    }"#
                )),
            }),
            HsmlNode::Attribute(AttributeNode {
                key: String::from(":key"),
                value: Some(String::from("item.id")),
            }),
        ]
    );

    assert_eq!(input, "");
}
