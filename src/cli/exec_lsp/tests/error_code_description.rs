use crate::cli::exec_lsp::error_code_description;

#[test]
fn it_should_return_description_for_known_error_code() {
    assert_eq!(
        error_code_description("E001"),
        Some("Tag name must start with an ASCII letter")
    );
}

#[test]
fn it_should_return_description_for_known_warning_code() {
    assert_eq!(error_code_description("W002"), Some("Duplicate class"));
}

#[test]
fn it_should_return_none_for_unknown_code() {
    assert_eq!(error_code_description("X999"), None);
}
