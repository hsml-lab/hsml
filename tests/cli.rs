#![cfg(not(target_arch = "wasm32"))]

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

fn cmd() -> Command {
    let mut cmd = Command::cargo_bin("hsml").unwrap();
    cmd.env("NO_COLOR", "1");
    cmd
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
        .stdout("");

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

    fs::write(&input, "%%%invalid\n").unwrap();

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

    fs::write(&input, "%%%invalid\n").unwrap();

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
        "<div></div>"
    );
    assert_eq!(
        fs::read_to_string(sub.join("nested.html")).unwrap(),
        "<span></span>"
    );
}

#[test]
fn compile_directory_reports_errors_from_invalid_files() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("good.hsml"), "h1 OK\n").unwrap();
    fs::write(dir.path().join("bad.hsml"), "%%%\n").unwrap();

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
fn compile_defaults_to_current_directory() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("test.hsml"), "h1 Hello\n").unwrap();

    cmd()
        .args(["compile"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("test.html").exists());
}

#[cfg(unix)]
#[test]
fn compile_directory_skips_symlinks() {
    use std::os::unix::fs::symlink;

    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("good.hsml"), "h1 Hello\n").unwrap();

    // Create a circular symlink: sub -> parent
    let sub = dir.path().join("sub");
    symlink(dir.path(), &sub).unwrap();

    // Should succeed without infinite recursion
    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join("good.html").exists());
}

#[test]
fn compile_does_not_print_status_messages_by_default() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "h1 Hello\n").unwrap();

    let output = cmd()
        .args(["compile", input.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty(), "stdout should be empty by default");
}

#[test]
fn compile_debug_flag_prints_status_messages() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "h1 Hello\n").unwrap();

    let output = cmd()
        .args(["compile", "--debug", "--no-color", input.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ms") || stdout.contains("µs"),
        "--debug should print timing, got: {stdout}"
    );
    assert!(
        stdout.contains("test.html"),
        "--debug should print output filename, got: {stdout}"
    );
}

#[test]
fn compile_debug_flag_prints_directory_summary() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.hsml"), "h1 A\n").unwrap();
    fs::write(dir.path().join("b.hsml"), "h2 B\n").unwrap();

    let output = cmd()
        .args([
            "compile",
            "--debug",
            "--no-color",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Compiling 2 file(s)"),
        "--debug should print file count for directories, got: {stdout}"
    );
    assert!(
        stdout.contains("2 files compiled in"),
        "--debug should print summary, got: {stdout}"
    );
}

#[test]
fn compile_debug_summary_shows_checkmark_for_clean_run() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("test.hsml"), "h1 Hello\n").unwrap();

    let output = cmd()
        .args([
            "compile",
            "--debug",
            "--no-color",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓"),
        "clean compile should show checkmark, got: {stdout}"
    );
}

#[test]
fn compile_debug_summary_shows_warnings() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("warn.hsml"), "h1.foo.foo Hello\n").unwrap();

    let output = cmd()
        .args([
            "compile",
            "--debug",
            "--no-color",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓") && stdout.contains("1 warning"),
        "compile with warnings should show checkmark and warning count, got: {stdout}"
    );
}

#[test]
fn compile_debug_summary_shows_cross_for_errors() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("bad.hsml"), "%%%invalid\n").unwrap();

    let output = cmd()
        .args([
            "compile",
            "--debug",
            "--no-color",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✗") && stdout.contains("1 error"),
        "compile with errors should show cross and error count, got: {stdout}"
    );
}

#[test]
fn compile_pretty_produces_indented_html() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");
    let output = dir.path().join("test.html");

    fs::write(&input, "div\n  p Hello\n").unwrap();

    cmd()
        .args(["compile", "--pretty", input.to_str().unwrap()])
        .assert()
        .success();

    let html = fs::read_to_string(&output).unwrap();
    assert_eq!(html, "<div>\n  <p>Hello</p>\n</div>\n");
}

#[test]
fn compile_pretty_with_nested_structure() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");
    let output = dir.path().join("test.html");

    fs::write(&input, "doctype html\nhtml\n  head\n  body\n    p Hello\n").unwrap();

    cmd()
        .args(["compile", "--pretty", input.to_str().unwrap()])
        .assert()
        .success();

    let html = fs::read_to_string(&output).unwrap();
    assert_eq!(
        html,
        "<!DOCTYPE html>\n<html>\n  <head></head>\n  <body>\n    <p>Hello</p>\n  </body>\n</html>\n"
    );
}

#[test]
fn compile_without_pretty_produces_single_line() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");
    let output = dir.path().join("test.html");

    fs::write(&input, "div\n  p Hello\n").unwrap();

    cmd()
        .args(["compile", input.to_str().unwrap()])
        .assert()
        .success();

    let html = fs::read_to_string(&output).unwrap();
    assert_eq!(html, "<div><p>Hello</p></div>");
}

#[test]
fn compile_json_emits_empty_array_for_clean_run() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("clean.hsml");

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

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "[]");
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
    fs::write(dir.path().join("bad.hsml"), "%%%invalid\n").unwrap();

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
fn parse_outputs_ast_as_json() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "h1.title Hello World\n").unwrap();

    let output = cmd()
        .args(["parse", input.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("should be valid JSON");

    let nodes = parsed["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["type"], "tag");
    assert_eq!(nodes[0]["tag"], "h1");
    assert_eq!(nodes[0]["text"]["text"], "Hello World");
    assert_eq!(nodes[0]["classes"][0]["name"], "title");
    assert!(parsed["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn parse_outputs_nested_structure_with_attributes() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("nested.hsml");

    fs::write(
        &input,
        "\
doctype html
html
  head
    meta(charset=\"utf-8\")
    title My Page
  body
    .container#app
      img.rounded(src=\"/photo.jpg\" alt=\"Photo\")
      p.text-gray Hello World
",
    )
    .unwrap();

    let output = cmd()
        .args(["parse", input.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("should be valid JSON");

    let nodes = parsed["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 2); // doctype + html

    // Doctype
    assert_eq!(nodes[0]["type"], "doctype");
    assert_eq!(nodes[0]["doctype"], "html");

    // html > head > meta
    let html = &nodes[1];
    assert_eq!(html["tag"], "html");
    let head = &html["children"][0];
    assert_eq!(head["tag"], "head");
    let meta = &head["children"][0];
    assert_eq!(meta["tag"], "meta");
    assert_eq!(meta["attributes"][0]["type"], "attribute");
    assert_eq!(meta["attributes"][0]["key"], "charset");
    assert_eq!(meta["attributes"][0]["value"], "utf-8");

    // html > head > title
    let title = &head["children"][1];
    assert_eq!(title["tag"], "title");
    assert_eq!(title["text"]["text"], "My Page");

    // html > body > .container#app
    let body = &html["children"][1];
    assert_eq!(body["tag"], "body");
    let container = &body["children"][0];
    assert_eq!(container["tag"], "div"); // implicit div
    assert_eq!(container["ids"][0]["id"], "app");
    assert_eq!(container["classes"][0]["name"], "container");

    // img with classes and attributes
    let img = &container["children"][0];
    assert_eq!(img["tag"], "img");
    assert_eq!(img["classes"][0]["name"], "rounded");
    assert_eq!(img["attributes"][0]["key"], "src");
    assert_eq!(img["attributes"][0]["value"], "/photo.jpg");
    assert_eq!(img["attributes"][1]["key"], "alt");
    assert_eq!(img["attributes"][1]["value"], "Photo");

    // p with class and text
    let p = &container["children"][1];
    assert_eq!(p["tag"], "p");
    assert_eq!(p["classes"][0]["name"], "text-gray");
    assert_eq!(p["text"]["text"], "Hello World");
}

#[test]
fn parse_includes_diagnostics() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("warn.hsml");

    fs::write(&input, "h1.foo.foo Hello\n").unwrap();

    let output = cmd()
        .args(["parse", input.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("should be valid JSON");

    let diagnostics = parsed["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["code"], "W002");
    assert_eq!(diagnostics[0]["severity"], "warning");
}

#[test]
fn parse_returns_null_nodes_with_error_diagnostic_for_invalid_file() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("bad.hsml");

    fs::write(&input, "%%%invalid\n").unwrap();

    let output = cmd()
        .args(["parse", input.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("should be valid JSON");

    assert!(parsed["nodes"].is_null());
    let diagnostics = parsed["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["severity"], "error");
}

#[test]
fn parse_directory_outputs_array_with_file_paths() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.hsml"), "h1 A\n").unwrap();
    fs::write(dir.path().join("b.hsml"), "h2 B\n").unwrap();

    let output = cmd()
        .args(["parse", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("should be valid JSON");

    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert!(arr[0]["filePath"].is_string());
    assert!(arr[0]["nodes"].is_array());
    assert!(arr[0]["diagnostics"].is_array());
    assert!(arr[1]["filePath"].is_string());
    assert!(arr[1]["nodes"].is_array());
    assert!(arr[1]["diagnostics"].is_array());
}

#[test]
fn parse_directory_continues_on_parse_error() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("good.hsml"), "h1 Hello\n").unwrap();
    fs::write(dir.path().join("bad.hsml"), "%%%invalid\n").unwrap();

    let output = cmd()
        .args(["parse", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("should be valid JSON");

    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);

    // Find the good and bad results by checking nodes
    let good = arr.iter().find(|f| f["nodes"].is_array()).unwrap();
    let bad = arr.iter().find(|f| f["nodes"].is_null()).unwrap();

    assert!(good["filePath"].as_str().unwrap().contains("good.hsml"));
    assert!(good["diagnostics"].as_array().unwrap().is_empty());

    assert!(bad["filePath"].as_str().unwrap().contains("bad.hsml"));
    let diags = bad["diagnostics"].as_array().unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0]["severity"], "error");
}

#[test]
fn parse_fails_on_missing_file() {
    cmd()
        .args(["parse", "nonexistent.hsml"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("does not exist"));
}

#[test]
fn parse_fails_on_wrong_extension() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.txt");

    fs::write(&input, "h1 Hello\n").unwrap();

    cmd()
        .args(["parse", input.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains(".hsml extension"));
}

#[test]
fn fmt_formats_file_in_place() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "div\n    h1 Hello\n").unwrap();

    cmd()
        .args(["fmt", input.to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&input).unwrap(), "div\n  h1 Hello\n");
}

#[test]
fn fmt_check_fails_on_unformatted() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "div\n    h1 Hello\n").unwrap();

    cmd()
        .args(["fmt", "--check", input.to_str().unwrap()])
        .assert()
        .failure();

    // File should NOT be modified
    assert_eq!(fs::read_to_string(&input).unwrap(), "div\n    h1 Hello\n");
}

#[test]
fn fmt_check_succeeds_on_formatted() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "div\n  h1 Hello\n").unwrap();

    cmd()
        .args(["fmt", "--check", input.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn fmt_formats_directory() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.hsml"), "div\n    h1 A\n").unwrap();
    fs::write(dir.path().join("b.hsml"), "div\n    h2 B\n").unwrap();

    cmd()
        .args(["fmt", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("a.hsml")).unwrap(),
        "div\n  h1 A\n"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("b.hsml")).unwrap(),
        "div\n  h2 B\n"
    );
}

#[test]
fn fmt_normalizes_attribute_commas() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "img(src=\"a\"  alt=\"b\")\n").unwrap();

    cmd()
        .args(["fmt", input.to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(&input).unwrap(),
        "img(src=\"a\", alt=\"b\")\n"
    );
}

#[test]
fn fmt_debug_shows_timing_for_single_file() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "h1 Hello\n").unwrap();

    let output = cmd()
        .args(["fmt", "--debug", "--no-color", input.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("µs") || stdout.contains("ms"),
        "should show timing, got: {stdout}"
    );
    assert!(
        stdout.contains("unchanged"),
        "already formatted file should show unchanged, got: {stdout}"
    );
}

#[test]
fn fmt_debug_shows_directory_summary() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.hsml"), "h1 A\n").unwrap();
    fs::write(dir.path().join("b.hsml"), "h2 B\n").unwrap();

    let output = cmd()
        .args(["fmt", "--debug", "--no-color", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Formatting 2 file(s)"),
        "should show file count, got: {stdout}"
    );
    assert!(
        stdout.contains("2 files formatted in"),
        "should show summary, got: {stdout}"
    );
}

#[test]
fn fmt_debug_check_shows_needs_formatting() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("test.hsml");

    fs::write(&input, "div\n    h1 Hello\n").unwrap();

    let output = cmd()
        .args([
            "fmt",
            "--debug",
            "--no-color",
            "--check",
            input.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("needs formatting"),
        "unformatted file should show 'needs formatting', got: {stdout}"
    );
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

    fs::write(&input, "%%%invalid\n").unwrap();

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
fn check_github_format_outputs_warning_annotation() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("warn.hsml");

    fs::write(&input, "h1.foo.foo Hello\n").unwrap();

    let output = cmd()
        .args([
            "check",
            input.to_str().unwrap(),
            "--report-format",
            "github",
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let path = input.display().to_string();
    assert_eq!(
        stderr.trim(),
        format!(
            "::warning file={path},line=1,col=7,endLine=1,endColumn=11,title=W002::Duplicate class 'foo'"
        )
    );
}

#[test]
fn check_github_format_outputs_error_annotation() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("bad.hsml");

    fs::write(&input, "%%%invalid\n").unwrap();

    let output = cmd()
        .args([
            "check",
            input.to_str().unwrap(),
            "--report-format",
            "github",
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let path = input.display().to_string();
    assert_eq!(
        stderr.trim(),
        format!("::error file={path},line=1,col=1::parse error")
    );
}

#[test]
fn compile_github_format_outputs_warning_annotation() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("warn.hsml");

    fs::write(&input, "h1.foo.foo Hello\n").unwrap();

    let output = cmd()
        .args([
            "compile",
            input.to_str().unwrap(),
            "--report-format",
            "github",
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let path = input.display().to_string();
    assert_eq!(
        stderr.trim(),
        format!(
            "::warning file={path},line=1,col=7,endLine=1,endColumn=11,title=W002::Duplicate class 'foo'"
        )
    );
}

#[test]
fn check_json_emits_empty_array_for_clean_run() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("clean.hsml");

    fs::write(&input, "h1 Hello\n").unwrap();

    let output = cmd()
        .args(["check", input.to_str().unwrap(), "--report-format", "json"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.trim(), "[]");
}

#[test]
fn check_directory() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("good.hsml"), "h1 OK\n").unwrap();
    fs::write(dir.path().join("bad.hsml"), "%%%\n").unwrap();

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

// --- Ignore patterns ---

#[test]
fn compile_ignores_node_modules() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let nm = dir.path().join("node_modules").join("pkg");
    fs::create_dir_all(&nm).unwrap();
    fs::write(nm.join("lib.hsml"), "h2 Ignored\n").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join("index.html").exists());
    assert!(
        !nm.join("lib.html").exists(),
        "node_modules should be ignored"
    );
}

#[test]
fn check_ignores_node_modules() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let nm = dir.path().join("node_modules").join("pkg");
    fs::create_dir_all(&nm).unwrap();
    // Invalid hsml that would cause an error if not ignored
    fs::write(nm.join("bad.hsml"), "%%%invalid\n").unwrap();

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn compile_respects_gitignore() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join(".gitignore"), "ignored/\n").unwrap();
    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let ignored = dir.path().join("ignored");
    fs::create_dir(&ignored).unwrap();
    fs::write(ignored.join("skip.hsml"), "h2 Skipped\n").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join("index.html").exists());
    assert!(
        !ignored.join("skip.html").exists(),
        ".gitignore patterns should be respected"
    );
}

#[test]
fn check_respects_gitignore() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join(".gitignore"), "ignored/\n").unwrap();
    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let ignored = dir.path().join("ignored");
    fs::create_dir(&ignored).unwrap();
    fs::write(ignored.join("bad.hsml"), "%%%invalid\n").unwrap();

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn compile_respects_hsmlignore() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join(".hsmlignore"), "vendor/\n").unwrap();
    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let vendor = dir.path().join("vendor");
    fs::create_dir(&vendor).unwrap();
    fs::write(vendor.join("lib.hsml"), "h2 Vendor\n").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join("index.html").exists());
    assert!(
        !vendor.join("lib.html").exists(),
        ".hsmlignore patterns should be respected"
    );
}

#[test]
fn check_respects_hsmlignore() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join(".hsmlignore"), "vendor/\n").unwrap();
    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let vendor = dir.path().join("vendor");
    fs::create_dir(&vendor).unwrap();
    fs::write(vendor.join("bad.hsml"), "%%%invalid\n").unwrap();

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn compile_respects_ignore_pattern_flag() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let skip = dir.path().join("skip");
    fs::create_dir(&skip).unwrap();
    fs::write(skip.join("file.hsml"), "h2 Skipped\n").unwrap();

    cmd()
        .args([
            "compile",
            dir.path().to_str().unwrap(),
            "--ignore-pattern",
            "skip/",
        ])
        .assert()
        .success();

    assert!(dir.path().join("index.html").exists());
    assert!(
        !skip.join("file.html").exists(),
        "--ignore-pattern should exclude matching paths"
    );
}

#[test]
fn check_respects_ignore_pattern_flag() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let skip = dir.path().join("skip");
    fs::create_dir(&skip).unwrap();
    fs::write(skip.join("bad.hsml"), "%%%invalid\n").unwrap();

    cmd()
        .args([
            "check",
            dir.path().to_str().unwrap(),
            "--ignore-pattern",
            "skip/",
        ])
        .assert()
        .success();
}

#[test]
fn compile_ignore_pattern_supports_multiple_patterns() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let a = dir.path().join("a");
    let b = dir.path().join("b");
    fs::create_dir(&a).unwrap();
    fs::create_dir(&b).unwrap();
    fs::write(a.join("file.hsml"), "h2 A\n").unwrap();
    fs::write(b.join("file.hsml"), "h2 B\n").unwrap();

    cmd()
        .args([
            "compile",
            dir.path().to_str().unwrap(),
            "--ignore-pattern",
            "a/",
            "--ignore-pattern",
            "b/",
        ])
        .assert()
        .success();

    assert!(dir.path().join("index.html").exists());
    assert!(!a.join("file.html").exists(), "dir a should be ignored");
    assert!(!b.join("file.html").exists(), "dir b should be ignored");
}

#[test]
fn compile_skips_hidden_directories() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let hidden = dir.path().join(".hidden");
    fs::create_dir(&hidden).unwrap();
    fs::write(hidden.join("secret.hsml"), "h2 Hidden\n").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join("index.html").exists());
    assert!(
        !hidden.join("secret.html").exists(),
        "hidden directories should be skipped"
    );
}

#[test]
fn compile_builtin_ignores_target_directory() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let target = dir.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("out.hsml"), "h2 Target\n").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join("index.html").exists());
    assert!(
        !target.join("out.html").exists(),
        "target/ should be ignored by default"
    );
}

#[test]
fn compile_hsmlignore_can_reinclude_builtin_ignores() {
    let dir = TempDir::new().unwrap();

    // Re-include the `build/` directory which is ignored by default
    fs::write(dir.path().join(".hsmlignore"), "!build/\n").unwrap();
    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let build = dir.path().join("build");
    fs::create_dir(&build).unwrap();
    fs::write(build.join("page.hsml"), "h2 Built\n").unwrap();

    cmd()
        .args(["compile", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join("index.html").exists());
    assert!(
        build.join("page.html").exists(),
        "build/ should be re-included via .hsmlignore !build/"
    );
}

#[test]
fn check_skips_hidden_directories() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let hidden = dir.path().join(".hidden");
    fs::create_dir(&hidden).unwrap();
    fs::write(hidden.join("bad.hsml"), "%%%invalid\n").unwrap();

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn check_builtin_ignores_target_directory() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let target = dir.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("bad.hsml"), "%%%invalid\n").unwrap();

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn check_hsmlignore_can_reinclude_builtin_ignores() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join(".hsmlignore"), "!build/\n").unwrap();
    fs::write(dir.path().join("index.hsml"), "h1 Hello\n").unwrap();

    let build = dir.path().join("build");
    fs::create_dir(&build).unwrap();
    fs::write(build.join("page.hsml"), "h2 Built\n").unwrap();

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn check_debug_summary_shows_checked() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.hsml"), "h1 A\n").unwrap();
    fs::write(dir.path().join("b.hsml"), "h2 B\n").unwrap();

    let output = cmd()
        .args([
            "check",
            "--debug",
            "--no-color",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓") && stdout.contains("2 files checked in"),
        "check --debug should show summary with 'checked', got: {stdout}"
    );
}

#[test]
fn check_debug_summary_shows_warnings() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("warn.hsml"), "h1.foo.foo Hello\n").unwrap();

    let output = cmd()
        .args([
            "check",
            "--debug",
            "--no-color",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("✓") && stdout.contains("1 warning"),
        "check with warnings should show warning count, got: {stdout}"
    );
}

#[test]
fn compile_builtin_ignore_does_not_match_parent_dirs() {
    // If the project lives under a path containing a built-in ignore name
    // (e.g. /tmp/.../build/project/), files should NOT be filtered out.
    let dir = TempDir::new().unwrap();
    let project = dir.path().join("build").join("project");
    fs::create_dir_all(&project).unwrap();

    fs::write(project.join("index.hsml"), "h1 Hello\n").unwrap();

    cmd()
        .args(["compile", project.to_str().unwrap()])
        .assert()
        .success();

    assert!(
        project.join("index.html").exists(),
        "files should compile even if project is under a build/ parent"
    );
}
