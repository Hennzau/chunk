//! Task management module for handling asynchronous tasks in a structured way.

use std::{pin::Pin, time::Duration};

use tokio::sync::oneshot::Sender;

use crate::prelude::*;

pub(crate) enum SpecialTask<Message> {
    None,

    Stop,
    ResetState,

    Submit(Element<Message>),
    Close(String),
}

pub(crate) enum TaskHandle<Message> {
    Simple(Pin<Box<dyn Future<Output = Result<Message>> + Send + Sync + 'static>>),
    Batch(Vec<Task<Message>>),
    Then(Box<Task<Message>>, Box<Task<Message>>),
    Special(SpecialTask<Message>),
}

/// Represents a task that can be executed asynchronously.
///
/// Example usage:
/// ```rust
/// use hej::prelude::{reexport::*, *};
/// use std::time::Duration;
///
/// let task = Task::new(async {
///    Ok("Hello, World!")
/// });
///
/// let task = task.then(Task::wait(Duration::from_secs(1), "Done!"));
/// ```
pub struct Task<Message> {
    pub(crate) handle: TaskHandle<Message>,
    pub(crate) signal: Option<Sender<()>>,
}

impl<Message: Sync + Send + 'static> Task<Message> {
    /// Creates a new task with a future that resolves to a message.
    /// Example:
    /// ```rust
    /// use hej::prelude::{reexport::*, *};
    ///
    /// enum Message {
    ///     Nothing
    /// }
    ///
    /// let task = Task::new(async {
    ///     Ok(Message::Nothing)
    /// });
    /// ```
    pub fn new(fut: impl Future<Output = Result<Message>> + Send + Sync + 'static) -> Self {
        Task {
            handle: TaskHandle::Simple(Box::pin(fut)),
            signal: None,
        }
    }

    /// Batch multiple tasks together to be executed in parallel.
    /// Example:
    /// ```rust
    /// use hej::prelude::{reexport::*, *};
    ///
    /// enum Message {
    ///     Nothing
    /// }
    ///
    /// let mut task = Task::none();
    ///
    /// for _ in 0..10 {
    ///    task = task.batch(Task::msg(Message::Nothing));
    /// }
    /// ```
    pub fn batch(self, other: Task<Message>) -> Self {
        Task {
            handle: TaskHandle::Batch(vec![self, other]),
            signal: None,
        }
    }

    /// Chains two tasks together, where the next task will only run after the first one completes.
    /// Example usage:
    /// ```rust
    /// use hej::prelude::{reexport::*, *};
    /// use std::time::Duration;
    ///
    /// let task = Task::new(async {
    ///    Ok("Hello, World!")
    /// });
    ///
    /// let task = task.then(Task::wait(Duration::from_secs(1), "Done!"));
    /// ```
    pub fn then(self, next: Task<Message>) -> Self {
        Task {
            handle: TaskHandle::Then(Box::new(self), Box::new(next)),
            signal: None,
        }
    }

    /// Creates a special task that stops the application.
    pub fn stop() -> Self {
        Task {
            handle: TaskHandle::Special(SpecialTask::Stop),
            signal: None,
        }
    }

    /// Creates a special task that does nothing.
    pub fn none() -> Self {
        Task {
            handle: TaskHandle::Special(SpecialTask::None),
            signal: None,
        }
    }

    /// Creates a task that waits for a specified duration before resolving with a message.
    pub fn wait(duration: Duration, message: Message) -> Self {
        Task {
            handle: TaskHandle::Simple(Box::pin(async move {
                tokio::time::sleep(duration).await;

                Ok(message)
            })),
            signal: None,
        }
    }

    /// Creates a special task that resets the application state.
    pub fn reset_state() -> Self {
        Task {
            handle: TaskHandle::Special(SpecialTask::ResetState),
            signal: None,
        }
    }

    /// Creates a task that resolves immediately with the provided message.
    pub fn msg(message: Message) -> Self {
        Task {
            handle: TaskHandle::Simple(Box::pin(async move { Ok(message) })),
            signal: None,
        }
    }

    pub fn submit(element: Element<Message>) -> Self {
        Task {
            handle: TaskHandle::Special(SpecialTask::Submit(element)),
            signal: None,
        }
    }

    pub fn close(label: impl Into<String>) -> Self {
        Task {
            handle: TaskHandle::Special(SpecialTask::Close(label.into())),
            signal: None,
        }
    }

    /// Maps this Task<Message> to another Task<NewMessage>
    pub fn map<NewMessage: 'static + Send + Sync>(
        self,
        map: Map<Message, NewMessage>,
    ) -> Task<NewMessage> {
        Task {
            handle: match self.handle {
                TaskHandle::Simple(fut) => TaskHandle::Simple(Box::pin(async move {
                    let message = fut.await?;
                    Ok(map.map(message))
                })),
                TaskHandle::Batch(tasks) => TaskHandle::Batch(
                    tasks
                        .into_iter()
                        .map(|task| task.map(map.clone()))
                        .collect(),
                ),
                TaskHandle::Then(first, second) => TaskHandle::Then(
                    Box::new(first.map(map.clone())),
                    Box::new(second.map(map.clone())),
                ),
                TaskHandle::Special(special) => match special {
                    SpecialTask::None => TaskHandle::Special(SpecialTask::None),
                    SpecialTask::ResetState => TaskHandle::Special(SpecialTask::ResetState),
                    SpecialTask::Stop => TaskHandle::Special(SpecialTask::Stop),
                    SpecialTask::Submit(element) => {
                        TaskHandle::Special(SpecialTask::Submit(element.map(map.clone())))
                    }
                    SpecialTask::Close(label) => TaskHandle::Special(SpecialTask::Close(label)),
                },
            },
            signal: None,
        }
    }
}
