use crate::parser::{
    HsmlNode, RootNode,
    angular::node::{AngularNode, DefaultBranch},
    tag::node::TagNode,
};

/// A single item inside an attribute list, either an attribute or a comment.
enum AttrItem {
    Attr { text: String, end_line: u32 },
    Comment { text: String, start_line: u32 },
}

/// Options for the HSML formatter.
pub struct FormatOptions {
    /// Number of spaces per indentation level.
    pub indent_size: usize,
    /// Maximum line width before wrapping attributes.
    pub print_width: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_size: 2,
            print_width: 80,
        }
    }
}

/// Get the start line of a node from its location, if available.
fn node_start_line(node: &HsmlNode) -> Option<u32> {
    match node {
        HsmlNode::Tag(tag) => Some(tag.location.start.line),
        HsmlNode::Comment(comment) => Some(comment.location.start.line),
        HsmlNode::Angular(angular) => Some(angular.location().start.line),
        _ => None,
    }
}

/// Get the last source line a node occupies (approximate).
fn node_end_line(node: &HsmlNode) -> Option<u32> {
    match node {
        HsmlNode::Tag(tag) => {
            let mut last = tag.location.start.line;
            if let Some(children) = &tag.children
                && let Some(child) = children.last()
                && let Some(child_end) = node_end_line(child)
            {
                last = last.max(child_end);
            }
            if let Some(text) = &tag.text
                && text.is_block
            {
                last += text.text.lines().count() as u32;
            }
            if let Some(attrs) = &tag.attributes
                && let Some(last_attr) = attrs.last()
            {
                let attr_end = match last_attr {
                    HsmlNode::Attribute(a) => a.location.end.line,
                    HsmlNode::Comment(c) => c.location.end.line,
                    _ => 0,
                };
                if attr_end > last {
                    // +1 for the closing ')' on its own line
                    last = attr_end + 1;
                }
            }
            Some(last)
        }
        HsmlNode::Comment(comment) => Some(comment.location.end.line),
        HsmlNode::Angular(angular) => Some(angular.location().end.line),
        _ => None,
    }
}

/// Format an HSML AST back into source text.
pub fn format(ast: &RootNode, options: &FormatOptions) -> String {
    let mut output = String::new();

    format_nodes(&ast.nodes, 0, options, &mut output);

    // Collapse consecutive blank lines (including indentation-only lines)
    let lines: Vec<&str> = output.lines().collect();
    let mut collapsed = String::new();
    let mut prev_blank = false;
    for line in &lines {
        let is_blank = line.trim().is_empty();
        if is_blank && prev_blank {
            continue;
        }
        if !collapsed.is_empty() {
            collapsed.push('\n');
        }
        if !is_blank {
            collapsed.push_str(line);
        }
        prev_blank = is_blank;
    }
    output = collapsed;

    // Ensure trailing newline
    if !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

fn format_nodes(nodes: &[HsmlNode], depth: usize, options: &FormatOptions, output: &mut String) {
    for (i, node) in nodes.iter().enumerate() {
        // Insert a blank line if there was one between this node and the previous
        if i > 0
            && let (Some(prev_end), Some(current_start)) =
                (node_end_line(&nodes[i - 1]), node_start_line(node))
            && current_start > prev_end + 1
        {
            output.push('\n');
        }

        format_node(node, depth, options, output);
    }
}

fn format_node(node: &HsmlNode, depth: usize, options: &FormatOptions, output: &mut String) {
    match node {
        HsmlNode::Tag(tag) => format_tag(tag, depth, options, output),
        HsmlNode::Comment(comment) => {
            let indent = " ".repeat(depth * options.indent_size);
            if comment.is_dev {
                output.push_str(&format!("{indent}//{}\n", comment.text));
            } else {
                output.push_str(&format!("{indent}//!{}\n", comment.text));
            }
        }
        HsmlNode::Doctype(doctype) => {
            output.push_str(&format!("doctype {}\n", doctype.doctype));
        }
        HsmlNode::Angular(angular) => format_angular_node(angular, depth, options, output),
        _ => {}
    }
}

fn format_angular_node(
    node: &AngularNode,
    depth: usize,
    options: &FormatOptions,
    output: &mut String,
) {
    let indent = " ".repeat(depth * options.indent_size);
    match node {
        AngularNode::Let(let_node) => {
            output.push_str(&format!(
                "{indent}@let {} = {};\n",
                let_node.name, let_node.expression
            ));
        }
        AngularNode::If(if_node) => {
            format_block_head(
                output,
                &indent,
                &format!("@if ({})", if_node.condition),
                &if_node.then_branch,
                depth,
                options,
            );
            for branch in &if_node.else_if_branches {
                format_block_head(
                    output,
                    &indent,
                    &format!("@else if ({})", branch.condition),
                    &branch.body,
                    depth,
                    options,
                );
            }
            if let Some(else_body) = &if_node.else_branch {
                format_block_head(output, &indent, "@else", else_body, depth, options);
            }
        }
        AngularNode::For(for_node) => {
            format_block_head(
                output,
                &indent,
                &format!("@for ({})", for_node.expression),
                &for_node.body,
                depth,
                options,
            );
            if let Some(empty_body) = &for_node.empty_branch {
                format_block_head(output, &indent, "@empty", empty_body, depth, options);
            }
        }
        AngularNode::Switch(switch_node) => {
            output.push_str(&format!("{indent}@switch ({})\n", switch_node.expression));
            let case_depth = depth + 1;
            let case_indent = " ".repeat(case_depth * options.indent_size);
            for case in &switch_node.cases {
                // Stacked bare cases render on their own lines; the last carries the body.
                let (last, leading) = case
                    .values
                    .split_last()
                    .expect("a @case always has at least one value");
                for value in leading {
                    output.push_str(&format!("{case_indent}@case ({value})\n"));
                }
                format_block_head(
                    output,
                    &case_indent,
                    &format!("@case ({last})"),
                    &case.body,
                    case_depth,
                    options,
                );
            }
            if let Some(default) = &switch_node.default {
                match default {
                    DefaultBranch::Block(body) => format_block_head(
                        output,
                        &case_indent,
                        "@default",
                        body,
                        case_depth,
                        options,
                    ),
                    DefaultBranch::Never(None) => {
                        output.push_str(&format!("{case_indent}@default never;\n"))
                    }
                    DefaultBranch::Never(Some(expression)) => {
                        output.push_str(&format!("{case_indent}@default never({expression});\n"))
                    }
                }
            }
        }
        AngularNode::Defer(defer_node) => {
            let head = match &defer_node.triggers {
                Some(triggers) => format!("@defer ({triggers})"),
                None => "@defer".to_string(),
            };
            format_block_head(output, &indent, &head, &defer_node.body, depth, options);

            if let Some(placeholder) = &defer_node.placeholder {
                let head = match &placeholder.params {
                    Some(params) => format!("@placeholder ({params})"),
                    None => "@placeholder".to_string(),
                };
                format_block_head(output, &indent, &head, &placeholder.body, depth, options);
            }
            if let Some(loading) = &defer_node.loading {
                let head = match &loading.params {
                    Some(params) => format!("@loading ({params})"),
                    None => "@loading".to_string(),
                };
                format_block_head(output, &indent, &head, &loading.body, depth, options);
            }
            if let Some(error_body) = &defer_node.error {
                format_block_head(output, &indent, "@error", error_body, depth, options);
            }
        }
        AngularNode::Boundary(boundary_node) => {
            format_block_head(
                output,
                &indent,
                "@boundary",
                &boundary_node.body,
                depth,
                options,
            );
            if let Some(catch) = &boundary_node.catch {
                let head = match &catch.binding {
                    Some(binding) => format!("@catch ({binding})"),
                    None => "@catch".to_string(),
                };
                format_block_head(output, &indent, &head, &catch.body, depth, options);
            }
        }
    }
}

/// Format a control-flow block head and its body: `{head}` followed by either
/// ` {}` for an empty body or an indented child block.
fn format_block_head(
    output: &mut String,
    indent: &str,
    head: &str,
    body: &[HsmlNode],
    depth: usize,
    options: &FormatOptions,
) {
    output.push_str(indent);
    output.push_str(head);
    if body.is_empty() {
        output.push_str(" {}\n");
    } else {
        output.push('\n');
        format_nodes(body, depth + 1, options, output);
    }
}

fn format_tag(tag: &TagNode, depth: usize, options: &FormatOptions, output: &mut String) {
    let indent = " ".repeat(depth * options.indent_size);

    // Build the tag line: tag#id.class.class(attrs) text
    let mut line = String::new();

    // Tag name (omit "div" when there are classes or ids — implicit div)
    let has_selectors = !tag.ids.is_empty() || tag.classes.as_ref().is_some_and(|c| !c.is_empty());
    if tag.tag != "div" || !has_selectors {
        line.push_str(&tag.tag);
    }

    // IDs
    for id in &tag.ids {
        line.push('#');
        line.push_str(&id.id);
    }

    // Classes
    if let Some(classes) = &tag.classes {
        for class in classes {
            line.push('.');
            line.push_str(&class.name);
        }
    }

    // Attributes
    if let Some(attributes) = &tag.attributes {
        let items: Vec<AttrItem> = attributes
            .iter()
            .filter_map(|node| match node {
                HsmlNode::Attribute(attr) => {
                    let s = if let Some(ref value) = attr.value {
                        format!("{}=\"{}\"", attr.key, value)
                    } else {
                        attr.key.clone()
                    };
                    Some(AttrItem::Attr {
                        text: s,
                        end_line: attr.location.end.line,
                    })
                }
                HsmlNode::Comment(comment) if comment.is_dev => {
                    let text = comment.text.trim_end_matches(',');
                    Some(AttrItem::Comment {
                        text: format!("//{text}"),
                        start_line: comment.location.start.line,
                    })
                }
                _ => None,
            })
            .collect();

        let has_comments = items.iter().any(|i| matches!(i, AttrItem::Comment { .. }));
        let attr_only: Vec<&str> = items
            .iter()
            .filter_map(|i| match i {
                AttrItem::Attr { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();

        if !items.is_empty() {
            // Try single-line only if there are no comments
            let single_line = format!("({})", attr_only.join(", "));
            let full_line_len = indent.len() + line.len() + single_line.len();

            if !has_comments && full_line_len <= options.print_width {
                line.push_str(&single_line);
            } else {
                // Multi-line attributes
                line.push_str("(\n");
                let attr_indent = " ".repeat((depth + 1) * options.indent_size);

                let mut attr_iter = items.iter().peekable();
                while let Some(item) = attr_iter.next() {
                    line.push_str(&attr_indent);
                    match item {
                        AttrItem::Attr { text: s, end_line } => {
                            line.push_str(s);
                            // Check if next item is a trailing comment on the same line
                            let is_trailing = matches!(
                                attr_iter.peek(),
                                Some(AttrItem::Comment { start_line, .. }) if *start_line == *end_line
                            );
                            let has_more_attrs = if is_trailing {
                                attr_iter
                                    .clone()
                                    .skip(1)
                                    .any(|i| matches!(i, AttrItem::Attr { .. }))
                            } else {
                                attr_iter
                                    .clone()
                                    .any(|i| matches!(i, AttrItem::Attr { .. }))
                            };
                            if has_more_attrs {
                                line.push(',');
                            }
                            if is_trailing
                                && let Some(AttrItem::Comment { text: c, .. }) = attr_iter.next()
                            {
                                line.push(' ');
                                line.push_str(c);
                            }
                        }
                        AttrItem::Comment { text: s, .. } => {
                            // Standalone comment (not trailing an attribute)
                            line.push_str(s);
                        }
                    }
                    line.push('\n');
                }

                let closing_indent = " ".repeat(depth * options.indent_size);
                line.push_str(&closing_indent);
                line.push(')');
            }
        }
    }

    // Text content
    if let Some(text) = &tag.text {
        let use_block =
            text.is_block || indent.len() + line.len() + 1 + text.text.len() > options.print_width;

        if use_block {
            write_text_block(output, &indent, &line, &text.text, depth, options);
        } else {
            output.push_str(&format!("{indent}{line} {}\n", text.text));
        }
    } else {
        output.push_str(&format!("{indent}{line}\n"));
    }

    // Children
    if let Some(children) = &tag.children {
        format_nodes(children, depth + 1, options, output);
    }
}

fn write_text_block(
    output: &mut String,
    indent: &str,
    line: &str,
    text: &str,
    depth: usize,
    options: &FormatOptions,
) {
    output.push_str(&format!("{indent}{line}.\n"));
    let text_indent = " ".repeat((depth + 1) * options.indent_size);
    let max_width = options.print_width.saturating_sub(text_indent.len());
    let wrapped = word_wrap(text, max_width);
    for wrapped_line in wrapped.lines() {
        output.push_str(&format!("{text_indent}{wrapped_line}\n"));
    }
}

/// Word-wrap each line of text to fit within a maximum width.
/// Preserves existing line breaks but wraps long lines at word boundaries.
fn word_wrap(text: &str, max_width: usize) -> String {
    let mut result = String::new();

    for (i, paragraph) in text.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        wrap_line(paragraph, max_width, &mut result);
    }

    result
}

fn wrap_line(text: &str, max_width: usize, output: &mut String) {
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return;
    }

    let mut current_line_len = 0;

    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            output.push_str(word);
            current_line_len = word.len();
        } else if current_line_len + 1 + word.len() > max_width {
            output.push('\n');
            output.push_str(word);
            current_line_len = word.len();
        } else {
            output.push(' ');
            output.push_str(word);
            current_line_len += 1 + word.len();
        }
    }
}

#[cfg(test)]
mod tests;
