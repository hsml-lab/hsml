use crate::parser::{
    HsmlNode, RootNode, attribute::node::AttributeNode, comment::node::CommentNode,
    tag::node::TagNode,
};

#[derive(Default)]
pub struct HsmlCompileOptions {}

fn compile_tag_node(tag_node: &TagNode, _options: &HsmlCompileOptions) -> Result<String, String> {
    let mut html_content = String::new();

    html_content.push('<');
    html_content.push_str(&tag_node.tag);

    if let Some(id_node) = &tag_node.id {
        html_content.push_str(r#" id=""#);
        html_content.push_str(&id_node.id);
        html_content.push('\"');
    }

    if let Some(class_nodes) = &tag_node.classes {
        html_content.push_str(r#" class=""#);

        let class_names: String = class_nodes
            .iter()
            .map(|class_node| class_node.name.as_str())
            .collect::<Vec<&str>>()
            .join(" ");

        html_content.push_str(&class_names);

        html_content.push('\"');
    }

    if let Some(attributes) = &tag_node.attributes {
        for node in attributes {
            match node {
                HsmlNode::Attribute(AttributeNode { key, value }) => {
                    html_content.push(' ');
                    html_content.push_str(key);

                    if let Some(value) = value {
                        html_content.push_str(r#"=""#);
                        html_content.push_str(value);
                        html_content.push('\"');
                    }
                }
                HsmlNode::Comment(node) if node.is_dev => {
                    // do nothing
                }
                other => {
                    return Err(format!(
                        "Unsupported node type in attributes of <{}>: {other:?}",
                        tag_node.tag
                    ));
                }
            }
        }
    }

    let should_auto_close = tag_node.children.is_none() && tag_node.text.is_none();
    if should_auto_close {
        html_content.push_str("/>");
        return Ok(html_content);
    } else {
        html_content.push('>');
    }

    if let Some(text) = &tag_node.text {
        html_content.push_str(&text.text);
    }

    if let Some(child_nodes) = &tag_node.children {
        for child_node in child_nodes {
            match child_node {
                HsmlNode::Tag(tag_node) => {
                    html_content.push_str(&compile_tag_node(tag_node, _options)?)
                }
                HsmlNode::Comment(comment_node) => {
                    if !comment_node.is_dev {
                        html_content.push_str(&compile_comment_node(comment_node, _options))
                    }
                }
                other => {
                    return Err(format!(
                        "Unsupported child node type in <{}>: {other:?}",
                        tag_node.tag
                    ));
                }
            }
        }
    }

    html_content.push_str("</");
    html_content.push_str(&tag_node.tag);
    html_content.push('>');

    Ok(html_content)
}

fn compile_comment_node(comment_node: &CommentNode, _options: &HsmlCompileOptions) -> String {
    let mut html_content = String::new();

    html_content.push_str("<!--");
    html_content.push_str(&comment_node.text);
    html_content.push_str(" -->");

    html_content
}

fn compile_node(node: &HsmlNode, options: &HsmlCompileOptions) -> Result<String, String> {
    match node {
        HsmlNode::Tag(tag_node) => compile_tag_node(tag_node, options),
        HsmlNode::Comment(comment_node) if !comment_node.is_dev => {
            Ok(compile_comment_node(comment_node, options))
        }
        HsmlNode::Comment(_) => Ok(String::from("")),
        other => Err(format!("Unsupported root node type: {other:?}")),
    }
}

pub fn compile(hsml_ast: &RootNode, options: &HsmlCompileOptions) -> Result<String, String> {
    let mut html_content = String::new();

    for node in &hsml_ast.nodes {
        html_content.push_str(&compile_node(node, options)?);
    }

    Ok(html_content)
}
