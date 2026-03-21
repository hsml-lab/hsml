pub mod default;
pub mod json;

#[cfg(test)]
mod tests;

use super::Diagnostic;

/// Trait for rendering diagnostics in a specific output format.
pub trait DiagnosticFormatter {
    /// Render diagnostics to a string.
    /// `source` is the original input text (used for source context display).
    fn format(&self, diagnostics: &[Diagnostic], source: Option<&str>) -> String;
}
