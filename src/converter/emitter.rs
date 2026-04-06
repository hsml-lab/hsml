use html5ever::tendril::StrTendril;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use crate::common::is_void_element;

const INDENT_SIZE: usize = 2;

/// Emit HSML source from an html5ever DOM.
pub fn emit(dom: &RcDom) -> String {
    let mut output = String::new();

    // html5ever wraps content in <html><head><body> for document parsing.
    // We need to find the meaningful content nodes.
    let nodes = find_content_nodes(&dom.document);

    for node in &nodes {
        emit_node(node, 0, &mut output);
    }

    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

/// Find the content nodes, skipping the implicit html/head/body wrapper
/// that html5ever adds for document parsing.
fn find_content_nodes(document: &Handle) -> Vec<Handle> {
    let children = document.children.borrow();

    // Check if this looks like a full document (has DOCTYPE or html element)
    let has_doctype = children
        .iter()
        .any(|c| matches!(c.data, NodeData::Doctype { .. }));

    let html_node = children.iter().find(
        |c| matches!(&c.data, NodeData::Element { name, .. } if name.local.as_ref() == "html"),
    );

    if has_doctype {
        // Full document — emit doctype + html element
        children.iter().cloned().collect()
    } else if let Some(html) = html_node {
        // html5ever wrapped a fragment in <html><head><body>
        // Collect document-level comments + head elements + body children
        let mut nodes: Vec<Handle> = Vec::new();

        // Collect document-level comments
        for child in children.iter() {
            if matches!(child.data, NodeData::Comment { .. }) {
                nodes.push(child.clone());
            }
        }

        let html_children = html.children.borrow();
        let head = html_children.iter().find(
            |c| matches!(&c.data, NodeData::Element { name, .. } if name.local.as_ref() == "head"),
        );
        let body = html_children.iter().find(
            |c| matches!(&c.data, NodeData::Element { name, .. } if name.local.as_ref() == "body"),
        );

        // Head may contain elements like <template>
        if let Some(head) = head {
            for child in head.children.borrow().iter() {
                if matches!(
                    child.data,
                    NodeData::Element { .. } | NodeData::Comment { .. }
                ) {
                    nodes.push(child.clone());
                }
            }
        }

        if let Some(body) = body {
            nodes.extend(body.children.borrow().iter().cloned());
        }

        nodes
    } else {
        children.iter().cloned().collect()
    }
}

/// Get the effective children of a node (handles template_contents).
fn effective_children(node: &Handle) -> Vec<Handle> {
    if let NodeData::Element {
        template_contents, ..
    } = &node.data
        && let Some(tmpl) = template_contents.borrow().as_ref()
    {
        return tmpl.children.borrow().iter().cloned().collect();
    }
    node.children.borrow().iter().cloned().collect()
}

/// Check if an element has mixed content (text interleaved with element children).
fn has_mixed_content(node: &Handle) -> bool {
    let children = effective_children(node);
    let has_elements = children
        .iter()
        .any(|c| matches!(c.data, NodeData::Element { .. }));
    let has_significant_text = children.iter().any(|c| {
        if let NodeData::Text { contents } = &c.data {
            !contents.borrow().trim().is_empty()
        } else {
            false
        }
    });
    has_elements && has_significant_text
}

/// Serialize a node's children back to raw HTML (for mixed content).
fn serialize_inner_html(node: &Handle) -> String {
    let mut html = String::new();
    for child in &effective_children(node) {
        serialize_node_to_html(child, &mut html);
    }
    html
}

/// Escape special HTML characters in text content.
fn escape_html_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape special HTML characters in attribute values (for HTML serialization).
fn escape_html_attr(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Check if an attribute key is a framework directive (Vue/Angular)
/// whose value contains a JavaScript expression rather than a plain HTML value.
fn is_framework_directive(key: &str) -> bool {
    key.starts_with(':')
        || key.starts_with('@')
        || key.starts_with("v-")
        || key.starts_with('#')
        || key.starts_with('[')
        || key.starts_with('(')
        || key.starts_with('*')
}

/// Escape an HSML attribute value.
/// Framework directives (Vue `:`, `@`, `v-`, `#`; Angular `[`, `(`, `*`)
/// contain JavaScript expressions where `&` is valid (e.g. `&&`).
/// Regular HTML attributes contain values where `&` must be `&amp;`
/// so the compiled HTML is valid.
fn escape_hsml_attr(key: &str, value: &str) -> String {
    if is_framework_directive(key) {
        value.replace('"', "&quot;")
    } else {
        // Encode & first, then " — order matters to avoid double-encoding
        value.replace('&', "&amp;").replace('"', "&quot;")
    }
}

fn serialize_node_to_html(node: &Handle, output: &mut String) {
    match &node.data {
        NodeData::Text { contents } => {
            output.push_str(&escape_html_text(&contents.borrow()));
        }
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.as_ref();
            output.push('<');
            output.push_str(tag);
            for attr in attrs.borrow().iter() {
                output.push(' ');
                output.push_str(attr.name.local.as_ref());
                output.push_str("=\"");
                output.push_str(&escape_html_attr(&attr.value));
                output.push('"');
            }
            if is_void_element(tag) {
                output.push_str(" />");
            } else {
                output.push('>');
                for child in &effective_children(node) {
                    serialize_node_to_html(child, output);
                }
                output.push_str("</");
                output.push_str(tag);
                output.push('>');
            }
        }
        NodeData::Comment { contents } => {
            output.push_str("<!--");
            output.push_str(contents);
            output.push_str("-->");
        }
        _ => {}
    }
}

fn emit_node(node: &Handle, depth: usize, output: &mut String) {
    match &node.data {
        NodeData::Doctype { name, .. } => {
            output.push_str(&format!("doctype {name}\n"));
        }
        NodeData::Element { name, attrs, .. } => {
            emit_element(node, &name.local, &attrs.borrow(), depth, output);
        }
        NodeData::Comment { contents } => {
            let indent = " ".repeat(depth * INDENT_SIZE);
            let text = contents.trim();
            output.push_str(&format!("{indent}//! {text}\n"));
        }
        NodeData::Text { contents } => {
            // Standalone text nodes (not inside an element) — should not happen
            // in well-formed HTML, but handle gracefully
            let text = contents.borrow().to_string();
            if !text.trim().is_empty() {
                let indent = " ".repeat(depth * INDENT_SIZE);
                output.push_str(&format!("{indent}| {}\n", text.trim()));
            }
        }
        _ => {}
    }
}

fn emit_element(
    node: &Handle,
    tag: &str,
    attrs: &[html5ever::interface::Attribute],
    depth: usize,
    output: &mut String,
) {
    let indent = " ".repeat(depth * INDENT_SIZE);

    // Extract id and class from attributes
    let mut id: Option<&str> = None;
    let mut classes: Vec<&str> = Vec::new();
    let mut other_attrs: Vec<(&str, &StrTendril)> = Vec::new();

    for attr in attrs {
        let key = attr.name.local.as_ref();
        match key {
            "id" => id = Some(&attr.value),
            "class" => {
                classes.extend(attr.value.split_whitespace());
            }
            _ => {
                other_attrs.push((key, &attr.value));
            }
        }
    }

    // Build tag line
    let mut line = String::new();

    // Implicit div: omit "div" when there are selectors
    let is_implicit_div = tag == "div" && (id.is_some() || !classes.is_empty());
    if !is_implicit_div {
        line.push_str(tag);
    }

    // Append #id
    if let Some(id) = id {
        line.push('#');
        line.push_str(id);
    }

    // Append .classes
    for class in &classes {
        line.push('.');
        line.push_str(class);
    }

    // Append (attributes)
    if !other_attrs.is_empty() {
        line.push('(');
        for (i, (key, value)) in other_attrs.iter().enumerate() {
            if i > 0 {
                line.push_str(", ");
            }
            if value.is_empty() {
                // Boolean attribute
                line.push_str(key);
            } else {
                line.push_str(key);
                line.push_str("=\"");
                line.push_str(&escape_hsml_attr(key, value));
                line.push('"');
            }
        }
        line.push(')');
    }

    // Handle children
    let children = effective_children(node);
    let significant_children: Vec<&Handle> = children
        .iter()
        .filter(|c| match &c.data {
            NodeData::Text { contents } => !contents.borrow().trim().is_empty(),
            NodeData::Element { .. } | NodeData::Comment { .. } => true,
            _ => false,
        })
        .collect();

    if is_void_element(tag) {
        // Void elements: no children
        output.push_str(&format!("{indent}{line}\n"));
        return;
    }

    if significant_children.is_empty() {
        // No content
        output.push_str(&format!("{indent}{line}\n"));
        return;
    }

    // Check for mixed content
    if has_mixed_content(node) {
        let inner = serialize_inner_html(node);
        let trimmed = inner.trim();
        if trimmed.contains('\n') {
            // Multi-line mixed content → text block
            output.push_str(&format!("{indent}{line}.\n"));
            let text_indent = " ".repeat((depth + 1) * INDENT_SIZE);
            for text_line in trimmed.lines() {
                output.push_str(&format!("{text_indent}{text_line}\n"));
            }
        } else {
            // Single-line mixed content → inline text
            output.push_str(&format!("{indent}{line} {trimmed}\n"));
        }
        return;
    }

    // Check if only text content (no element children)
    let text_only = significant_children
        .iter()
        .all(|c| matches!(c.data, NodeData::Text { .. }));

    if text_only {
        let text: String = significant_children
            .iter()
            .filter_map(|c| {
                if let NodeData::Text { contents } = &c.data {
                    Some(contents.borrow().trim().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        if text.contains('\n') {
            // Multi-line text → text block
            output.push_str(&format!("{indent}{line}.\n"));
            let text_indent = " ".repeat((depth + 1) * INDENT_SIZE);
            for text_line in text.lines() {
                output.push_str(&format!("{text_indent}{text_line}\n"));
            }
        } else {
            // Single-line text → inline
            output.push_str(&format!("{indent}{line} {text}\n"));
        }
        return;
    }

    // Element children only — recurse
    output.push_str(&format!("{indent}{line}\n"));
    for child in &significant_children {
        emit_node(child, depth + 1, output);
    }
}
