mod local_spawner;
mod hover_dictionary;
mod document_store;
mod lsp;
mod auto_complete;
mod word_handling;

use crate::local_spawner::{LocalSpawner};
use std::fs::File;
use std::sync::Mutex;
use tower_lsp::{LspService, Server};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;
use crate::lsp::Backend;

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
    }).finish();
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
