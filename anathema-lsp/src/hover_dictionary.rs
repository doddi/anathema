use std::collections::HashMap;
use tower_lsp::lsp_types::{MarkupContent, MarkupKind};
use crate::word_handling;

pub(crate) struct HoverDictionary {
    store: HashMap<&'static str, &'static str>,
}

const TEXT: &str = include_str!("../docs/text.md");
const SPAN: &str = include_str!("../docs/span.md");
const BORDER: &str = include_str!("../docs/border.md");
const ALIGN: &str = include_str!("../docs/align.md");
const VSTACK: &str = include_str!("../docs/vstack.md");
const HSTACK: &str = include_str!("../docs/hstack.md");
const ZSTACK: &str = include_str!("../docs/zstack.md");
const ROW: &str = include_str!("../docs/row.md");
const COLUMN: &str = include_str!("../docs/column.md");
const EXPAND: &str = include_str!("../docs/expand.md");
const POSITION: &str = include_str!("../docs/position.md");
const SPACER: &str = include_str!("../docs/spacer.md");
const OVERFLOW: &str = include_str!("../docs/overflow.md");
const PADDING: &str = include_str!("../docs/padding.md");
const CANVAS: &str = include_str!("../docs/canvas.md");
const CONTAINER: &str = include_str!("../docs/container.md");

impl HoverDictionary {
    pub fn new() -> Self {
        let mut dictionary = HashMap::new();
        dictionary.insert("text", TEXT);
        dictionary.insert("span", SPAN);
        dictionary.insert("border", BORDER);
        dictionary.insert("align", ALIGN);
        dictionary.insert("vstack", VSTACK);
        dictionary.insert( "hstack", HSTACK);
        dictionary.insert( "zstack", ZSTACK);
        dictionary.insert( "row", ROW);
        dictionary.insert( "column", COLUMN);
        dictionary.insert( "expand", EXPAND);
        dictionary.insert( "position", POSITION);
        dictionary.insert( "spacer", SPACER);
        dictionary.insert( "overflow", OVERFLOW);
        dictionary.insert( "padding", PADDING);
        dictionary.insert( "canvas", CANVAS);
        dictionary.insert( "container", CONTAINER);

        HoverDictionary {
            store: dictionary,
        }
    }

    pub(crate) fn lookup_word_markup(&self, line: &str, character_pos: u32) -> Option<MarkupContent> {
        if let Some(word) = word_handling::get_current_word(line, character_pos) {
            if let Some(description) = self.store.get(word) {
                return Some(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: description.to_string(),
                });
            }
        }
        None
    }
}
