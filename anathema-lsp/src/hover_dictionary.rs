use std::collections::HashMap;
use tower_lsp::lsp_types::{MarkupContent, MarkupKind};

pub(crate) struct HoverDictionary {
    store: HashMap<&'static str, &'static str>,
}

const TEXT: &str = include_str!("../docs/text.md");
const SPAN: &str = include_str!("../docs/span.md");
const BORDER: &str = include_str!("../docs/border.md");
const ALIGN: &str = include_str!("../docs/align.md");
const VSTACK: &str = include_str!("../docs/vstack.md");
const HSTACK: &str = include_str!("../docs/hstack.md");

impl HoverDictionary {
    pub fn new() -> Self {
        let mut dictionary = HashMap::new();
        dictionary.insert("text", TEXT);
        dictionary.insert("span", SPAN);
        dictionary.insert("border", BORDER);
        dictionary.insert("align", ALIGN);
        dictionary.insert("vstack", VSTACK);
        dictionary.insert( "hstack", HSTACK);

        HoverDictionary {
            store: dictionary,
        }
    }

    pub(crate) fn lookup_word_markup(&self, word: &str) -> Option<MarkupContent> {
        if let Some(description) = self.store.get(word) {
            return Some(MarkupContent {
                kind: MarkupKind::Markdown,
                value: description.to_string(),
            });
        }
        None
    }
}
