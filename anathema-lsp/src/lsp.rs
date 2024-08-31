use log::{debug, info};
use tower_lsp::{Client, LanguageServer};
use tower_lsp::lsp_types::{CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url};
use crate::{document_store, hover_dictionary};
use crate::auto_complete::get_auto_complete_options;
use crate::local_spawner::{LocalSpawner, Task};

pub(crate) struct Backend {
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
    async fn initialize(&self, _params: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        info!("Initializing");

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "anathema-lsp".to_string(),
                version: None,
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..ServerCapabilities::default()
            },
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        debug!("Initialized");
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
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

    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
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

    async fn completion(&self, params: CompletionParams) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        if let Some(document) = self.store.get(&params.text_document_position.text_document.uri) {
            let line_pos = params.text_document_position.position.line;
            let character_pos = params.text_document_position.position.character;

            if let Some(line) = document.lines().nth(line_pos as usize) {
                //find the word at the character position in the line
                let word = line.split_whitespace().find(|word| {
                    let start = line.find(word).unwrap();
                    let end = start + word.len();
                    start <= character_pos as usize && character_pos as usize <= end
                });

                if let Some(word) = word {
                    let complete_options = get_auto_complete_options(line, word);
                    if let Some(options) = complete_options { 
                        return Ok(Some(CompletionResponse::Array(options)));
                    }
                }
            }
        }
        Ok(None)
    }
}

