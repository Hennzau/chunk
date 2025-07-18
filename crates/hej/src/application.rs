//! The module defines the `Application` struct, which represents a UI application.

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

pub(crate) type StateFn<State> = Box<dyn Fn() -> State + Send>;
pub(crate) type UpdateFn<State, Message> = Box<dyn Fn(&mut State, Message) -> Task<Message> + Send>;
pub(crate) type RenderFn<State, Message> = Box<dyn Fn(&State) -> Element<Message> + Send>;

/// The `Application` struct represents a UI application with a state, update function, and render function.
/// Example usage:
///
/// ```rust
/// use std::{sync::Arc, time::Duration};
///
/// use hej::prelude::*;
///
/// let application =
///     Application::new(State::default, State::update, State::render)
///     .initial_task(Task::msg(Message::Nothing));
///
/// enum Message {
///     Nothing,
///     Error(Arc<Report>),
/// }
///
/// #[derive(Default)]
/// struct State {}
///
/// impl State {
///     fn update(&mut self, _message: Message) -> Task<Message> {
///         Task::stop()
///     }
///     fn render(&self) -> Element<Message> {
///         Element::empty()
///     }
/// }
/// ```
pub struct Application<State, Message> {
    pub(crate) state: StateFn<State>,
    pub(crate) update: UpdateFn<State, Message>,
    pub(crate) render: RenderFn<State, Message>,

    pub(crate) initial_task: Option<Task<Message>>,
}

impl<State: Send + 'static, Message: 'static + Send + Sync> Application<State, Message> {
    /// Creates a new `Application` instance with the provided state, update function, and render function.
    pub fn new(
        state: impl Fn() -> State + 'static + Send,
        update: impl Fn(&mut State, Message) -> Task<Message> + 'static + Send,
        render: impl Fn(&State) -> Element<Message> + 'static + Send,
    ) -> Self {
        Self {
            state: Box::new(state),
            update: Box::new(update),
            render: Box::new(render),
            initial_task: None,
        }
    }

    /// Sets the initial task to be executed when the application starts.
    pub fn initial_task(self, task: Task<Message>) -> Self {
        Self {
            initial_task: Some(task),
            ..self
        }
    }

    pub(crate) fn jobs<Backend: BackendTrait<Message>>(
        self,
        backend: Backend,
        on_error: impl Fn(Report) -> Message + 'static + Send + Sync,
    ) -> (
        JoinHandle<Result<()>>,
        JoinHandle<Result<()>>,
        JoinHandle<()>,
    ) {
        let (msg_client, mut msg_server) = unbounded_channel::<Message>();
        let (directive_client, mut directive_server) = unbounded_channel::<ApplicationDirective>();

        let (pool, tasks) = {
            let pool = TaskPool::<Message>::new();
            let tasks = Arc::new(pool.sender());
            let pool = task::spawn(pool.run(on_error, msg_client.clone(), directive_client));

            (pool, tasks)
        };

        if let Some(task) = self.initial_task {
            tracing::info!("Sending initial task to the task pool");
            tasks.send(task).unwrap_or_else(|e| {
                tracing::error!("Failed to send initial task: {}", e);
            });
        }

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

    /// Runs the application with the specified backend.
    /// Example usage:
    ///
    /// ```rust
    /// use std::{sync::Arc, time::Duration};
    ///
    /// use hej::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     Application::new(State::default, State::update, State::render)
    ///         .initial_task(Task::msg(Message::Nothing))
    ///         .run::<EmptyBackend<Message>>(|e| Message::Error(Arc::new(e)))
    ///         .await
    /// }
    ///
    /// enum Message {
    ///     Nothing,
    ///     Error(Arc<Report>),
    /// }
    ///
    /// #[derive(Default)]
    /// struct State {}
    ///
    /// impl State {
    ///     fn update(&mut self, _message: Message) -> Task<Message> {
    ///         Task::stop()
    ///     }
    ///     fn render(&self) -> Element<Message> {
    ///         Element::empty()
    ///     }
    /// }
    /// ```
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

                result.map_err(Report::msg)
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
