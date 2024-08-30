use log::debug;
use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot};
use tokio::task::LocalSet;
use tower_lsp::lsp_types;
use tower_lsp::lsp_types::{Diagnostic, Range};
use anathema_templates::error::Error::ParseError;

#[derive(Debug)]
pub(crate) enum Task {
    Compile(String, oneshot::Sender<Option<Vec<Diagnostic>>>),
}

#[derive(Clone)]
pub(crate) struct LocalSpawner {
    send: mpsc::UnboundedSender<Task>,
}

impl LocalSpawner {
    pub fn new() -> Self {
        let (send, mut recv) = mpsc::unbounded_channel();

        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        std::thread::spawn(move || {
            let local = LocalSet::new();

            debug!("Creating local thread");
            local.spawn_local(async move {
                debug!("About to wait for tasks");
                while let Some(new_task) = recv.recv().await {
                    debug!("Received task");
                    tokio::task::spawn_local(run_task(new_task));
                }
                // If the while loop returns, then all the LocalSpawner
                // objects have been dropped.
            });

            // This will return once all senders are dropped and all
            // spawned tasks have returned.
            rt.block_on(local);
        });

        Self {
            send,
        }
    }

    pub fn spawn(&self, task: Task) {
        self.send.send(task).expect("Thread with LocalSet has shut down.");
    }
}

// of operations.
async fn run_task(task: Task) {
    match task {
        Task::Compile(content, response) => {
            // We ignore failures to send the response.
            debug!("Compiling content");
            let compilation_result = anathema_templates::Document::new(content.clone()).compile();
            debug!("Compilation result: {:?}", compilation_result);
            
            match compilation_result {
                Ok(_) => {
                    debug!("Compilation success");
                    let _ = response.send(None);
                },
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
                            let _ = response.send(Some(diagnostics));
                        }
                        _ => {
                            let _ = response.send(None);
                        },
                    }
                }
            }
        },
    }
}