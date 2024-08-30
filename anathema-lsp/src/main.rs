mod local_spawner;
mod hover_dictionary;
mod document_store;

use crate::local_spawner::{LocalSpawner, Task};
use log::{debug, info};
use std::fs::File;
use std::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

struct Backend {
    spawner: LocalSpawner,
    client: Client,
    store: document_store::DocumentStore,
    hover_dictionary: hover_dictionary::HoverDictionary,
}

impl Backend {
    pub fn new(spawner: LocalSpawner, client: Client) -> Self {
        Backend {
            spawner,
            client,
            store: document_store::DocumentStore::new(),
            hover_dictionary: hover_dictionary::HoverDictionary::new(),
        }
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
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
        self.store.insert(params.text_document.uri.clone(), params.text_document.text.clone());
        self.compile(params.text_document.uri, params.text_document.text.as_str())
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        info!("doc changed {}", params.text_document.uri);
        self.store.insert(params.text_document.uri.clone(), params.content_changes[0].text.clone());
        self.compile(params.text_document.uri, params.content_changes[0].text.as_str())
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.store.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<tower_lsp::lsp_types::Hover>> {
        if let Some(document) = self.store.get(&params.text_document_position_params.text_document.uri) {
            let line_pos = params.text_document_position_params.position.line;
            let character_pos = params.text_document_position_params.position.character;

            if let Some(line) = document.lines().nth(line_pos as usize) {
                //find the word at the character position in the line
                let word = line.split_whitespace().find(|word| {
                    let start = line.find(word).unwrap();
                    let end = start + word.len();
                    start <= character_pos as usize && character_pos as usize <= end
                });

                if let Some(word) = word {
                    if let Some(markup) = self.hover_dictionary.lookup_word_markup(word) {
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(markup),
                            range: None,
                        }))
                    }
                }
            }
        }
        Ok(None)
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
