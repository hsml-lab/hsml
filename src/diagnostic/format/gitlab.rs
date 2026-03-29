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

/// Generate a stable fingerprint for a diagnostic using FNV-1a.
/// This hash is deterministic across Rust versions and platforms.
fn fingerprint(diag: &Diagnostic) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in diag.message.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    if let Some(ref code) = diag.code {
        for byte in code.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    if let Some(ref path) = diag.file_path {
        for byte in path.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    if let Some(ref loc) = diag.location {
        for byte in loc.start.line.to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        for byte in loc.start.column.to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    format!("{hash:016x}")
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

                // Only include location when both path and line info are available
                if let (Some(path), Some(loc)) = (&diag.file_path, &diag.location) {
                    issue["location"] = serde_json::json!({
                        "path": path,
                        "lines": { "begin": loc.start.line },
                    });
                }

                issue
            })
            .collect();

        serde_json::to_string(&issues).unwrap_or_else(|_| "[]".to_string())
    }
}
