#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn wasm_compile_content_returns_html() {
    let result = hsml::compile_content("h1 Hello\n");
    assert_eq!(result.unwrap(), "<h1>Hello</h1>");
}

#[wasm_bindgen_test]
fn wasm_compile_content_returns_error_for_invalid_input() {
    let result = hsml::compile_content("@@@invalid");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn wasm_compile_with_diagnostics_returns_object() {
    let result = hsml::compile_content_with_diagnostics("h1 Hello\n");
    assert!(result.is_truthy());
}

#[wasm_bindgen_test]
fn wasm_compile_with_diagnostics_returns_object_for_duplicate_class() {
    let result = hsml::compile_content_with_diagnostics("h1.foo.foo Hello\n");
    assert!(result.is_truthy());
}
