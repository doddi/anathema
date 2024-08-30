use std::collections::HashMap;
use tower_lsp::lsp_types::{MarkupContent, MarkupKind};

pub(crate) struct HoverDictionary {
    store: HashMap<&'static str, &'static str>,
}

const VSTACK: &'static str = r#"vstack [docs](https://togglebyte.github.io/anathema-guide/templates/elements/vstack.html)
A vertical stack of elements, accepts many children
```yaml
vstack
  text "one"
  text "two"
```
"#;

const HSTACK: &'static str = r#"hstack [docs](https://togglebyte.github.io/anathema-guide/templates/elements/hstack.html)
A vertical stack of elements, accepts many children
```yaml
hstack
  text "one"
  text "two"
```
"#;

const BORDER: &'static str = r#"border [docs](https://togglebyte.github.io/anathema-guide/templates/elements/border.html)
A border around an element
```yaml
border
  text "hello"
```
"#;

const TEXT: &'static str = r#"text [docs](https://togglebyte.github.io/anathema-guide/templates/elements/text.html)
A text element
```yaml
text "hello"
```
"#;

impl HoverDictionary {
    pub fn new() -> Self {
        let mut dictionary = HashMap::new();
        dictionary.insert("vstack", VSTACK);
        dictionary.insert( "hstack", HSTACK);
        dictionary.insert("border", BORDER);
        dictionary.insert("text", TEXT);

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
