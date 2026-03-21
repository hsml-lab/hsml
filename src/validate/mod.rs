use crate::diagnostic::{Diagnostic, Location, Severity};
use crate::parser::error::ErrorCode;
use crate::parser::tag::node::TagNode;
use crate::parser::{HsmlNode, RootNode};

/// Validate an AST and return any diagnostics (warnings, etc.).
/// This runs after parsing succeeds — errors here don't prevent compilation.
pub fn validate(ast: &RootNode) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for node in &ast.nodes {
        validate_node(node, &mut diagnostics);
    }

    diagnostics
}

fn validate_node(node: &HsmlNode, diagnostics: &mut Vec<Diagnostic>) {
    if let HsmlNode::Tag(tag) = node {
        validate_tag(tag, diagnostics);
    }
}

fn validate_tag(tag: &TagNode, diagnostics: &mut Vec<Diagnostic>) {
    // Check for duplicate classes
    if let Some(classes) = &tag.classes {
        let mut seen: Vec<&str> = Vec::new();
        for class in classes {
            if seen.contains(&class.name.as_str()) {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{} '{}'", ErrorCode::DuplicateClass.message(), class.name),
                    code: Some(ErrorCode::DuplicateClass.code().to_string()),
                    location: Some(Location {
                        line: class.line,
                        column: class.column,
                    }),
                    file_path: None,
                });
            } else {
                seen.push(&class.name);
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
