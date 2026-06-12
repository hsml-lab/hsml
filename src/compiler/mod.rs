use crate::common::is_void_element;
use crate::parser::{
    HsmlNode, RootNode,
    angular::node::{AngularNode, DefaultBranch},
    attribute::node::AttributeNode,
    comment::node::CommentNode,
    doctype::node::DoctypeNode,
    tag::node::TagNode,
};

/// Options for configuring the HSML-to-HTML compiler.
pub struct HsmlCompileOptions {
    /// Emit pretty-printed HTML with indentation and newlines.
    pub pretty: bool,
    /// Number of spaces per indentation level (only used when `pretty` is true).
    pub indent_size: usize,
}

impl Default for HsmlCompileOptions {
    fn default() -> Self {
        Self {
            pretty: false,
            indent_size: 2,
        }
    }
}

fn compile_tag_node(
    tag_node: &TagNode,
    options: &HsmlCompileOptions,
    depth: usize,
) -> Result<String, String> {
    let mut html_content = String::new();
    let indent = if options.pretty {
        " ".repeat(depth * options.indent_size)
    } else {
        String::new()
    };

    html_content.push_str(&indent);
    html_content.push('<');
    html_content.push_str(&tag_node.tag);

    // Use the first id (duplicates are warned about by the validator)
    if let Some(id_node) = tag_node.ids.first() {
        html_content.push_str(r#" id=""#);
        html_content.push_str(&id_node.id);
        html_content.push('\"');
    }

    // Collect shorthand classes
    let mut all_classes: Vec<&str> = Vec::new();
    if let Some(class_nodes) = &tag_node.classes {
        all_classes.extend(class_nodes.iter().map(|c| c.name.as_str()));
    }

    // Merge class attributes from (class="...") with shorthand classes
    if let Some(attributes) = &tag_node.attributes {
        for node in attributes {
            if let HsmlNode::Attribute(AttributeNode { key, value, .. }) = node
                && key == "class"
                && let Some(value) = value
            {
                all_classes.extend(value.split_whitespace());
            }
        }
    }

    // Write merged class attribute
    if !all_classes.is_empty() {
        html_content.push_str(r#" class=""#);
        html_content.push_str(&all_classes.join(" "));
        html_content.push('\"');
    }

    // Write remaining attributes (skip class — already merged)
    if let Some(attributes) = &tag_node.attributes {
        for node in attributes {
            match node {
                HsmlNode::Attribute(AttributeNode { key, value, .. }) if key != "class" => {
                    html_content.push(' ');
                    html_content.push_str(key);

                    if let Some(value) = value {
                        html_content.push_str(r#"=""#);
                        html_content.push_str(value);
                        html_content.push('\"');
                    }
                }
                HsmlNode::Attribute(AttributeNode { key, .. }) if key == "class" => {
                    // Already merged above
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

    if is_void_element(&tag_node.tag) {
        html_content.push_str(" />");
        if options.pretty {
            html_content.push('\n');
        }
        return Ok(html_content);
    }

    html_content.push('>');

    if let Some(text) = &tag_node.text {
        html_content.push_str(&text.text);
    }

    if let Some(child_nodes) = &tag_node.children {
        if options.pretty {
            html_content.push('\n');
        }
        for child_node in child_nodes {
            match child_node {
                HsmlNode::Tag(tag_node) => {
                    html_content.push_str(&compile_tag_node(tag_node, options, depth + 1)?)
                }
                HsmlNode::Angular(angular_node) => {
                    html_content.push_str(&compile_angular_node(angular_node, options, depth + 1)?)
                }
                HsmlNode::Comment(comment_node) => {
                    if !comment_node.is_dev {
                        html_content.push_str(&compile_comment_node(
                            comment_node,
                            options,
                            depth + 1,
                        ))
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
        if options.pretty {
            html_content.push_str(&indent);
        }
    }

    html_content.push_str("</");
    html_content.push_str(&tag_node.tag);
    html_content.push('>');
    if options.pretty {
        html_content.push('\n');
    }

    Ok(html_content)
}

fn compile_angular_node(
    angular_node: &AngularNode,
    options: &HsmlCompileOptions,
    depth: usize,
) -> Result<String, String> {
    let mut html_content = String::new();
    let indent = if options.pretty {
        " ".repeat(depth * options.indent_size)
    } else {
        String::new()
    };

    match angular_node {
        // The expression is emitted verbatim (including any internal line breaks).
        AngularNode::Let(let_node) => {
            html_content.push_str(&indent);
            html_content.push_str("@let ");
            html_content.push_str(&let_node.name);
            html_content.push_str(" = ");
            html_content.push_str(&let_node.expression);
            html_content.push(';');
            if options.pretty {
                html_content.push('\n');
            }
        }
        AngularNode::If(if_node) => {
            html_content.push_str(&indent);
            html_content.push_str("@if (");
            html_content.push_str(&if_node.condition);
            html_content.push(')');
            html_content.push_str(&compile_block_body(&if_node.then_branch, options, depth)?);

            for branch in &if_node.else_if_branches {
                html_content.push_str(" @else if (");
                html_content.push_str(&branch.condition);
                html_content.push(')');
                html_content.push_str(&compile_block_body(&branch.body, options, depth)?);
            }

            if let Some(else_body) = &if_node.else_branch {
                html_content.push_str(" @else");
                html_content.push_str(&compile_block_body(else_body, options, depth)?);
            }

            if options.pretty {
                html_content.push('\n');
            }
        }
        AngularNode::For(for_node) => {
            html_content.push_str(&indent);
            html_content.push_str("@for (");
            html_content.push_str(&for_node.expression);
            html_content.push(')');
            html_content.push_str(&compile_block_body(&for_node.body, options, depth)?);

            if let Some(empty_body) = &for_node.empty_branch {
                html_content.push_str(" @empty");
                html_content.push_str(&compile_block_body(empty_body, options, depth)?);
            }

            if options.pretty {
                html_content.push('\n');
            }
        }
        AngularNode::Switch(switch_node) => {
            html_content.push_str(&indent);
            html_content.push_str("@switch (");
            html_content.push_str(&switch_node.expression);
            html_content.push_str(") {");
            if options.pretty {
                html_content.push('\n');
            }

            let inner_indent = if options.pretty {
                " ".repeat((depth + 1) * options.indent_size)
            } else {
                String::new()
            };

            for case in &switch_node.cases {
                if options.pretty {
                    html_content.push_str(&inner_indent);
                } else {
                    html_content.push(' ');
                }
                for (i, value) in case.values.iter().enumerate() {
                    if i > 0 {
                        html_content.push(' ');
                    }
                    html_content.push_str("@case (");
                    html_content.push_str(value);
                    html_content.push(')');
                }
                html_content.push_str(&compile_block_body(&case.body, options, depth + 1)?);
                if options.pretty {
                    html_content.push('\n');
                }
            }

            if let Some(default) = &switch_node.default {
                if options.pretty {
                    html_content.push_str(&inner_indent);
                } else {
                    html_content.push(' ');
                }
                match default {
                    DefaultBranch::Block(body) => {
                        html_content.push_str("@default");
                        html_content.push_str(&compile_block_body(body, options, depth + 1)?);
                    }
                    DefaultBranch::Never(None) => html_content.push_str("@default never;"),
                    DefaultBranch::Never(Some(expression)) => {
                        html_content.push_str("@default never(");
                        html_content.push_str(expression);
                        html_content.push_str(");");
                    }
                }
                if options.pretty {
                    html_content.push('\n');
                }
            }

            if options.pretty {
                html_content.push_str(&indent);
            }
            html_content.push('}');
            if options.pretty {
                html_content.push('\n');
            }
        }
        AngularNode::Defer(defer_node) => {
            html_content.push_str(&indent);
            html_content.push_str("@defer");
            if let Some(triggers) = &defer_node.triggers {
                html_content.push_str(" (");
                html_content.push_str(triggers);
                html_content.push(')');
            }
            html_content.push_str(&compile_block_body(&defer_node.body, options, depth)?);

            if let Some(placeholder) = &defer_node.placeholder {
                html_content.push_str(" @placeholder");
                if let Some(params) = &placeholder.params {
                    html_content.push_str(" (");
                    html_content.push_str(params);
                    html_content.push(')');
                }
                html_content.push_str(&compile_block_body(&placeholder.body, options, depth)?);
            }

            if let Some(loading) = &defer_node.loading {
                html_content.push_str(" @loading");
                if let Some(params) = &loading.params {
                    html_content.push_str(" (");
                    html_content.push_str(params);
                    html_content.push(')');
                }
                html_content.push_str(&compile_block_body(&loading.body, options, depth)?);
            }

            if let Some(error_body) = &defer_node.error {
                html_content.push_str(" @error");
                html_content.push_str(&compile_block_body(error_body, options, depth)?);
            }

            if options.pretty {
                html_content.push('\n');
            }
        }
        AngularNode::Boundary(boundary_node) => {
            html_content.push_str(&indent);
            html_content.push_str("@boundary");
            html_content.push_str(&compile_block_body(&boundary_node.body, options, depth)?);

            if let Some(catch) = &boundary_node.catch {
                html_content.push_str(" @catch");
                if let Some(binding) = &catch.binding {
                    html_content.push_str(" (");
                    html_content.push_str(binding);
                    html_content.push(')');
                }
                html_content.push_str(&compile_block_body(&catch.body, options, depth)?);
            }

            if options.pretty {
                html_content.push('\n');
            }
        }
    }

    Ok(html_content)
}

/// Compile a block body (the `{ … }` following a control-flow head). An empty
/// body becomes `{}`.
fn compile_block_body(
    body: &[HsmlNode],
    options: &HsmlCompileOptions,
    depth: usize,
) -> Result<String, String> {
    let mut html_content = String::from(" {");
    if options.pretty {
        html_content.push('\n');
    }

    for child in body {
        html_content.push_str(&compile_node(child, options, depth + 1)?);
    }

    if options.pretty {
        html_content.push_str(&" ".repeat(depth * options.indent_size));
    }
    html_content.push('}');

    Ok(html_content)
}

fn compile_comment_node(
    comment_node: &CommentNode,
    options: &HsmlCompileOptions,
    depth: usize,
) -> String {
    let mut html_content = String::new();

    if options.pretty {
        html_content.push_str(&" ".repeat(depth * options.indent_size));
    }

    html_content.push_str("<!--");
    html_content.push_str(&comment_node.text);
    html_content.push_str(" -->");

    if options.pretty {
        html_content.push('\n');
    }

    html_content
}

fn compile_doctype_node(doctype_node: &DoctypeNode) -> String {
    format!("<!DOCTYPE {}>", doctype_node.doctype)
}

fn compile_node(
    node: &HsmlNode,
    options: &HsmlCompileOptions,
    depth: usize,
) -> Result<String, String> {
    match node {
        HsmlNode::Doctype(doctype_node) => {
            let mut s = compile_doctype_node(doctype_node);
            if options.pretty {
                s.push('\n');
            }
            Ok(s)
        }
        HsmlNode::Tag(tag_node) => compile_tag_node(tag_node, options, depth),
        HsmlNode::Angular(angular_node) => compile_angular_node(angular_node, options, depth),
        HsmlNode::Comment(comment_node) if !comment_node.is_dev => {
            Ok(compile_comment_node(comment_node, options, depth))
        }
        HsmlNode::Comment(_) => Ok(String::from("")),
        other => Err(format!("Unsupported root node type: {other:?}")),
    }
}

/// Compile an HSML AST into an HTML string.
///
/// Returns `Ok(html)` on success, or `Err(message)` if a node cannot be compiled.
pub fn compile(hsml_ast: &RootNode, options: &HsmlCompileOptions) -> Result<String, String> {
    let mut html_content = String::new();

    for node in &hsml_ast.nodes {
        html_content.push_str(&compile_node(node, options, 0)?);
    }

    Ok(html_content)
}
