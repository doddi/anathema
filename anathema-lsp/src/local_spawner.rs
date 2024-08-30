use anathema_templates::error::Error;
use anathema_templates::error::Error::ParseError;
use log::{debug, trace};
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tokio::task::LocalSet;
use tower_lsp::lsp_types::{Diagnostic, MessageType, Range, Url};
use tower_lsp::{lsp_types, Client};

#[derive(Debug)]
pub(crate) enum Task {
    Compile(Url, String, Client),
}

#[derive(Clone)]
pub(crate) struct LocalSpawner {
    send: mpsc::UnboundedSender<Task>,
}

impl LocalSpawner {
    pub fn new() -> Self {
        let (send, mut recv) = mpsc::unbounded_channel();

        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        std::thread::spawn(move || {
            let local = LocalSet::new();

            local.spawn_local(async move {
                while let Some(new_task) = recv.recv().await {
                    trace!("Received task");
                    tokio::task::spawn_local(run_task(new_task));
                }
            });

            // This will return once all senders are dropped and all
            // spawned tasks have returned.
            rt.block_on(local);
        });

        Self { send }
    }

    pub fn spawn(&self, task: Task) {
        self.send.send(task).expect("Thread with LocalSet has shut down.");
    }
}

async fn run_task(task: Task) {
    match task {
        Task::Compile(uri, content, client) => {
            // We ignore failures to send the response.
            let compilation_result = anathema_templates::Document::new(content.clone()).compile();

            match compilation_result {
                Ok(_) => {
                    client
                        .log_message(MessageType::INFO, "anathema template compilation successful")
                        .await;
                    client.publish_diagnostics(uri, vec![], None).await;
                }
                Err(err) => {
                    debug!("Compilation error: {:?}", err);
                    match err {
                        ParseError(msg) => {
                            let line = msg.line;
                            let line_length = content.lines().nth(line - 1).unwrap().len();
                            let diagnostics = vec![Diagnostic::new_simple(
                                Range::new(
                                    lsp_types::Position::new(line as u32 - 1, 0),
                                    lsp_types::Position::new(line as u32 - 1, line_length as u32),
                                ),
                                format!("{:?}", msg.kind),
                            )];
                            client.publish_diagnostics(uri, diagnostics, None).await;
                        }
                        Error::CircularDependency => {
                            client
                                .show_message(MessageType::ERROR, "Circular dependency detected")
                                .await;
                        }
                        Error::MissingComponent(_component) => {
                            // Although this is an error, we dont have knowledge of components so will always get a failure here
                            // Assume it was a success, the most important thing is that the template is valid from a parsing pov
                            client
                                .log_message(MessageType::INFO, "anathema template compilation successful")
                                .await;
                            client.publish_diagnostics(uri, vec![], None).await;
                        }
                        Error::EmptyTemplate => {
                            client.show_message(MessageType::INFO, "Empty template").await;
                        }
                        Error::EmptyBody => {
                            client
                                .show_message(MessageType::WARNING, "if or else node has no children")
                                .await;
                        }
                        Error::Io(_) => {}
                    };
                }
            }
        }
    }
}
