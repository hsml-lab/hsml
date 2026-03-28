#![cfg(not(target_arch = "wasm32"))]

use std::io::{Read, Write};
use std::process::{Command, Stdio};

fn lsp_cmd() -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_hsml"))
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start hsml lsp")
}

fn send(stdin: &mut impl Write, content: &str) {
    write!(
        stdin,
        "Content-Length: {}\r\n\r\n{}",
        content.len(),
        content
    )
    .unwrap();
    stdin.flush().unwrap();
}

fn read_stdout(stdout: &mut impl Read) -> String {
    // Read in small chunks to avoid BufReader buffering issues
    let mut header = Vec::new();
    let mut content_length = 0usize;

    // Read headers byte by byte until \r\n\r\n
    loop {
        let mut buf = [0u8; 1];
        stdout.read_exact(&mut buf).unwrap();
        header.push(buf[0]);

        if header.len() >= 4 && header[header.len() - 4..] == [b'\r', b'\n', b'\r', b'\n'] {
            break;
        }
    }

    let header_str = String::from_utf8_lossy(&header);
    for line in header_str.split("\r\n") {
        if let Some(len) = line.strip_prefix("Content-Length: ") {
            content_length = len.parse().unwrap();
        }
    }

    let mut body = vec![0u8; content_length];
    stdout.read_exact(&mut body).unwrap();
    String::from_utf8(body).unwrap()
}

/// Read messages until one contains the expected substring.
fn read_until(stdout: &mut impl Read, expected: &str) -> String {
    for _ in 0..20 {
        let msg = read_stdout(stdout);
        if msg.contains(expected) {
            return msg;
        }
    }
    panic!("did not find message containing '{expected}'");
}

#[test]
fn lsp_initialize_returns_capabilities() {
    let mut child = lsp_cmd();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
    );

    let response = read_until(stdout, "\"id\":1");
    assert!(response.contains("HSML Language Server"));
    assert!(response.contains("textDocumentSync"));

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}"#,
    );
    send(stdin, r#"{"jsonrpc":"2.0","method":"exit","params":null}"#);
    child.wait().unwrap();
}

#[test]
fn lsp_did_open_publishes_diagnostics_for_invalid_file() {
    let mut child = lsp_cmd();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
    );
    let _ = read_until(stdout, "\"id\":1");
    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
    );

    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.hsml","languageId":"hsml","version":1,"text":"@@@invalid\n"}}}"#,
    );

    let msg = read_until(stdout, "publishDiagnostics");
    assert!(msg.contains("file:///test.hsml"));
    assert!(!msg.contains("\"diagnostics\":[]"));

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}"#,
    );
    send(stdin, r#"{"jsonrpc":"2.0","method":"exit","params":null}"#);
    child.wait().unwrap();
}

#[test]
fn lsp_did_open_publishes_empty_diagnostics_for_valid_file() {
    let mut child = lsp_cmd();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
    );
    let _ = read_until(stdout, "\"id\":1");
    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
    );

    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///valid.hsml","languageId":"hsml","version":1,"text":"h1 Hello\n"}}}"#,
    );

    let msg = read_until(stdout, "publishDiagnostics");
    assert!(msg.contains("\"diagnostics\":[]"));

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}"#,
    );
    send(stdin, r#"{"jsonrpc":"2.0","method":"exit","params":null}"#);
    child.wait().unwrap();
}

#[test]
fn lsp_did_change_updates_diagnostics() {
    let mut child = lsp_cmd();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
    );
    let _ = read_until(stdout, "\"id\":1");
    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
    );

    // Open with invalid content
    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.hsml","languageId":"hsml","version":1,"text":"@@@invalid\n"}}}"#,
    );
    let msg = read_until(stdout, "publishDiagnostics");
    assert!(!msg.contains("\"diagnostics\":[]"));

    // Change to valid content
    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///test.hsml","version":2},"contentChanges":[{"text":"h1 Hello\n"}]}}"#,
    );
    let msg = read_until(stdout, "publishDiagnostics");
    assert!(msg.contains("\"diagnostics\":[]"));

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}"#,
    );
    send(stdin, r#"{"jsonrpc":"2.0","method":"exit","params":null}"#);
    child.wait().unwrap();
}

#[test]
fn lsp_did_close_clears_diagnostics() {
    let mut child = lsp_cmd();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
    );
    let _ = read_until(stdout, "\"id\":1");
    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
    );

    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.hsml","languageId":"hsml","version":1,"text":"@@@invalid\n"}}}"#,
    );
    let _ = read_until(stdout, "publishDiagnostics");

    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":"file:///test.hsml"}}}"#,
    );
    let msg = read_until(stdout, "publishDiagnostics");
    assert!(msg.contains("\"diagnostics\":[]"));

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}"#,
    );
    send(stdin, r#"{"jsonrpc":"2.0","method":"exit","params":null}"#);
    child.wait().unwrap();
}

#[test]
fn lsp_diagnostics_include_warning_codes() {
    let mut child = lsp_cmd();
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
    );
    let _ = read_until(stdout, "\"id\":1");
    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
    );

    send(
        stdin,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///warn.hsml","languageId":"hsml","version":1,"text":"h1.foo.foo Hello\n"}}}"#,
    );

    let msg = read_until(stdout, "publishDiagnostics");
    assert!(msg.contains("W002"));

    send(
        stdin,
        r#"{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}"#,
    );
    send(stdin, r#"{"jsonrpc":"2.0","method":"exit","params":null}"#);
    child.wait().unwrap();
}
