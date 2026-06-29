use std::collections::HashSet;

use crate::common::{Location, Position, is_void_element};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parser::angular::node::{AngularNode, DefaultBranch};
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

fn valid_location(location: &Location) -> Option<Location> {
    if location.is_valid() {
        Some(location.clone())
    } else {
        None
    }
}

fn validate_node(node: &HsmlNode, diagnostics: &mut Vec<Diagnostic>) {
    match node {
        HsmlNode::Tag(tag) => validate_tag(tag, diagnostics),
        HsmlNode::Angular(angular) => validate_angular(angular, diagnostics),
        _ => {}
    }
}

fn validate_nodes(nodes: &[HsmlNode], diagnostics: &mut Vec<Diagnostic>) {
    for node in nodes {
        validate_node(node, diagnostics);
    }
}

/// Recurse into the bodies of an Angular `@`-block so tags nested in control
/// flow are validated just like top-level tags.
fn validate_angular(node: &AngularNode, diagnostics: &mut Vec<Diagnostic>) {
    match node {
        AngularNode::Let(_) => {}
        AngularNode::If(if_node) => {
            validate_nodes(&if_node.then_branch, diagnostics);
            for branch in &if_node.else_if_branches {
                validate_nodes(&branch.body, diagnostics);
            }
            if let Some(else_branch) = &if_node.else_branch {
                validate_nodes(else_branch, diagnostics);
            }
        }
        AngularNode::For(for_node) => {
            validate_nodes(&for_node.body, diagnostics);
            if let Some(empty_branch) = &for_node.empty_branch {
                validate_nodes(empty_branch, diagnostics);
            }
        }
        AngularNode::Switch(switch_node) => {
            for case in &switch_node.cases {
                validate_nodes(&case.body, diagnostics);
            }
            if let Some(DefaultBranch::Block(body)) = &switch_node.default {
                validate_nodes(body, diagnostics);
            }
        }
        AngularNode::Defer(defer_node) => {
            validate_nodes(&defer_node.body, diagnostics);
            if let Some(placeholder) = &defer_node.placeholder {
                validate_nodes(&placeholder.body, diagnostics);
            }
            if let Some(loading) = &defer_node.loading {
                validate_nodes(&loading.body, diagnostics);
            }
            if let Some(error_body) = &defer_node.error {
                validate_nodes(error_body, diagnostics);
            }
        }
        AngularNode::Boundary(boundary_node) => {
            validate_nodes(&boundary_node.body, diagnostics);
            if let Some(catch) = &boundary_node.catch {
                validate_nodes(&catch.body, diagnostics);
            }
        }
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
            location: valid_location(&tag.location),
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
                location: if tag.location.is_valid() {
                    Some(tag.location.clone())
                } else {
                    None
                },
                file_path: None,
            });
        }
    }

    // Check for duplicate ids (first wins, rest are warned)
    if tag.ids.len() > 1 {
        for id in &tag.ids[1..] {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: format!("Duplicate id '{}' is not allowed", id.id),
                code: Some(ErrorCode::DuplicateId.code().to_string()),
                location: valid_location(&id.location),
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
                    location: valid_location(&class.location),
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
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        message: format!("{} '{key}'", ErrorCode::DuplicateAttribute.message()),
                        code: Some(ErrorCode::DuplicateAttribute.code().to_string()),
                        location: valid_location(location),
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
