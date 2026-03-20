pub mod compiler;
pub mod parser;

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

#[wasm_bindgen(js_name = "compileContent")]
pub fn compile_content(source: &str) -> Result<String, JsError> {
    compile_content_core(source).map_err(|e| JsError::new(&e))
}
