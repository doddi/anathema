use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower_lsp::lsp_types::Url;

pub(crate) struct DocumentStore {
    documents: Arc<Mutex<HashMap<Url, String>>>,
}

impl DocumentStore {
    pub(crate) fn new() -> Self {
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub(crate) fn get(&self, uri: &Url) -> Option<String> {
        self.documents.lock().unwrap().get(uri).cloned()
    }

    pub(crate) fn insert(&self, uri: Url, content: String) {
        self.documents.lock().unwrap().insert(uri, content);
    }

    pub(crate) fn remove(&self, url: &Url) {
        self.documents.lock().unwrap().remove(url);
    }
}

