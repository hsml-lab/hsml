use super::{Diagnostic, DiagnosticFormatter};
use crate::diagnostic::Severity;

/// GitLab Code Quality report formatter (CodeClimate format).
///
/// Outputs diagnostics as a JSON array compatible with GitLab's Code Quality widget.
/// Each diagnostic is a CodeClimate issue object with description, check_name,
/// fingerprint, severity, and location.
///
/// See: https://docs.gitlab.com/ci/testing/code_quality/
///
/// Output format:
/// ```json
/// [{"description":"Duplicate class 'foo'","check_name":"W002","fingerprint":"...","severity":"minor","location":{"path":"example.hsml","lines":{"begin":1}}}]
/// ```
pub struct GitlabFormatter;

/// Map hsml severity to CodeClimate severity.
fn codeclimate_severity(severity: &Severity) -> &'static str {
    match severity {
        Severity::Error => "major",
        Severity::Warning => "minor",
    }
}

/// Generate a fingerprint for a diagnostic.
/// Uses a simple hash of the key fields to produce a stable identifier.
fn fingerprint(diag: &Diagnostic) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    diag.message.hash(&mut hasher);
    diag.code.hash(&mut hasher);
    if let Some(ref path) = diag.file_path {
        path.hash(&mut hasher);
    }
    if let Some(ref loc) = diag.location {
        loc.start.line.hash(&mut hasher);
        loc.start.column.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

impl DiagnosticFormatter for GitlabFormatter {
    fn format(&self, diagnostics: &[Diagnostic], _source: Option<&str>) -> String {
        let issues: Vec<serde_json::Value> = diagnostics
            .iter()
            .map(|diag| {
                let mut issue = serde_json::json!({
                    "description": diag.message,
                    "check_name": diag.code.as_deref().unwrap_or("hsml"),
                    "fingerprint": fingerprint(diag),
                    "severity": codeclimate_severity(&diag.severity),
                });

                if let Some(ref path) = diag.file_path {
                    let mut location = serde_json::json!({
                        "path": path,
                    });
                    if let Some(ref loc) = diag.location {
                        location["lines"] = serde_json::json!({
                            "begin": loc.start.line,
                        });
                    }
                    issue["location"] = location;
                }

                issue
            })
            .collect();

        serde_json::to_string(&issues).unwrap_or_else(|_| "[]".to_string())
    }
}
