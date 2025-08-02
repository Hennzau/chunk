//! The module defines the `Application` struct, which represents a UI application.

use tokio::task::{self, JoinHandle};

use crate::prelude::*;

pub(crate) enum ApplicationDirective<Message> {
    Stop,

    ResetState,

    Submit(Element<Message>),
    Close(String),
}

pub(crate) type StateFn<State> = Box<dyn Fn() -> State + Send>;
pub(crate) type UpdateFn<State, Message> = Box<dyn Fn(&mut State, Message) -> Task<Message> + Send>;
pub(crate) type ViewFn<State, Message> = Box<dyn Fn(&State) -> Element<Message> + Send>;

/// The `Application` struct represents a UI application with a state, update function, and view function.
/// Example usage:
///
/// ```rust
/// use std::{sync::Arc, time::Duration};
///
/// use hej::prelude::*;
///
/// let application =
///     Application::new(State::default, State::update, State::view)
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
///     fn view(&self) -> Element<Message> {
///         Element::empty()
///     }
/// }
/// ```
pub struct Application<State, Message> {
    pub(crate) state: StateFn<State>,
    pub(crate) update: UpdateFn<State, Message>,
    pub(crate) view: ViewFn<State, Message>,

    pub(crate) initial_task: Option<Task<Message>>,
}

impl<State: Send + 'static, Message: 'static + Send + Sync> Application<State, Message> {
    /// Creates a new `Application` instance with the provided state, update function, and view function.
    pub fn new(
        state: impl Fn() -> State + 'static + Send,
        update: impl Fn(&mut State, Message) -> Task<Message> + 'static + Send,
        view: impl Fn(&State) -> Element<Message> + 'static + Send,
    ) -> Self {
        Self {
            state: Box::new(state),
            update: Box::new(update),
            view: Box::new(view),
            initial_task: None,
        }
    }

    /// Sets the initial task to be executed when the application starts.
    pub fn task(self, task: Task<Message>) -> Self {
        Self {
            initial_task: Some(task),
            ..self
        }
    }

    pub(crate) async fn jobs<T: Backend<Message>>(
        self,
        on_error: impl Fn(Report) -> Message + 'static + Send + Sync,
    ) -> Result<(
        JoinHandle<Result<()>>,
        JoinHandle<Result<()>>,
        JoinHandle<()>,
    )> {
        let (msg_submitter, mut msg_server) = channel::<Message>();
        let (directive_submitter, mut directive_server) =
            channel::<ApplicationDirective<Message>>();

        let (pool, tasks) = {
            let pool = TaskPool::<Message>::new();
            let tasks = pool.submitter();
            let pool = task::spawn(pool.run(on_error, msg_submitter.clone(), directive_submitter));

            (pool, tasks)
        };

        if let Some(task) = self.initial_task {
            tasks.submit(task).unwrap_or_else(|e| {
                tracing::error!("Failed to send initial task: {}", e);
            });
        }

        let mut state = (self.state)();

        let backend = T::new(msg_submitter.clone()).await?;

        let backend_submitter = backend.submitter();
        let backend_closer = backend.closer();

        let server = tokio::spawn(async move {
            tracing::info!("Server started");

            let element = (self.view)(&state);
            let mut labels = element.labels();

            backend_submitter.submit(element).unwrap_or_else(|e| {
                tracing::error!("Failed to submit element: {}", e);
            });

            loop {
                tokio::select! {
                    Ok(message) = msg_server.recv() => {
                        let task = (self.update)(&mut state, message);
                        tasks.submit(task).unwrap_or_else(|e| {
                            tracing::error!("Failed to send task: {}", e);
                        });

                        let element = (self.view)(&state);
                        let new_labels = element.labels();

                        backend_submitter.submit(element).unwrap_or_else(|e| {
                            tracing::error!("Failed to submit element: {}", e);
                        });

                        for label in labels {
                            if !new_labels.contains(&label) {
                                if let Some(label) = label {
                                    backend_closer.submit(label).unwrap_or_else(|e| {
                                        tracing::error!("Failed to submit a close request for this label: {}", e);
                                    });
                                }
                            }
                        }

                        labels = new_labels;
                    }
                    Ok(directive) = directive_server.recv() => {
                        match directive {
                            ApplicationDirective::Stop => break,
                            ApplicationDirective::ResetState => {
                                state = (self.state)();
                                tracing::info!("State has been reset");
                            },
                            ApplicationDirective::Submit(element) => {
                                backend_submitter.submit(element).unwrap_or_else(|e| {
                                    tracing::error!("Failed to submit element: {}", e);
                                });
                            },
                            ApplicationDirective::Close(label) => {
                                backend_closer.submit(label).unwrap_or_else(|e| {
                                    tracing::error!("Failed to submit a close request for this label: {}", e);
                                });
                            }
                        }
                    }
                }
            }

            Ok(())
        });

        let backend = tokio::spawn(backend.run());

        Ok((server, backend, pool))
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
    ///     Application::new(State::default, State::update, State::view)
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
    ///     fn view(&self) -> Element<Message> {
    ///         Element::empty()
    ///     }
    /// }
    /// ```
    pub async fn run<T: Backend<Message> + 'static>(
        self,
        on_error: impl Fn(Report) -> Message + 'static + Send + Sync,
    ) -> Result<()> {
        let (server, backend, pool) = self.jobs::<T>(on_error).await?;

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
