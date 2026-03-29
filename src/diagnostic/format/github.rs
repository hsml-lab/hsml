use super::{Diagnostic, DiagnosticFormatter};
use crate::diagnostic::Severity;

/// GitHub Actions workflow command formatter.
///
/// Outputs diagnostics as `::error` and `::warning` commands that GitHub Actions
/// renders as annotations on pull requests and commits.
///
/// See: https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions
///
/// Output format:
/// ```text
/// ::warning file=example.hsml,line=1,col=7,endLine=1,endColumn=11,title=W002::Duplicate class 'foo'
/// ::error file=example.hsml,line=1,col=1,title=E001::Tag name must start with an ASCII letter
/// ```
pub struct GithubFormatter;

impl DiagnosticFormatter for GithubFormatter {
    fn format(&self, diagnostics: &[Diagnostic], _source: Option<&str>) -> String {
        let mut output = String::new();

        for diag in diagnostics {
            let severity = match diag.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };

            let mut params = Vec::new();

            if let Some(ref path) = diag.file_path {
                params.push(format!("file={path}"));
            }

            if let Some(ref loc) = diag.location {
                params.push(format!("line={}", loc.start.line));
                params.push(format!("col={}", loc.start.column));
                if loc.start != loc.end {
                    params.push(format!("endLine={}", loc.end.line));
                    params.push(format!("endColumn={}", loc.end.column));
                }
            }

            if let Some(ref code) = diag.code {
                params.push(format!("title={code}"));
            }

            let params_str = if params.is_empty() {
                String::new()
            } else {
                format!(" {}", params.join(","))
            };

            output.push_str(&format!("::{severity}{params_str}::{}\n", diag.message));
        }

        output
    }
}
