use std::sync::Arc;

use tokio::{
    sync::mpsc::unbounded_channel,
    task::{self, JoinHandle},
};

use crate::prelude::*;

pub(crate) enum ApplicationDirective {
    Stop,

    ResetState,
}

pub struct Application<State, Message> {
    pub(crate) state: Box<dyn Fn() -> State + Send>,
    pub(crate) update: Box<dyn Fn(&mut State, Message) -> Task<Message> + Send>,
    pub(crate) render: Box<dyn Fn(&State) -> Element<Message> + Send>,
}

impl<State: Send + 'static, Message: 'static + Send + Sync> Application<State, Message> {
    pub async fn new(
        state: impl Fn() -> State + 'static + Send,
        update: impl Fn(&mut State, Message) -> Task<Message> + 'static + Send,
        render: impl Fn(&State) -> Element<Message> + 'static + Send,
    ) -> Result<Self> {
        Ok(Self {
            state: Box::new(state),
            update: Box::new(update),
            render: Box::new(render),
        })
    }

    pub(crate) fn jobs<Backend: BackendTrait<Message>>(
        self,
        backend: Backend,
        on_error: impl Fn(Report) -> Message + 'static + Send + Sync,
    ) -> (
        JoinHandle<Result<()>>,
        JoinHandle<Result<()>>,
        JoinHandle<Result<()>>,
    ) {
        let (msg_client, mut msg_server) = unbounded_channel::<Message>();
        let (directive_client, mut directive_server) = unbounded_channel::<ApplicationDirective>();

        let (pool, tasks) = {
            let pool = TaskPool::<Message>::new();
            let tasks = Arc::new(pool.sender());
            let pool = task::spawn(pool.run(on_error, msg_client.clone(), directive_client));

            (pool, tasks)
        };

        let mut state = (self.state)();
        let backend_client = backend.client();

        let server = tokio::spawn(async move {
            tracing::info!("Server started");

            let element = (self.render)(&state);
            backend_client.send(element).unwrap_or_else(|e| {
                tracing::error!("Failed to submit element: {}", e);
            });

            loop {
                tokio::select! {
                    Some(message) = msg_server.recv() => {
                        let task = (self.update)(&mut state, message);
                        tasks.send(task).unwrap_or_else(|e| {
                            tracing::error!("Failed to send task: {}", e);
                        });

                        let element = (self.render)(&state);
                        backend_client.send(element).unwrap_or_else(|e| {
                            tracing::error!("Failed to submit element: {}", e);
                        });
                    }
                    Some(directive) = directive_server.recv() => {
                        match directive {
                            ApplicationDirective::Stop => break,
                            ApplicationDirective::ResetState => {
                                state = (self.state)();
                                tracing::info!("State has been reset");
                            }
                        }
                    }
                }
            }

            Ok(())
        });

        let backend = tokio::spawn(backend.run(msg_client));

        (server, backend, pool)
    }

    pub async fn run<Backend: BackendTrait<Message> + 'static>(
        self,
        on_error: impl Fn(Report) -> Message + 'static + Send + Sync,
    ) -> Result<()> {
        let backend = Backend::new().await?;

        let (server, backend, pool) = self.jobs(backend, on_error);

        let ctrl_c = tokio::signal::ctrl_c();

        tokio::select! {
            result = pool => {
                tracing::info!("Task pool has stopped");

                result?
            }
            result = server => {
                tracing::info!("Server task has stopped");

                result?
            }
            result = backend => {
                tracing::info!("Backend task has stopped");

                result?
            }
            result = ctrl_c => {
                tracing::info!("Received Ctrl+C, stopping application");

                result.map_err(Report::msg)
            }
        }
    }
}
