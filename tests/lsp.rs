#![cfg(not(target_arch = "wasm32"))]

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TEST_TIMEOUT: Duration = Duration::from_secs(10);

struct LspProcess {
    child: std::process::Child,
    deadline: Instant,
}

impl LspProcess {
    fn new() -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_hsml"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to start hsml lsp");

        Self {
            child,
            deadline: Instant::now() + TEST_TIMEOUT,
        }
    }

    fn stdin(&mut self) -> &mut std::process::ChildStdin {
        self.child.stdin.as_mut().unwrap()
    }

    fn stdout(&mut self) -> &mut std::process::ChildStdout {
        self.child.stdout.as_mut().unwrap()
    }

    fn shutdown(mut self) {
        let stdin = self.child.stdin.as_mut().unwrap();
        send(
            stdin,
            r#"{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}"#,
        );
        send(stdin, r#"{"jsonrpc":"2.0","method":"exit","params":null}"#);
        let _ = self.child.wait();
    }
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

fn read_stdout(lsp: &mut LspProcess) -> String {
    let deadline = lsp.deadline;
    let stdout = lsp.stdout();
    let mut header = Vec::new();
    let mut content_length = 0usize;

    // Read headers byte by byte until \r\n\r\n
    loop {
        if Instant::now() > deadline {
            panic!("LSP test timed out after {TEST_TIMEOUT:?}");
        }
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
fn read_until(lsp: &mut LspProcess, expected: &str) -> String {
    for _ in 0..20 {
        let msg = read_stdout(lsp);
        if msg.contains(expected) {
            return msg;
        }
    }
    panic!("did not find message containing '{expected}'");
}

fn initialize(lsp: &mut LspProcess) {
    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
    );
    let response = read_until(lsp, "\"id\":1");
    assert!(response.contains("HSML Language Server"));

    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
    );
}

#[test]
fn lsp_initialize_returns_capabilities() {
    let mut lsp = LspProcess::new();

    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
    );

    let response = read_until(&mut lsp, "\"id\":1");
    assert!(response.contains("HSML Language Server"));
    assert!(response.contains("textDocumentSync"));

    lsp.shutdown();
}

#[test]
fn lsp_did_open_publishes_diagnostics_for_invalid_file() {
    let mut lsp = LspProcess::new();
    initialize(&mut lsp);

    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.hsml","languageId":"hsml","version":1,"text":"@@@invalid\n"}}}"#,
    );

    let msg = read_until(&mut lsp, "publishDiagnostics");
    assert!(msg.contains("file:///test.hsml"));
    assert!(!msg.contains("\"diagnostics\":[]"));

    lsp.shutdown();
}

#[test]
fn lsp_did_open_publishes_empty_diagnostics_for_valid_file() {
    let mut lsp = LspProcess::new();
    initialize(&mut lsp);

    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///valid.hsml","languageId":"hsml","version":1,"text":"h1 Hello\n"}}}"#,
    );

    let msg = read_until(&mut lsp, "publishDiagnostics");
    assert!(msg.contains("\"diagnostics\":[]"));

    lsp.shutdown();
}

#[test]
fn lsp_did_change_updates_diagnostics() {
    let mut lsp = LspProcess::new();
    initialize(&mut lsp);

    // Open with invalid content
    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.hsml","languageId":"hsml","version":1,"text":"@@@invalid\n"}}}"#,
    );
    let msg = read_until(&mut lsp, "publishDiagnostics");
    assert!(!msg.contains("\"diagnostics\":[]"));

    // Change to valid content
    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///test.hsml","version":2},"contentChanges":[{"text":"h1 Hello\n"}]}}"#,
    );
    let msg = read_until(&mut lsp, "publishDiagnostics");
    assert!(msg.contains("\"diagnostics\":[]"));

    lsp.shutdown();
}

#[test]
fn lsp_did_close_clears_diagnostics() {
    let mut lsp = LspProcess::new();
    initialize(&mut lsp);

    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.hsml","languageId":"hsml","version":1,"text":"@@@invalid\n"}}}"#,
    );
    let _ = read_until(&mut lsp, "publishDiagnostics");

    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":"file:///test.hsml"}}}"#,
    );
    let msg = read_until(&mut lsp, "publishDiagnostics");
    assert!(msg.contains("\"diagnostics\":[]"));

    lsp.shutdown();
}

#[test]
fn lsp_diagnostics_include_warning_codes() {
    let mut lsp = LspProcess::new();
    initialize(&mut lsp);

    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///warn.hsml","languageId":"hsml","version":1,"text":"h1.foo.foo Hello\n"}}}"#,
    );

    let msg = read_until(&mut lsp, "publishDiagnostics");
    assert!(msg.contains("W002"));

    lsp.shutdown();
}

#[test]
fn lsp_hover_returns_diagnostic_info_at_error_position() {
    let mut lsp = LspProcess::new();
    initialize(&mut lsp);

    // Open file with parse error at line 1, column 1
    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///hover.hsml","languageId":"hsml","version":1,"text":"@@@invalid\n"}}}"#,
    );
    let _ = read_until(&mut lsp, "publishDiagnostics");

    // Hover at position (0,0) — where the error is
    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///hover.hsml"},"position":{"line":0,"character":0}}}"#,
    );

    let msg = read_until(&mut lsp, "\"id\":2");
    assert!(msg.contains("error"));

    lsp.shutdown();
}

#[test]
fn lsp_hover_shows_html_tag_documentation() {
    let mut lsp = LspProcess::new();
    initialize(&mut lsp);

    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tag.hsml","languageId":"hsml","version":1,"text":"h1 Hello\n"}}}"#,
    );
    let _ = read_until(&mut lsp, "publishDiagnostics");

    // Hover on the tag name "h1"
    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tag.hsml"},"position":{"line":0,"character":0}}}"#,
    );

    let msg = read_until(&mut lsp, "\"id\":2");
    assert!(msg.contains("section heading"));
    assert!(msg.contains("MDN Reference"));

    lsp.shutdown();
}

#[test]
fn lsp_hover_returns_null_for_unknown_tag() {
    let mut lsp = LspProcess::new();
    initialize(&mut lsp);

    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///custom.hsml","languageId":"hsml","version":1,"text":"mycomponent Hello\n"}}}"#,
    );
    let _ = read_until(&mut lsp, "publishDiagnostics");

    // Hover on unknown tag
    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///custom.hsml"},"position":{"line":0,"character":0}}}"#,
    );

    let msg = read_until(&mut lsp, "\"id\":2");
    assert!(msg.contains("\"result\":null"));

    lsp.shutdown();
}

#[test]
fn lsp_hover_shows_warning_code_description() {
    let mut lsp = LspProcess::new();
    initialize(&mut lsp);

    // Open file with duplicate class warning at column 8
    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///warn.hsml","languageId":"hsml","version":1,"text":"h1.foo.foo Hello\n"}}}"#,
    );
    let _ = read_until(&mut lsp, "publishDiagnostics");

    // Hover at the warning position (0-based: line 0, char 6)
    send(
        lsp.stdin(),
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///warn.hsml"},"position":{"line":0,"character":6}}}"#,
    );

    let msg = read_until(&mut lsp, "\"id\":2");
    assert!(msg.contains("W002"));
    assert!(msg.contains("Duplicate class"));

    lsp.shutdown();
}
