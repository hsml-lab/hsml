use crate::diagnostic::{Diagnostic, Location, Severity};
use crate::parser::error::ErrorCode;
use crate::parser::tag::node::TagNode;
use crate::parser::{HsmlNode, RootNode};

/// Validate an AST and return any diagnostics (warnings, etc.).
/// This runs after parsing succeeds — errors here don't prevent compilation.
pub fn validate(ast: &RootNode, source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for node in &ast.nodes {
        validate_node(node, source, &mut diagnostics);
    }

    diagnostics
}

fn validate_node(node: &HsmlNode, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    if let HsmlNode::Tag(tag) = node {
        validate_tag(tag, source, diagnostics);
    }
}

fn validate_tag(tag: &TagNode, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    // Check for duplicate classes
    if let Some(classes) = &tag.classes {
        let mut seen: Vec<&str> = Vec::new();
        for class in classes {
            if seen.contains(&class.name.as_str()) {
                // Find the location of the duplicate in the source
                let location = find_duplicate_class_location(source, &tag.tag, &class.name);

                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{} '{}'", ErrorCode::DuplicateClass.message(), class.name),
                    code: Some(ErrorCode::DuplicateClass.code().to_string()),
                    location,
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
            validate_node(child, source, diagnostics);
        }
    }
}

/// Try to find the source location of a duplicate class.
/// This is a best-effort search in the source text.
fn find_duplicate_class_location(source: &str, _tag: &str, class_name: &str) -> Option<Location> {
    let search = format!(".{class_name}");

    // Find the second occurrence of the class
    if let Some(first) = source.find(&search)
        && let Some(second_offset) = source[first + search.len()..].find(&search)
    {
        let abs_offset = first + search.len() + second_offset;
        // Convert byte offset to line/column
        let before = &source[..abs_offset];
        let line = before.matches('\n').count() as u32 + 1;
        let last_newline = before.rfind('\n').map_or(0, |i| i + 1);
        let column = (abs_offset - last_newline) as u32 + 1;

        return Some(Location { line, column });
    }

    None
}

#[cfg(test)]
mod tests;
