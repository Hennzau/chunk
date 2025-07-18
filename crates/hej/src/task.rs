//! Task management module for handling asynchronous tasks in a structured way.

use std::{pin::Pin, time::Duration};

use tokio::sync::oneshot::Sender;

use crate::prelude::*;

#[derive(Debug)]
pub(crate) enum SpecialTask {
    None,
    Stop,
    ResetState,
}

pub(crate) enum TaskHandle<Message> {
    Simple(Pin<Box<dyn Future<Output = Result<Message>> + Send + Sync + 'static>>),
    Batch(Vec<Task<Message>>),
    Then(Box<Task<Message>>, Box<Task<Message>>),
    Special(SpecialTask),
}

/// Represents a task that can be executed asynchronously.
pub struct Task<Message> {
    pub(crate) handle: TaskHandle<Message>,
    pub(crate) signal: Option<Sender<()>>,
}

impl<Message: Sync + Send + 'static> Task<Message> {
    /// Creates a new task with a future that resolves to a message.
    pub fn new(fut: impl Future<Output = Result<Message>> + Send + Sync + 'static) -> Self {
        Task {
            handle: TaskHandle::Simple(Box::pin(fut)),
            signal: None,
        }
    }

    /// Batch multiple tasks together to be executed in parallel.
    pub fn batch(self, other: Task<Message>) -> Self {
        Task {
            handle: TaskHandle::Batch(vec![self, other]),
            signal: None,
        }
    }

    /// Chains two tasks together, where the next task will only run after the first one completes.
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
}
