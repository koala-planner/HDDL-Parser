use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use super::*;

pub struct RequestHandler {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, Vec<u8>>>>,
}

impl RequestHandler {
    pub fn new(client: Client) -> RequestHandler {
        RequestHandler {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn sync(&self, url: Url, content: Vec<u8>) {
        self.documents.write().await.insert(url, content);
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for RequestHandler {
    async fn initialize(
        &self,
        _: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("HDDL Server".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                // Add other capabilities as needed
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "HDDL Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        ()
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.sync(
            params.text_document.uri,
            params.text_document.text.into_bytes(),
        )
        .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // get the saved file content
        let text = match params.text {
            Some(content) => content.into_bytes(),
            None => tokio::fs::read(params.text_document.uri.path())
                .await
                .unwrap(),
        };
        // sync the file
        self.sync(params.text_document.uri, text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        // TODO: add support for incremental change
        if let Some(new_text) = params.content_changes.into_iter().next() {
            self.sync(uri, new_text.text.into_bytes()).await
        }
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> tower_lsp::jsonrpc::Result<DocumentDiagnosticReportResult> {
        match self.documents.read().await.get(&params.text_document.uri) {
            Some(document) => {
                self.client
                    .log_message(MessageType::LOG, "Diagnostic Request Recieved.")
                    .await;
                let file_path = params.text_document.uri.to_file_path().unwrap();
                match classify_file(document) {
                    FileVariant::Domain => {
                        self.client
                            .log_message(
                                MessageType::LOG,
                                format!(
                                    "{} is a domain. Attempting to diagnose.",
                                    params.text_document.uri.to_string()
                                ),
                            )
                            .await;
                        let diagnosis = diagnose_domain(document);
                        return Ok(diagnosis);
                    }
                    FileVariant::Problem => {
                        let root_folder = file_path.parent().unwrap();
                        let mut files = tokio::fs::read_dir(root_folder).await.unwrap();
                        while let Ok(Some(entry)) = files.next_entry().await {
                            match entry.path().extension() {
                                Some(extension) if (extension == "hddl" || extension == "pddl") => {
                                    let content = tokio::fs::read(entry.path()).await.unwrap();
                                    match classify_file(&content) {
                                        FileVariant::Domain => {
                                            self.client
                                                .log_message(
                                                    MessageType::LOG,
                                                    format!(
                                                        "{} is the domain for {}. Attempting to diagnose.",
                                                        root_folder.to_str().unwrap(),
                                                        params.text_document.uri.to_string()
                                                    ),
                                                ).await;
                                            return Ok(diagnose_problem(Some(&content), document));
                                        }
                                        // File is not the domain
                                        _ => {}
                                    }
                                }
                                // File is not .PDDL or .HDDL
                                _ => {}
                            }
                        }
                        // could not find the domain
                        self.client
                            .log_message(
                                MessageType::LOG,
                                format!(
                                    "Could not find the domain in {}",
                                    root_folder.to_str().unwrap()
                                ),
                            )
                            .await;
                        return Ok(diagnose_problem(None, document));
                    }
                    FileVariant::MaybeNotHDDL => {
                        // TODO: attempt to fix this
                        self.client
                            .log_message(
                                MessageType::LOG,
                                format!(
                                    "{} does not have proper HDDL header. Ignoring diagnostic request.",
                                    params.text_document.uri.to_string()
                                ),
                            )
                            .await;
                        Ok(generate_empty_report())
                    }
                }
            }
            None => {
                return Err(tower_lsp::jsonrpc::Error::invalid_params(format!(
                    "{} is not synced",
                    params.text_document.uri
                )));
            }
        }
    }
}
