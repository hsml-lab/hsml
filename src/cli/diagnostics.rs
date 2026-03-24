//! CLI-specific diagnostic output helpers.
//! These write directly to stderr and are not intended for use outside the CLI
//! (e.g., the LSP server uses its own protocol for diagnostics).

use hsml::diagnostic::{
    Diagnostic, Severity,
    format::{DiagnosticFormatter, default::DefaultFormatter, json::JsonFormatter},
};

/// Collected diagnostics from a single file with its source content.
pub struct FileDiagnostics {
    pub diagnostics: Vec<Diagnostic>,
    pub source: String,
}

/// Render collected diagnostics to stderr.
/// JSON format outputs a single aggregated array.
/// Default format renders per-file with source context.
pub fn render_diagnostics(results: &[&FileDiagnostics], format: Option<&str>) {
    let all_diagnostics: Vec<&Diagnostic> =
        results.iter().flat_map(|r| r.diagnostics.iter()).collect();

    match format {
        Some("json") => {
            let owned: Vec<_> = all_diagnostics.into_iter().cloned().collect();
            let output = JsonFormatter.format(&owned, None);
            eprintln!("{output}");
        }
        _ => {
            for result in results {
                if !result.diagnostics.is_empty() {
                    eprint!(
                        "{}",
                        DefaultFormatter.format(&result.diagnostics, Some(&result.source))
                    );
                }
            }
        }
    }
}

/// Check if any diagnostic is an error.
pub fn has_errors(results: &[&FileDiagnostics]) -> bool {
    results
        .iter()
        .flat_map(|r| r.diagnostics.iter())
        .any(|d| d.severity == Severity::Error)
}
