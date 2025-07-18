use std::{pin::Pin, time::Duration};

use crate::prelude::*;

#[derive(Debug)]
pub enum SpecialTask {
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

pub struct Task<Message> {
    pub(crate) handle: TaskHandle<Message>,
}

impl<Message: Sync + Send + 'static> Task<Message> {
    pub fn new(fut: impl Future<Output = Result<Message>> + Send + Sync + 'static) -> Self {
        Task {
            handle: TaskHandle::Simple(Box::pin(fut)),
        }
    }

    pub fn batch(self, other: Task<Message>) -> Self {
        Task {
            handle: TaskHandle::Batch(vec![self, other]),
        }
    }

    pub fn then(self, next: Task<Message>) -> Self {
        Task {
            handle: TaskHandle::Then(Box::new(self), Box::new(next)),
        }
    }

    pub fn stop() -> Self {
        Task {
            handle: TaskHandle::Special(SpecialTask::Stop),
        }
    }

    pub fn none() -> Self {
        Task {
            handle: TaskHandle::Special(SpecialTask::None),
        }
    }

    pub fn wait(duration: Duration, message: Message) -> Self {
        Task {
            handle: TaskHandle::Simple(Box::pin(async move {
                tokio::time::sleep(duration).await;

                Ok(message)
            })),
        }
    }

    pub fn reset_state() -> Self {
        Task {
            handle: TaskHandle::Special(SpecialTask::ResetState),
        }
    }

    pub fn msg(message: Message) -> Self {
        Task {
            handle: TaskHandle::Simple(Box::pin(async move { Ok(message) })),
        }
    }
}
