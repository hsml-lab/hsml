use std::collections::HashMap;
use std::sync::Mutex;

use clap::ArgMatches;
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        Diagnostic as LspDiagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, Hover, HoverContents, HoverParams,
        HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams,
        MarkupContent, MarkupKind, MessageType, NumberOrString, Position, Range,
        ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
};

use hsml::diagnostic::{Diagnostic, Severity};
use hsml::parser::error::ErrorCode;

#[cfg(test)]
mod tests;

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Mutex<HashMap<Url, (i32, String)>>,
    diagnostics: Mutex<HashMap<Url, Vec<LspDiagnostic>>>,
}

impl Backend {
    fn is_current_version(&self, uri: &Url, version: i32) -> bool {
        self.documents
            .lock()
            .unwrap()
            .get(uri)
            .is_some_and(|(v, _)| *v == version)
    }

    async fn publish_diagnostics(&self, uri: Url, version: i32, source: &str) {
        // Fast-path: skip parsing if a newer version already arrived
        if !self.is_current_version(&uri, version) {
            return;
        }

        let diagnostics = hsml::check_content(source);
        let lsp_diagnostics: Vec<LspDiagnostic> =
            diagnostics.iter().map(to_lsp_diagnostic).collect();

        // Re-check after parsing in case a newer version arrived during validation
        if !self.is_current_version(&uri, version) {
            return;
        }

        self.diagnostics
            .lock()
            .unwrap()
            .insert(uri.clone(), lsp_diagnostics.clone());

        self.client
            .publish_diagnostics(uri, lsp_diagnostics, Some(version))
            .await;
    }
}

/// Look up an error code description for hover display.
fn error_code_description(code: &str) -> Option<&'static str> {
    ErrorCode::ALL
        .iter()
        .find(|ec| ec.code() == code)
        .map(|ec| ec.message())
}

fn to_lsp_diagnostic(d: &Diagnostic) -> LspDiagnostic {
    let range = d
        .location
        .as_ref()
        .map(|loc| {
            let line = loc.line.saturating_sub(1);
            let col = loc.column.saturating_sub(1);
            let pos = Position::new(line, col);
            Range::new(pos, pos)
        })
        .unwrap_or_default();

    let severity = Some(match d.severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
    });

    LspDiagnostic {
        range,
        severity,
        code: d.code.as_ref().map(|c| NumberOrString::String(c.clone())),
        source: Some("hsml".to_string()),
        message: d.message.clone(),
        ..Default::default()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "initializing")
            .await;

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "HSML Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "shutting down")
            .await;

        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        let text = params.text_document.text.clone();

        self.documents
            .lock()
            .unwrap()
            .insert(uri.clone(), (version, text.clone()));

        self.publish_diagnostics(uri, version, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;

            self.documents
                .lock()
                .unwrap()
                .insert(uri.clone(), (version, text.clone()));

            self.publish_diagnostics(uri, version, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        self.documents.lock().unwrap().remove(&uri);
        self.diagnostics.lock().unwrap().remove(&uri);

        // Clear diagnostics for closed file
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let diagnostics = self.diagnostics.lock().unwrap();
        let Some(file_diagnostics) = diagnostics.get(uri) else {
            return Ok(None);
        };

        // Find diagnostics at the hover position
        let hover_diags: Vec<_> = file_diagnostics
            .iter()
            .filter(|d| d.range.start <= pos && pos <= d.range.end)
            .collect();

        if hover_diags.is_empty() {
            return Ok(None);
        }

        let mut parts = Vec::new();
        for d in &hover_diags {
            let severity = match d.severity {
                Some(DiagnosticSeverity::ERROR) => "error",
                Some(DiagnosticSeverity::WARNING) => "warning",
                _ => "diagnostic",
            };

            let code_str = match &d.code {
                Some(NumberOrString::String(c)) => {
                    let desc = error_code_description(c).unwrap_or(&d.message);
                    format!("**{severity}[{c}]**: {desc}")
                }
                _ => format!("**{severity}**: {}", d.message),
            };

            parts.push(code_str);
        }

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: parts.join("\n\n"),
            }),
            range: None,
        }))
    }
}

pub async fn exec_lsp(_matches: &ArgMatches) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        documents: Mutex::new(HashMap::new()),
        diagnostics: Mutex::new(HashMap::new()),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
