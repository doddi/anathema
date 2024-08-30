mod local_spawner;

use crate::local_spawner::{LocalSpawner, Task};
use log::{debug, info};
use std::fs::File;
use std::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

struct Backend {
    spawner: LocalSpawner,
    client: Client,
}

impl Backend {
    pub fn new(spawner: LocalSpawner, client: Client) -> Self {
        Backend { spawner, client }
    }

    async fn compile(&self, uri: Url, content: &str) {
        self.spawner
            .spawn(Task::Compile(uri, content.to_string(), self.client.clone()));
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
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
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

        self.compile(params.text_document.uri, params.content_changes[0].text.as_str())
            .await;
    }
}

#[tokio::main]
async fn main() {
    let log_file = File::create("/tmp/trace.log").expect("should create trace file");

    let env = EnvFilter::from_default_env().add_directive(LevelFilter::TRACE.into());

    tracing_subscriber::fmt()
        .with_env_filter(env)
        .with_writer(Mutex::new(log_file))
        .init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::build(|client| {
        let spawner = LocalSpawner::new();
        Backend::new(spawner, client)
    })
    .finish();
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
