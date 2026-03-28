use std::collections::HashMap;
use std::sync::Mutex;

use clap::ArgMatches;
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        Diagnostic as LspDiagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
        InitializedParams, MessageType, NumberOrString, Position, Range, ServerCapabilities,
        ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
};

use hsml::diagnostic::{Diagnostic, Severity};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Mutex<HashMap<Url, String>>,
}

impl Backend {
    async fn publish_diagnostics(&self, uri: Url, source: &str) {
        let diagnostics = hsml::check_content(source);
        let lsp_diagnostics: Vec<LspDiagnostic> =
            diagnostics.iter().map(to_lsp_diagnostic).collect();
        self.client
            .publish_diagnostics(uri, lsp_diagnostics, None)
            .await;
    }
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
        let text = params.text_document.text.clone();

        self.documents
            .lock()
            .unwrap()
            .insert(uri.clone(), text.clone());

        self.publish_diagnostics(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;

            self.documents
                .lock()
                .unwrap()
                .insert(uri.clone(), text.clone());

            self.publish_diagnostics(uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        self.documents.lock().unwrap().remove(&uri);

        // Clear diagnostics for closed file
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }
}

pub async fn exec_lsp(_matches: &ArgMatches) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        documents: Mutex::new(HashMap::new()),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
