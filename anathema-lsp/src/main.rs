use log::{debug, info};
use std::fs::File;
use std::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    Diagnostic, DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams,
    InitializeResult, InitializedParams, Range, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{lsp_types, Client, LanguageServer, LspService, Server};
use tracing_subscriber::EnvFilter;
use anathema_templates::error::Error::ParseError;

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend { client }
    }

    async fn compile(&self, uri: Url, content: &str) {
        let compilation_result = anathema_templates::Document::new(content).compile();

        match compilation_result {
            Ok(_) => {
                debug!("compilation success");
                self.client
                    .log_message(
                        lsp_types::MessageType::INFO,
                        "anathema template compilation successful",
                    )
                    .await;

                self.client.publish_diagnostics(uri, vec![], None).await;
            }
            Err(err) => {
                match err {
                    ParseError(msg) => {
                        let line = msg.line;
                        let line_length = content.lines().nth(line - 1).unwrap().len();
                        self.client
                            .publish_diagnostics(
                                uri,
                                vec![Diagnostic::new_simple(
                                    Range::new(
                                        lsp_types::Position::new(line as u32 - 1, 0),
                                        lsp_types::Position::new(line as u32 - 1, line_length as u32),
                                    ),
                                    format!("{:?}", msg.kind),
                                )],
                                None,
                            )
                            .await;
                    }
                    // Error::CircularDependency => {}
                    // Error::MissingComponent(msg) => {}
                    // Error::EmptyTemplate => {}
                    // Error::EmptyBody => {}
                    // Error::Io(msg) => {}
                    _ => {}
                }
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        info!("Initializing");

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "anathema-lsp".to_string(),
                version: None,
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        debug!("Initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("doc opened {}", params.text_document.uri);

        self.compile(params.text_document.uri, params.text_document.text.as_str())
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        info!("doc changed {}", params.text_document.uri);

        self.compile(
            params.text_document.uri,
            params.content_changes[0].text.as_str(),
        )
            .await;
    }
}

#[tokio::main]
async fn main() {
    let log_file = File::create("/tmp/trace.log").expect("should create trace file");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(Mutex::new(log_file))
        .init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::build(|client| Backend::new(client)).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod test {
    #[test]
    fn compilation_fails() {
        let src = r#"
        vstack
            [
        "#;

        let result = anathema_templates::Document::new(src).compile();
        assert!(result.is_err());
    }

    #[test]
    fn compilation_successful() {
        let src = r#"
        vstack
            border
            border
                text
        "#;

        let result = anathema_templates::Document::new(src).compile();
        assert!(result.is_ok());
    }
}
