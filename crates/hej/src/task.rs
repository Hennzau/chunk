use std::{pin::Pin, sync::Arc};

use tokio::{runtime::Handle, task::JoinHandle};

use crate::prelude::*;

pub enum Status {
    TaskRunning(JoinHandle<()>),

    TaskAskedForStop,
}

pub trait Executable<Message>: Send + Sync {
    fn execute(
        self: Box<Self>,
        runtime: Handle,
        client: Arc<Client<Message>>,
        on_err: Arc<dyn Fn(Report) -> Message + Send + Sync>,
    ) -> Status;
}

pub(crate) enum TaskHandle<Message> {
    Executable(Box<dyn Executable<Message>>),

    SpecialStop,
}

pub struct Task<Message> {
    pub(crate) task: TaskHandle<Message>,
}

impl<Message: 'static + Send + Sync> Task<Message> {
    pub fn new(
        task: impl Future<Output = Result<Message>> + 'static + Send + Sync,
    ) -> Task<Message> {
        Task {
            task: TaskHandle::Executable(Box::new(SimpleTask {
                payload: Box::pin(task),
            })),
        }
    }

    pub fn none() -> Option<Task<Message>> {
        None
    }

    pub fn some(
        task: impl Future<Output = Result<Message>> + 'static + Send + Sync,
    ) -> Option<Task<Message>> {
        Some(Task::new(task))
    }

    pub fn execute(
        self,
        runtime: Handle,
        client: Arc<Client<Message>>,
        on_err: Arc<dyn Fn(Report) -> Message + Send + Sync>,
    ) -> Status {
        match self.task {
            TaskHandle::Executable(task) => match task.execute(runtime, client, on_err) {
                Status::TaskRunning(handle) => Status::TaskRunning(handle),
                Status::TaskAskedForStop => {
                    tracing::info!("Task asked for stop");

                    Status::TaskAskedForStop
                }
            },
            TaskHandle::SpecialStop => {
                tracing::info!("Task asked for stop");

                Status::TaskAskedForStop
            }
        }
    }

    pub fn batch(self, other: Task<Message>) -> Task<Message> {
        Task {
            task: TaskHandle::Executable(Box::new(BatchTask {
                tasks: vec![self, other],
            })),
        }
    }

    pub fn then(self, other: Task<Message>) -> Task<Message> {
        Task {
            task: TaskHandle::Executable(Box::new(ThenTask {
                first: Box::new(self),
                second: Box::new(other),
            })),
        }
    }

    pub fn stop() -> Option<Task<Message>> {
        Some(Task {
            task: TaskHandle::SpecialStop,
        })
    }
}

pub(crate) struct SimpleTask<Message> {
    pub(crate) payload: Pin<Box<dyn Future<Output = Result<Message>> + Send + Sync + 'static>>,
}

impl<Message: 'static + Send + Sync> Executable<Message> for SimpleTask<Message> {
    fn execute(
        self: Box<Self>,
        runtime: Handle,
        client: Arc<Client<Message>>,
        on_err: Arc<dyn Fn(Report) -> Message + Send + Sync>,
    ) -> Status {
        Status::TaskRunning(runtime.spawn(async move {
            if let Err(report) = self.payload.await {
                tracing::error!("Task execution failed: {}", report);
                let message = on_err(report);

                client.send_no_result(message);
            }
        }))
    }
}

pub(crate) struct BatchTask<Message> {
    pub(crate) tasks: Vec<Task<Message>>,
}

impl<Message: 'static + Send + Sync> Executable<Message> for BatchTask<Message> {
    fn execute(
        self: Box<Self>,
        runtime: Handle,
        client: Arc<Client<Message>>,
        on_err: Arc<dyn Fn(Report) -> Message + Send + Sync>,
    ) -> Status {
        let mut futures = Vec::with_capacity(self.tasks.len());

        for task in self.tasks {
            let status = task.execute(runtime.clone(), client.clone(), on_err.clone());

            match status {
                Status::TaskAskedForStop => {
                    tracing::info!(
                        "A task in the batch asked for stop, it will abort the entire batch."
                    );

                    return Status::TaskAskedForStop;
                }
                Status::TaskRunning(handle) => {
                    futures.push(handle);
                }
            }
        }

        Status::TaskRunning(tokio::spawn(async move {
            for future in futures {
                if let Err(report) = future.await {
                    tracing::error!("Error joining a task in a batch: {}", report);
                }
            }
        }))
    }
}

pub(crate) struct ThenTask<Message> {
    pub(crate) first: Box<Task<Message>>,
    pub(crate) second: Box<Task<Message>>,
}

impl<Message: 'static + Send + Sync> Executable<Message> for ThenTask<Message> {
    fn execute(
        self: Box<Self>,
        runtime: Handle,
        client: Arc<Client<Message>>,
        on_err: Arc<dyn Fn(Report) -> Message + Send + Sync>,
    ) -> Status {
        let first = match self
            .first
            .execute(runtime.clone(), client.clone(), on_err.clone())
        {
            Status::TaskAskedForStop => {
                tracing::info!("First task asked for stop, the second task will be skipped.");

                return Status::TaskAskedForStop;
            }
            Status::TaskRunning(handle) => handle,
        };

        let second = self.second;
        Status::TaskRunning(tokio::spawn(async move {
            let first = first.await;

            if let Err(report) = first {
                tracing::error!("failed to join first task, aborting: {}", report);

                return;
            }

            let second = match second.execute(runtime, client, on_err) {
                Status::TaskAskedForStop => {
                    tracing::info!(
                        "Second task asked for stop,
                        but the runtime cannot be stopped at this stage.
                        Stop tasks should not be used in a 'then' task.
                        Consider redirecting your stop signal as a Message
                        and then stop the runtime in a simple special task."
                    );

                    return;
                }
                Status::TaskRunning(handle) => handle,
            }
            .await;

            if let Err(report) = second {
                tracing::error!("failed to join second task: {}", report);
            }
        }))
    }
}
