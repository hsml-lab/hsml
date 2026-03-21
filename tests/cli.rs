#![cfg(not(target_arch = "wasm32"))]

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("hsml").unwrap()
}

// --- Compile single file ---

#[test]
fn compile_single_file_produces_html() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");
    let output = dir.path().join("test.html");

    fs::write(&input, "h1 Hello World\n").unwrap();

    cmd()
        .args(["compile", input.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("Compiled HTML written to"));

    let html = fs::read_to_string(&output).unwrap();
    assert_eq!(html, "<h1>Hello World</h1>");
}

#[test]
fn compile_single_file_with_custom_output() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");
    let output = dir.path().join("custom.html");

    fs::write(&input, "p Hello\n").unwrap();

    cmd()
        .args([
            "compile",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    let html = fs::read_to_string(&output).unwrap();
    assert_eq!(html, "<p>Hello</p>");
}

// --- Error cases ---

#[test]
fn compile_nonexistent_path_fails() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("definitely_missing.hsml");

    cmd()
        .arg("compile")
        .arg(&missing)
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Path must be a file or directory",
        ));
}

#[test]
fn compile_wrong_extension_fails() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.txt");

    fs::write(&input, "h1 Hello\n").unwrap();

    cmd()
        .args(["compile", input.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("File must have .hsml extension"));
}

#[test]
fn compile_invalid_hsml_content_fails() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("bad.hsml");

    fs::write(&input, "@@@invalid\n").unwrap();

    cmd()
        .args(["compile", input.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("error: parse error"));
}

#[test]
fn compile_invalid_hsml_content_fails_with_json_format() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("bad.hsml");

    fs::write(&input, "@@@invalid\n").unwrap();

    cmd()
        .args([
            "compile",
            input.to_str().unwrap(),
            "--report-format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(r#""severity":"error""#))
        .stderr(predicates::str::contains(r#""message":"parse error""#));
}

#[test]
fn compile_duplicate_class_shows_warning() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("warn.hsml");

    fs::write(&input, "h1.foo.foo Hello\n").unwrap();

    cmd()
        .args(["compile", input.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicates::str::contains("warning[W002]"))
        .stderr(predicates::str::contains("Duplicate class 'foo'"));

    // HTML should still be produced
    let output = dir.path().join("warn.html");
    assert_eq!(
        fs::read_to_string(output).unwrap(),
        r#"<h1 class="foo foo">Hello</h1>"#
    );
}

// --- Directory compilation ---

#[test]
fn compile_directory_compiles_all_hsml_files() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.hsml"), "h1 A\n").unwrap();
    fs::write(dir.path().join("b.hsml"), "h2 B\n").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("a.html")).unwrap(),
        "<h1>A</h1>"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("b.html")).unwrap(),
        "<h2>B</h2>"
    );
}

#[test]
fn compile_directory_recurses_into_subdirs() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();

    fs::write(dir.path().join("root.hsml"), "div\n").unwrap();
    fs::write(sub.join("nested.hsml"), "span\n").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("root.html")).unwrap(),
        "<div/>"
    );
    assert_eq!(
        fs::read_to_string(sub.join("nested.html")).unwrap(),
        "<span/>"
    );
}

#[test]
fn compile_directory_reports_errors_from_invalid_files() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("good.hsml"), "h1 OK\n").unwrap();
    fs::write(dir.path().join("bad.hsml"), "@@@\n").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("error: parse error"));

    // good file should still have been compiled
    assert_eq!(
        fs::read_to_string(dir.path().join("good.html")).unwrap(),
        "<h1>OK</h1>"
    );
}

#[test]
fn compile_directory_skips_non_hsml_files() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.hsml"), "h1 A\n").unwrap();
    fs::write(dir.path().join("readme.txt"), "not hsml").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join("a.html").exists());
    assert!(!dir.path().join("readme.html").exists());
}

// --- Unimplemented commands ---

#[test]
fn parse_command_shows_not_implemented() {
    cmd()
        .args(["parse"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not yet implemented"));
}

#[test]
fn fmt_command_shows_not_implemented() {
    cmd()
        .args(["fmt"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not yet implemented"));
}

#[test]
fn check_command_shows_not_implemented() {
    cmd()
        .args(["check"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not yet implemented"));
}
