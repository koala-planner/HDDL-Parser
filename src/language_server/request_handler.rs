use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc;
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

    async fn read_saved_document(&self, uri: &Url) -> Option<Vec<u8>> {
        let file_path = match uri.to_file_path() {
            Ok(file_path) => file_path,
            Err(()) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Could not read saved non-file URI {uri}"),
                    )
                    .await;
                return None;
            }
        };

        match tokio::fs::read(&file_path).await {
            Ok(content) => Some(content),
            Err(error) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!(
                            "Could not read saved document {}: {error}",
                            file_path.display()
                        ),
                    )
                    .await;
                None
            }
        }
    }

    fn diagnostic_file_path(uri: &Url) -> jsonrpc::Result<PathBuf> {
        uri.to_file_path().map_err(|()| {
            jsonrpc::Error::invalid_params(format!("diagnostic URI must be a file URI: {uri}"))
        })
    }

    async fn diagnose_problem_with_sibling_domain(
        &self,
        file_path: PathBuf,
        document: &Vec<u8>,
    ) -> DocumentDiagnosticReportResult {
        let Some(root_folder) = file_path.parent() else {
            return diagnose_problem(None, document);
        };

        let mut files = match tokio::fs::read_dir(root_folder).await {
            Ok(files) => files,
            Err(error) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!(
                            "Could not read sibling-domain directory {}: {error}",
                            root_folder.display()
                        ),
                    )
                    .await;
                return diagnose_problem(None, document);
            }
        };

        loop {
            let entry = match files.next_entry().await {
                Ok(Some(entry)) => entry,
                Ok(None) => break,
                Err(error) => {
                    self.client
                        .log_message(
                            MessageType::ERROR,
                            format!(
                                "Could not continue sibling-domain discovery in {}: {error}",
                                root_folder.display()
                            ),
                        )
                        .await;
                    break;
                }
            };

            match entry.path().extension() {
                Some(extension) if extension == "hddl" || extension == "pddl" => {
                    let entry_path = entry.path();
                    let content = match tokio::fs::read(&entry_path).await {
                        Ok(content) => content,
                        Err(error) => {
                            self.client
                                .log_message(
                                    MessageType::ERROR,
                                    format!(
                                        "Could not read sibling-domain candidate {}: {error}",
                                        entry_path.display()
                                    ),
                                )
                                .await;
                            continue;
                        }
                    };

                    if let FileVariant::Domain = classify_file(&content) {
                        self.client
                            .log_message(
                                MessageType::LOG,
                                format!(
                                    "{} is the domain for the diagnostic request. Attempting to diagnose.",
                                    entry_path.display()
                                ),
                            )
                            .await;
                        return diagnose_problem(Some(&content), document);
                    }
                }
                // File is not .PDDL or .HDDL
                _ => {}
            }
        }

        self.client
            .log_message(
                MessageType::LOG,
                format!("Could not find the domain in {}", root_folder.display()),
            )
            .await;
        diagnose_problem(None, document)
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
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {}

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
            None => match self.read_saved_document(&params.text_document.uri).await {
                Some(content) => content,
                None => return,
            },
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
                let file_path = Self::diagnostic_file_path(&params.text_document.uri)?;
                match classify_file(document) {
                    FileVariant::Domain => {
                        self.client
                            .log_message(
                                MessageType::LOG,
                                format!(
                                    "{} is a domain. Attempting to diagnose.",
                                    params.text_document.uri
                                ),
                            )
                            .await;
                        let diagnosis = diagnose_domain(document);
                        return Ok(diagnosis);
                    }
                    FileVariant::Problem => {
                        return Ok(self
                            .diagnose_problem_with_sibling_domain(file_path, document)
                            .await);
                    }
                    FileVariant::MaybeNotHDDL => {
                        // TODO: attempt to fix this
                        self.client
                            .log_message(
                                MessageType::LOG,
                                format!(
                                    "{} does not have proper HDDL header. Ignoring diagnostic request.",
                                    params.text_document.uri
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
