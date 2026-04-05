#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsValue;
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
fn wasm_compile_with_diagnostics_success() {
    let result = hsml::compile_content_with_diagnostics("h1 Hello\n");

    let success = js_sys::Reflect::get(&result, &JsValue::from_str("success")).unwrap();
    assert_eq!(success, JsValue::from_bool(true));

    let html = js_sys::Reflect::get(&result, &JsValue::from_str("html")).unwrap();
    assert_eq!(html, JsValue::from_str("<h1>Hello</h1>"));

    let diagnostics = js_sys::Reflect::get(&result, &JsValue::from_str("diagnostics")).unwrap();
    let arr = js_sys::Array::from(&diagnostics);
    assert_eq!(arr.length(), 0);
}

#[wasm_bindgen_test]
fn wasm_compile_with_diagnostics_warning() {
    let result = hsml::compile_content_with_diagnostics("h1.foo.foo Hello\n");

    let success = js_sys::Reflect::get(&result, &JsValue::from_str("success")).unwrap();
    assert_eq!(success, JsValue::from_bool(true));

    let html = js_sys::Reflect::get(&result, &JsValue::from_str("html")).unwrap();
    assert!(html.is_string(), "html should be a string");

    let diagnostics = js_sys::Reflect::get(&result, &JsValue::from_str("diagnostics")).unwrap();
    let arr = js_sys::Array::from(&diagnostics);
    assert_eq!(arr.length(), 1);

    let diag = arr.get(0);
    let severity = js_sys::Reflect::get(&diag, &JsValue::from_str("severity")).unwrap();
    assert_eq!(severity, JsValue::from_str("warning"));
}

#[wasm_bindgen_test]
fn wasm_compile_with_diagnostics_error() {
    let result = hsml::compile_content_with_diagnostics("@@@invalid");

    let success = js_sys::Reflect::get(&result, &JsValue::from_str("success")).unwrap();
    assert_eq!(success, JsValue::from_bool(false));

    let html = js_sys::Reflect::get(&result, &JsValue::from_str("html")).unwrap();
    assert!(html.is_null(), "html should be null on error");

    let diagnostics = js_sys::Reflect::get(&result, &JsValue::from_str("diagnostics")).unwrap();
    let arr = js_sys::Array::from(&diagnostics);
    assert!(arr.length() > 0, "should have at least one diagnostic");

    let diag = arr.get(0);
    let severity = js_sys::Reflect::get(&diag, &JsValue::from_str("severity")).unwrap();
    assert_eq!(severity, JsValue::from_str("error"));
}

#[wasm_bindgen_test]
fn wasm_format_content_returns_formatted() {
    let result = hsml::format_content("div\n    h1 Hello\n", JsValue::UNDEFINED);
    assert_eq!(result.unwrap(), "div\n  h1 Hello\n");
}

#[wasm_bindgen_test]
fn wasm_format_content_returns_error_for_invalid_input() {
    let result = hsml::format_content("@@@invalid", JsValue::UNDEFINED);
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn wasm_format_content_respects_custom_indent_size() {
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
        &options,
        &JsValue::from_str("indentSize"),
        &JsValue::from(4),
    )
    .unwrap();

    let result = hsml::format_content("div\n  h1 Hello\n", options.into());
    assert_eq!(result.unwrap(), "div\n    h1 Hello\n");
}
