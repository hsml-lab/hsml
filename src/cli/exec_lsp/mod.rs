use std::collections::HashMap;
use std::sync::Mutex;

use clap::ArgMatches;
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        Diagnostic as LspDiagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams, Hover,
        HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
        InitializedParams, MarkupContent, MarkupKind, MessageType, NumberOrString, OneOf, Position,
        Range, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
        TextEdit, Url,
    },
};

use hsml::diagnostic::{Diagnostic, Severity};
use hsml::formatter::{FormatOptions, format};
use hsml::parser::Span;
use hsml::parser::error::ErrorCode;
use hsml::parser::parse::parse;

fn position_to_lsp(pos: &hsml::common::Position) -> Position {
    Position::new(pos.line.saturating_sub(1), pos.column.saturating_sub(1))
}

mod html_tags;

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
        .map(|loc| Range::new(position_to_lsp(&loc.start), position_to_lsp(&loc.end)))
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
                document_formatting_provider: Some(OneOf::Left(true)),
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

        // Check diagnostics at hover position
        let diagnostic_hover = {
            let diagnostics = self.diagnostics.lock().unwrap();
            if let Some(file_diagnostics) = diagnostics.get(uri) {
                let hover_diags: Vec<_> = file_diagnostics
                    .iter()
                    .filter(|d| {
                        if d.range.start == d.range.end {
                            // Zero-width (point) diagnostic: match at exact position
                            d.range.start == pos
                        } else {
                            // Span diagnostic: start inclusive, end exclusive
                            d.range.start <= pos && pos < d.range.end
                        }
                    })
                    .collect();

                if !hover_diags.is_empty() {
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
                    Some(parts.join("\n\n"))
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(value) = diagnostic_hover {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: None,
            }));
        }

        // Check for HTML tag documentation at hover position
        let documents = self.documents.lock().unwrap();
        let Some((_, source)) = documents.get(uri) else {
            return Ok(None);
        };

        let tag_hover = extract_tag_at_position(source, pos);

        if let Some(tag) = tag_hover
            && let Some(info) = html_tags::lookup(&tag)
        {
            let value = format!("{}\n\n[MDN Reference]({})", info.description, info.mdn_url);
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: None,
            }));
        }

        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        // Clone source and release lock before doing CPU work
        let source = {
            let documents = self.documents.lock().unwrap();
            let Some((_, source)) = documents.get(uri) else {
                return Ok(None);
            };
            source.clone()
        };

        let span = Span::new(&source);
        let Ok((_, ast)) = parse(span) else {
            return Ok(None);
        };

        let options = FormatOptions {
            indent_size: params.options.tab_size as usize,
            ..FormatOptions::default()
        };

        let formatted = format(&ast, &options);

        if formatted == source {
            return Ok(None);
        }

        // Replace the entire document
        let line_count = source.lines().count() as u32;
        let last_line_len = source.lines().last().map_or(0, |l| l.len() as u32);

        Ok(Some(vec![TextEdit {
            range: Range::new(
                Position::new(0, 0),
                Position::new(line_count, last_line_len),
            ),
            new_text: formatted,
        }]))
    }
}

/// Extract the tag name at a given cursor position from HSML source.
/// In HSML, the tag name is the first word on the line (after indentation),
/// before any `.`, `#`, `(`, or space.
fn extract_tag_at_position(source: &str, pos: Position) -> Option<String> {
    let line = source.lines().nth(pos.line as usize)?;
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();

    // Cursor must be within the tag name portion of the line
    let tag_name: String = trimmed
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();

    if tag_name.is_empty() {
        return None;
    }

    let tag_start = indent as u32;
    let tag_end = tag_start + tag_name.len() as u32;

    if pos.character >= tag_start && pos.character < tag_end {
        Some(tag_name.to_ascii_lowercase())
    } else {
        None
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
