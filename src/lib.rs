pub mod compiler;
pub mod parser;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "compileContent")]
pub fn compile_content(source: &str) -> Result<String, JsError> {
    let (_, ast) = parser::parse::parse(source)
        .map_err(|e| JsError::new(&format!("HSML parse error: {e}")))?;

    Ok(compiler::compile(
        &ast,
        &compiler::HsmlCompileOptions::default(),
    ))
}
