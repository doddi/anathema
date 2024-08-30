mod local_spawner;

use log::{debug, info};
use std::fs::File;
use std::sync::Mutex;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::RecvError;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{Diagnostic, DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url};
use tower_lsp::{lsp_types, Client, LanguageServer, LspService, Server};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use crate::local_spawner::{LocalSpawner, Task};

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend { client }
    }

    async fn compile(&self, uri: Url, content: &str) {
        let spawner = LocalSpawner::new();
        let (send, response) = oneshot::channel();
        spawner.spawn(Task::Compile(content.to_string(), send));

        let result = response.await;
        
        let compilation_result = match result {
            Ok(cr) => cr,
            Err(err) => panic!("error: {:?}", err), 
        };
        
        debug!("received response");
        
        match compilation_result {
            None => {
                debug!("compilation success");
                self.client
                    .log_message(
                        lsp_types::MessageType::INFO,
                        "anathema template compilation successful",
                    )
                    .await;

                self.client.publish_diagnostics(uri, vec![], None).await;
            }
            Some(diagnostics) => {
                self.client.publish_diagnostics(uri, diagnostics, None).await;
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

    let env = EnvFilter::from_default_env()
        .add_directive(LevelFilter::TRACE.into());
    
    tracing_subscriber::fmt()
        .with_env_filter(env)
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
