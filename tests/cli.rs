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

#[test]
fn compile_json_format_suppresses_status_messages() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "h1 Hello\n").unwrap();

    let output = cmd()
        .args([
            "compile",
            input.to_str().unwrap(),
            "--report-format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Compiling"),
        "JSON mode should not print status messages"
    );
    assert!(dir.path().join("test.html").exists());
}

#[test]
fn compile_directory_json_aggregates_all_diagnostics() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.hsml"), "h1.foo.foo A\n").unwrap();
    fs::write(dir.path().join("b.hsml"), "h2.bar.bar B\n").unwrap();

    let output = cmd()
        .args([
            "compile",
            dir.path().to_str().unwrap(),
            "--report-format",
            "json",
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    let parsed: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr should be a single valid JSON array");
    let arr = parsed.as_array().expect("should be an array");

    assert_eq!(arr.len(), 2, "should have one warning per file");
    assert!(arr.iter().all(|d| d["severity"] == "warning"));

    // HTML should still be produced
    assert!(dir.path().join("a.html").exists());
    assert!(dir.path().join("b.html").exists());
}

#[test]
fn compile_directory_json_mixes_errors_and_warnings() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("good.hsml"), "h1.foo.foo OK\n").unwrap();
    fs::write(dir.path().join("bad.hsml"), "@@@invalid\n").unwrap();

    let output = cmd()
        .args([
            "compile",
            dir.path().to_str().unwrap(),
            "--report-format",
            "json",
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    let parsed: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr should be a single valid JSON array");
    let arr = parsed.as_array().expect("should be an array");

    assert!(arr.len() >= 2, "should have at least error + warning");

    let has_error = arr.iter().any(|d| d["severity"] == "error");
    let has_warning = arr.iter().any(|d| d["severity"] == "warning");
    assert!(has_error, "should contain an error");
    assert!(has_warning, "should contain a warning");

    // Good file should still compile
    assert!(dir.path().join("good.html").exists());
    // Bad file should not produce HTML
    assert!(!dir.path().join("bad.html").exists());
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

// --- Check command ---

#[test]
fn check_valid_file_succeeds() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("valid.hsml");

    fs::write(&input, "h1 Hello\n").unwrap();

    cmd()
        .args(["check", input.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn check_invalid_file_fails() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("bad.hsml");

    fs::write(&input, "@@@invalid\n").unwrap();

    cmd()
        .args(["check", input.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("error: parse error"));
}

#[test]
fn check_file_with_warnings_succeeds() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("warn.hsml");

    fs::write(&input, "h1.foo.foo Hello\n").unwrap();

    cmd()
        .args(["check", input.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicates::str::contains("warning[W002]"));
}

#[test]
fn check_with_json_format() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("warn.hsml");

    fs::write(&input, "h1.foo.foo Hello\n").unwrap();

    cmd()
        .args(["check", input.to_str().unwrap(), "--report-format", "json"])
        .assert()
        .success()
        .stderr(predicates::str::contains(r#""severity":"warning""#));
}

#[test]
fn check_directory() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("good.hsml"), "h1 OK\n").unwrap();
    fs::write(dir.path().join("bad.hsml"), "@@@\n").unwrap();

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("error: parse error"));
}

#[test]
fn check_defaults_to_current_directory() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("test.hsml"), "h1 Hello\n").unwrap();

    // Run check without a path argument — should use current directory
    cmd()
        .args(["check"])
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn check_directory_json_aggregates_all_diagnostics() {
    let dir = TempDir::new().unwrap();

    // Two files with warnings
    fs::write(dir.path().join("a.hsml"), "h1.foo.foo Hello\n").unwrap();
    fs::write(dir.path().join("b.hsml"), "h2.bar.bar World\n").unwrap();

    let output = cmd()
        .args([
            "check",
            dir.path().to_str().unwrap(),
            "--report-format",
            "json",
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should be a single JSON array containing diagnostics from both files
    let parsed: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr should be a single valid JSON array");
    let arr = parsed.as_array().expect("should be an array");

    assert_eq!(arr.len(), 2, "should have one diagnostic per file");
    assert!(arr.iter().all(|d| d["severity"] == "warning"));
}

#[cfg(unix)]
#[test]
fn check_directory_skips_symlinks() {
    use std::os::unix::fs::symlink;

    let dir = TempDir::new().unwrap();

    // Create a valid hsml file
    fs::write(dir.path().join("good.hsml"), "h1 Hello\n").unwrap();

    // Create a circular symlink: sub -> parent
    let sub = dir.path().join("sub");
    symlink(dir.path(), &sub).unwrap();

    // Should succeed without infinite recursion
    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success();
}
