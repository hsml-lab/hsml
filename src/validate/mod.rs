use std::collections::HashSet;

use crate::common::Location;
use crate::diagnostic::{Diagnostic, Severity};
use crate::parser::error::ErrorCode;
use crate::parser::tag::node::TagNode;
use crate::parser::{HsmlNode, RootNode};

/// Validate an AST and source, returning any diagnostics (warnings, etc.).
/// This runs after parsing succeeds — diagnostics here don't prevent compilation.
pub fn validate(ast: &RootNode, source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    validate_mixed_indentation(source, &mut diagnostics);

    for node in &ast.nodes {
        validate_node(node, &mut diagnostics);
    }

    diagnostics
}

/// Check each line for mixed tabs and spaces in leading whitespace.
fn validate_mixed_indentation(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (line_idx, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let indent: &str = &line[..line.len() - line.trim_start().len()];

        if indent.contains('\t') && indent.contains(' ') {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: ErrorCode::MixedIndentation.message().to_string(),
                code: Some(ErrorCode::MixedIndentation.code().to_string()),
                location: Some(Location {
                    line: (line_idx + 1) as u32,
                    column: 1,
                }),
                file_path: None,
            });
        }
    }
}

fn validate_node(node: &HsmlNode, diagnostics: &mut Vec<Diagnostic>) {
    if let HsmlNode::Tag(tag) = node {
        validate_tag(tag, diagnostics);
    }
}

fn validate_tag(tag: &TagNode, diagnostics: &mut Vec<Diagnostic>) {
    // Check for duplicate ids (first wins, rest are warned)
    if tag.ids.len() > 1 {
        for id in &tag.ids[1..] {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: format!("Duplicate id '{}' is not allowed", id.id),
                code: Some(ErrorCode::DuplicateId.code().to_string()),
                location: if id.location.line > 0 && id.location.column > 0 {
                    Some(id.location.clone())
                } else {
                    None
                },
                file_path: None,
            });
        }
    }

    // Check for duplicate classes
    if let Some(classes) = &tag.classes {
        let mut seen: HashSet<&str> = HashSet::new();
        for class in classes {
            if seen.contains(class.name.as_str()) {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{} '{}'", ErrorCode::DuplicateClass.message(), class.name),
                    code: Some(ErrorCode::DuplicateClass.code().to_string()),
                    location: if class.location.line > 0 && class.location.column > 0 {
                        Some(class.location.clone())
                    } else {
                        None
                    },
                    file_path: None,
                });
            } else {
                seen.insert(&class.name);
            }
        }
    }

    // Recurse into children
    if let Some(children) = &tag.children {
        for child in children {
            validate_node(child, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests;
