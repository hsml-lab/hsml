pub mod common;
pub mod compiler;
pub mod diagnostic;
pub mod parser;
pub mod validate;

use wasm_bindgen::prelude::*;

/// Core compile logic shared by WASM and native callers.
pub fn compile_content_core(source: &str) -> Result<String, String> {
    let span = parser::Span::new(source);
    let (rest, ast) = parser::parse::parse(span).map_err(|e| format!("HSML parse error: {e}"))?;

    if !rest.fragment().is_empty() {
        return Err(format!(
            "HSML parse error: unconsumed input at line {}, column {}",
            rest.location_line(),
            rest.get_column()
        ));
    }

    compiler::compile(&ast, &compiler::HsmlCompileOptions::default())
}

/// Result of compiling HSML source with diagnostic support.
#[derive(Debug, Clone, PartialEq)]
pub struct CompileOutput {
    /// The compiled HTML output.
    pub html: String,
    /// Warnings and other non-fatal diagnostics collected during validation.
    pub diagnostics: Vec<diagnostic::Diagnostic>,
}

/// Compile HSML source, returning structured diagnostics on error.
/// On success, returns the HTML output along with any warnings.
pub fn compile_content_diagnostics(
    source: &str,
) -> Result<CompileOutput, Vec<diagnostic::Diagnostic>> {
    let span = parser::Span::new(source);

    let (rest, ast) =
        parser::parse::parse(span).map_err(|e| vec![diagnostic::Diagnostic::from(&e)])?;

    if !rest.fragment().is_empty() {
        return Err(vec![diagnostic::Diagnostic {
            severity: diagnostic::Severity::Error,
            message: "Unconsumed input".to_string(),
            code: None,
            location: Some(diagnostic::Location {
                line: rest.location_line(),
                column: rest.get_column() as u32,
            }),
            file_path: None,
        }]);
    }

    // Run validation to collect warnings
    let diagnostics = validate::validate(&ast);

    let html = compiler::compile(&ast, &compiler::HsmlCompileOptions::default())
        .map_err(|e| vec![diagnostic::Diagnostic::compiler_error(e)])?;

    Ok(CompileOutput { html, diagnostics })
}

#[wasm_bindgen(js_name = "compileContent")]
pub fn compile_content(source: &str) -> Result<String, JsError> {
    compile_content_core(source).map_err(|e| JsError::new(&e))
}
