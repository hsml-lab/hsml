use std::collections::HashMap;

use html5ever::tendril::StrTendril;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use crate::common::is_void_element;

const INDENT_SIZE: usize = 2;

/// Build a mapping from lowercased tag names to their original PascalCase form.
/// html5ever lowercases all tags per HTML5 spec, but Vue/Angular use PascalCase
/// components that we want to preserve.
fn build_case_map(original_html: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let bytes = original_html.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            // Found a tag opening — extract the tag name
            let start = i + 1;
            let mut end = start;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-' || bytes[end] == b'_')
            {
                end += 1;
            }
            let tag = &original_html[start..end];
            // Only store if it has uppercase characters (PascalCase)
            if tag.chars().any(|c| c.is_ascii_uppercase()) {
                let lower = tag.to_ascii_lowercase();
                map.entry(lower).or_insert_with(|| tag.to_string());
            }
            i = end;
        } else {
            i += 1;
        }
    }

    map
}

/// Resolve a tag name to its original casing if it was PascalCase in the source.
fn resolve_tag_case<'a>(tag: &'a str, case_map: &'a HashMap<String, String>) -> &'a str {
    case_map.get(tag).map(|s| s.as_str()).unwrap_or(tag)
}

/// Emit HSML source from an html5ever DOM.
pub fn emit(dom: &RcDom, original_html: &str) -> String {
    let mut output = String::new();
    let lower_html = original_html.to_ascii_lowercase();
    let case_map = build_case_map(original_html);

    // html5ever wraps content in <html><head><body> for document parsing.
    // We need to find the meaningful content nodes.
    let nodes = find_content_nodes(&dom.document, &lower_html);

    for node in &nodes {
        emit_node(node, 0, &lower_html, &case_map, &mut output);
    }

    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

/// Find the content nodes, skipping the implicit html/head/body wrapper
/// that html5ever adds for document parsing.
fn find_content_nodes(document: &Handle, lower_html: &str) -> Vec<Handle> {
    let children = document.children.borrow();

    // Check if this looks like a full document (has DOCTYPE or html element)
    let has_doctype = children
        .iter()
        .any(|c| matches!(c.data, NodeData::Doctype { .. }));

    let html_node = children.iter().find(
        |c| matches!(&c.data, NodeData::Element { name, .. } if name.local.as_ref() == "html"),
    );

    // Check if the user explicitly wrote <html> or <body> tags
    let has_explicit_html = lower_html.contains("<html") || lower_html.contains("<body");

    if has_doctype || has_explicit_html {
        // Full document or user-authored html/body — emit all children,
        // but skip empty elements synthesized by html5ever that the user didn't write
        children
            .iter()
            .filter(|c| !is_synthesized_empty_element(c, lower_html))
            .cloned()
            .collect()
    } else if let Some(html) = html_node {
        // html5ever synthesized <html><head><body> for a fragment
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

/// Check if a node is an empty element synthesized by html5ever
/// that doesn't appear in the original HTML source.
fn is_synthesized_empty_element(node: &Handle, lower_html: &str) -> bool {
    if let NodeData::Element { name, .. } = &node.data {
        let tag = name.local.as_ref();
        let tag_in_source = lower_html.contains(&format!("<{tag}"));
        if !tag_in_source {
            let children = node.children.borrow();
            // Empty or only contains whitespace text nodes
            return children.iter().all(|c| {
                if let NodeData::Text { contents } = &c.data {
                    contents.borrow().trim().is_empty()
                } else {
                    false
                }
            });
        }
    }
    false
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

/// Check if an element has mixed content (text interleaved with element or comment children).
fn has_mixed_content(node: &Handle) -> bool {
    let children = effective_children(node);
    let has_non_text = children
        .iter()
        .any(|c| matches!(c.data, NodeData::Element { .. } | NodeData::Comment { .. }));
    let has_significant_text = children.iter().any(|c| {
        if let NodeData::Text { contents } = &c.data {
            !contents.borrow().trim().is_empty()
        } else {
            false
        }
    });
    has_non_text && has_significant_text
}

/// Serialize a node's children back to raw HTML (for mixed content).
fn serialize_inner_html(node: &Handle, case_map: &HashMap<String, String>) -> String {
    let mut html = String::new();
    for child in &effective_children(node) {
        serialize_node_to_html(child, &mut html, false, case_map);
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

/// Check if a value contains `#` or `.` outside of `[...]` brackets.
/// TailwindCSS uses `[#hex]` and `[...]` which are safe for shorthand syntax.
fn has_selector_chars_outside_brackets(value: &str) -> bool {
    let mut in_bracket = false;
    for ch in value.chars() {
        match ch {
            '[' => in_bracket = true,
            ']' => in_bracket = false,
            '#' | '.' if !in_bracket => return true,
            _ => {}
        }
    }
    false
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

fn serialize_node_to_html(
    node: &Handle,
    output: &mut String,
    in_raw_text: bool,
    case_map: &HashMap<String, String>,
) {
    match &node.data {
        NodeData::Text { contents } => {
            if in_raw_text {
                output.push_str(&contents.borrow());
            } else {
                output.push_str(&escape_html_text(&contents.borrow()));
            }
        }
        NodeData::Element { name, attrs, .. } => {
            let lower_tag = name.local.as_ref();
            let tag = resolve_tag_case(lower_tag, case_map);
            let is_raw = matches!(lower_tag, "script" | "style");
            output.push('<');
            output.push_str(tag);
            for attr in attrs.borrow().iter() {
                output.push(' ');
                output.push_str(attr.name.local.as_ref());
                output.push_str("=\"");
                output.push_str(&escape_html_attr(&attr.value));
                output.push('"');
            }
            if is_void_element(lower_tag) {
                output.push_str(" />");
            } else {
                output.push('>');
                for child in &effective_children(node) {
                    serialize_node_to_html(child, output, is_raw, case_map);
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

fn emit_node(
    node: &Handle,
    depth: usize,
    lower_html: &str,
    case_map: &HashMap<String, String>,
    output: &mut String,
) {
    match &node.data {
        NodeData::Doctype { name, .. } => {
            output.push_str(&format!("doctype {name}\n"));
        }
        NodeData::Element { name, attrs, .. } => {
            let tag = resolve_tag_case(&name.local, case_map);
            emit_element(
                node,
                tag,
                &attrs.borrow(),
                depth,
                lower_html,
                case_map,
                output,
            );
        }
        NodeData::Comment { contents } => {
            let indent = " ".repeat(depth * INDENT_SIZE);
            let text = contents.trim();
            for line in text.lines() {
                output.push_str(&format!("{indent}//! {line}\n"));
            }
        }
        NodeData::Text { .. } => {
            // Standalone text nodes outside elements are ignored.
            // Significant text inside elements is handled by emit_element.
        }
        _ => {}
    }
}

fn emit_element(
    node: &Handle,
    tag: &str,
    attrs: &[html5ever::interface::Attribute],
    depth: usize,
    lower_html: &str,
    case_map: &HashMap<String, String>,
    output: &mut String,
) {
    let indent = " ".repeat(depth * INDENT_SIZE);

    // Extract id and class from attributes
    let mut id: Option<&str> = None;
    let mut classes: Vec<&str> = Vec::new();
    let mut unsafe_class_attr: Option<String> = None;
    let mut other_attrs: Vec<(&str, &StrTendril)> = Vec::new();

    for attr in attrs {
        let key = attr.name.local.as_ref();
        match key {
            "id" => {
                let val = &*attr.value;
                if has_selector_chars_outside_brackets(val) {
                    other_attrs.push((key, &attr.value));
                } else {
                    id = Some(val);
                }
            }
            "class" => {
                let mut unsafe_classes: Vec<&str> = Vec::new();
                for class in attr.value.split_whitespace() {
                    if has_selector_chars_outside_brackets(class) {
                        unsafe_classes.push(class);
                    } else {
                        classes.push(class);
                    }
                }
                if !unsafe_classes.is_empty() {
                    unsafe_class_attr = Some(unsafe_classes.join(" "));
                }
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
    let has_attrs = !other_attrs.is_empty() || unsafe_class_attr.is_some();
    if has_attrs {
        line.push('(');
        let mut first = true;

        // Unsafe classes that couldn't use shorthand syntax
        if let Some(ref unsafe_classes) = unsafe_class_attr {
            line.push_str("class=\"");
            line.push_str(&escape_hsml_attr("class", unsafe_classes));
            line.push('"');
            first = false;
        }

        for (key, value) in other_attrs.iter() {
            if !first {
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
            first = false;
        }
        line.push(')');
    }

    // Handle children
    let children = effective_children(node);
    let whitespace_sensitive = matches!(tag, "pre" | "textarea" | "script" | "style");

    let significant_children: Vec<&Handle> = children
        .iter()
        .filter(|c| match &c.data {
            NodeData::Text { contents } => {
                whitespace_sensitive || !contents.borrow().trim().is_empty()
            }
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

    // Whitespace-sensitive tags: preserve content exactly as text block
    if whitespace_sensitive {
        let has_non_text = significant_children
            .iter()
            .any(|c| !matches!(c.data, NodeData::Text { .. }));

        let raw = if has_non_text {
            // Has nested elements/comments — serialize as raw HTML to preserve markup
            serialize_inner_html(node, case_map)
        } else {
            // Text-only — collect raw text content
            significant_children
                .iter()
                .filter_map(|c| {
                    if let NodeData::Text { contents } = &c.data {
                        Some(contents.borrow().to_string())
                    } else {
                        None
                    }
                })
                .collect()
        };

        output.push_str(&format!("{indent}{line}.\n"));
        let text_indent = " ".repeat((depth + 1) * INDENT_SIZE);
        for text_line in raw.lines() {
            output.push_str(&format!("{text_indent}{text_line}\n"));
        }
        return;
    }

    // Check for mixed content
    if has_mixed_content(node) {
        let inner = serialize_inner_html(node, case_map);
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
    for child in significant_children {
        if !is_synthesized_empty_element(child, lower_html) {
            emit_node(child, depth + 1, lower_html, case_map, output);
        }
    }
}
