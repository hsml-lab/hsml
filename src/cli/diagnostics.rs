//! CLI-specific diagnostic output helpers.
//! These write directly to stderr and are not intended for use outside the CLI
//! (e.g., the LSP server uses its own protocol for diagnostics).

use std::time::Duration;

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
/// JSON format outputs a single aggregated array (empty `[]` when clean).
/// Default format renders per-file with source context.
pub fn render_diagnostics(results: &[FileDiagnostics], format: Option<&str>) {
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
pub fn has_errors(results: &[FileDiagnostics]) -> bool {
    results
        .iter()
        .flat_map(|r| r.diagnostics.iter())
        .any(|d| d.severity == Severity::Error)
}

/// Format a duration as a human-readable string (e.g. "123µs" or "4ms").
pub fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1000 {
        format!("{micros}µs")
    } else {
        format!("{}ms", duration.as_millis())
    }
}

/// Print a debug summary line with icon, file count, timing, and diagnostic counts.
pub fn print_summary(
    diagnostics: &[FileDiagnostics],
    file_count: usize,
    total_duration: Duration,
    dim: (&str, &str),
    no_color: bool,
    verb: &str,
) {
    let mut errors = 0;
    let mut warnings = 0;
    for fd in diagnostics {
        for d in &fd.diagnostics {
            match d.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
            }
        }
    }

    let timing = format_duration(total_duration);
    let files = if file_count == 1 {
        "1 file"
    } else {
        &format!("{file_count} files")
    };

    let icon = if no_color {
        if errors > 0 { "✗" } else { "✓" }
    } else if errors > 0 {
        "\x1b[31m✗\x1b[0m" // red
    } else {
        "\x1b[32m✓\x1b[0m" // green
    };

    let mut diag_parts = Vec::new();
    if errors > 0 {
        diag_parts.push(format!(
            "{errors} error{}",
            if errors == 1 { "" } else { "s" }
        ));
    }
    if warnings > 0 {
        diag_parts.push(format!(
            "{warnings} warning{}",
            if warnings == 1 { "" } else { "s" }
        ));
    }

    let summary = if diag_parts.is_empty() {
        format!("{files} {verb} in {timing}")
    } else {
        format!("{files} {verb} in {timing} ({})", diag_parts.join(", "))
    };

    println!("\n{icon} {}{summary}{}", dim.0, dim.1);
}
