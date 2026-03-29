use std::collections::HashSet;

use crate::common::{Location, Position, is_void_element};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parser::attribute::node::AttributeNode;
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
            let line_num = (line_idx + 1) as u32;
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: ErrorCode::MixedIndentation.message().to_string(),
                code: Some(ErrorCode::MixedIndentation.code().to_string()),
                location: Some(Location {
                    start: Position {
                        line: line_num,
                        column: 1,
                    },
                    end: Position {
                        line: line_num,
                        column: indent.len() as u32 + 1,
                    },
                }),
                file_path: None,
            });
        }
    }
}

/// Attributes that may legitimately appear multiple times and be merged.
/// Only `class`, `:class` (Vue v-bind:class), and `:style` (Vue v-bind:style)
/// support merging. All other duplicates are likely mistakes.
fn is_mergeable_attribute(key: &str) -> bool {
    key == "class" || key == ":class" || key == ":style"
}

fn validate_node(node: &HsmlNode, diagnostics: &mut Vec<Diagnostic>) {
    if let HsmlNode::Tag(tag) = node {
        validate_tag(tag, diagnostics);
    }
}

fn validate_tag(tag: &TagNode, diagnostics: &mut Vec<Diagnostic>) {
    // Check for void elements with content
    if is_void_element(&tag.tag) && (tag.text.is_some() || tag.children.is_some()) {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            message: format!(
                "{} '<{}>'",
                ErrorCode::VoidElementContent.message(),
                tag.tag
            ),
            code: Some(ErrorCode::VoidElementContent.code().to_string()),
            location: None,
            file_path: None,
        });
    }

    // Check for empty attribute parentheses
    if let Some(attributes) = &tag.attributes {
        let has_real_attributes = attributes
            .iter()
            .any(|n| matches!(n, HsmlNode::Attribute(_)));
        if !has_real_attributes {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: ErrorCode::EmptyAttributes.message().to_string(),
                code: Some(ErrorCode::EmptyAttributes.code().to_string()),
                location: None,
                file_path: None,
            });
        }
    }

    // Check for duplicate ids (first wins, rest are warned)
    if tag.ids.len() > 1 {
        for id in &tag.ids[1..] {
            let has_location = id.location.is_valid();
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: format!("Duplicate id '{}' is not allowed", id.id),
                code: Some(ErrorCode::DuplicateId.code().to_string()),
                location: if has_location {
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
                let has_location = class.location.is_valid();
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{} '{}'", ErrorCode::DuplicateClass.message(), class.name),
                    code: Some(ErrorCode::DuplicateClass.code().to_string()),
                    location: if has_location {
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

    // Check for duplicate attributes (skip class, data-*, and framework bindings)
    if let Some(attributes) = &tag.attributes {
        let mut seen: HashSet<&str> = HashSet::new();
        for node in attributes {
            if let HsmlNode::Attribute(AttributeNode { key, location, .. }) = node {
                if is_mergeable_attribute(key) {
                    continue;
                }
                if !seen.insert(key.as_str()) {
                    let has_location = location.is_valid();
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        message: format!("{} '{key}'", ErrorCode::DuplicateAttribute.message()),
                        code: Some(ErrorCode::DuplicateAttribute.code().to_string()),
                        location: if has_location {
                            Some(location.clone())
                        } else {
                            None
                        },
                        file_path: None,
                    });
                }
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
